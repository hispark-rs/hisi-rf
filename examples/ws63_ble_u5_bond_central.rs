#![no_std]
#![no_main]

use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

#[cfg(all(feature = "u5-pairing-reject-hil", feature = "u5-pairing-stale-hil"))]
compile_error!("select exactly one U5D negative pairing mode");

#[path = "support/ws63_u2_firmware.rs"]
#[allow(dead_code)]
mod firmware;

const PEER_PAYLOAD: &[u8] = &[
    2, 0x01, 0x06, 9, 0x09, b'H', b'I', b'S', b'I', b'U', b'5', b'P', b'R',
];

#[entry]
fn main() -> ! {
    firmware::run_with_uart(run)
}

fn run(
    mut parts: hisi_rf::ws63::RadioParts,
    uart: hisi_hal::uart::Uart<'static, hisi_hal::peripherals::Uart0<'static>>,
) -> ! {
    let restored = restore_bonds(&mut parts);
    if restored && negative_pairing_mode() {
        firmware::fail(b"RFDBG_RADIO_U5_BLE_NEGATIVE_REQUIRES_EMPTY\r\n");
    }
    submit_security(&mut parts);
    start_scan(&mut parts);
    let mut scanning = None;
    let mut peer = None;
    let mut connection: Option<hisi_rf::ws63::BleConnection> = None;
    let mut pair_command = None;
    let mut state_command = None;
    let mut remove_command = None;
    let mut response_command = None;
    let mut pending_responder = None;
    let mut passkey_input = PasskeyInput::new();
    let mut response_completed = false;
    let mut disconnected = false;
    let mut negative_reported = false;
    let mut pair_accepted = restored;
    let mut paired = restored;
    let mut state_confirmed = !restored;
    let mut authenticated = false;
    let mut observed = restored;
    let mut removal_confirmed = false;

    loop {
        progress(&mut parts);
        while let Some(completion) = parts.ble.try_take_completion().unwrap() {
            let id = completion.id();
            let result = completion.into_result();
            if pair_command == Some(id) {
                if result.is_err() {
                    firmware::fail(b"RFDBG_RADIO_U5_BLE_COMMAND_ERR\r\n");
                }
                pair_accepted = true;
                firmware::log(b"RFDBG_RADIO_U5_BLE_PAIR_ACCEPTED\r\n");
            } else if state_command == Some(id) {
                match result {
                    Ok(hisi_rf::ws63::BleOperation::PairingState(
                        hisi_rf::ble::PairingState::Paired,
                    )) => {
                        firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_RESTORED_ACTIVE\r\n");
                        if cfg!(feature = "u5-bond-removal-hil") {
                            let peer = connection
                                .as_ref()
                                .unwrap_or_else(|| {
                                    firmware::fail(b"RFDBG_RADIO_U5_BLE_STATE_LINK_ERR\r\n")
                                })
                                .peer();
                            remove_command =
                                Some(parts.ble.try_remove_bond(peer).unwrap_or_else(|_| {
                                    firmware::fail(b"RFDBG_RADIO_U5_BLE_REMOVE_QUEUE_ERR\r\n")
                                }));
                        } else {
                            state_confirmed = true;
                        }
                    }
                    _ => firmware::fail(b"RFDBG_RADIO_U5_BLE_RESTORED_STATE_ERR\r\n"),
                }
            } else if remove_command == Some(id) {
                match result {
                    Ok(hisi_rf::ws63::BleOperation::BondRemoved) => {
                        state_confirmed = true;
                        removal_confirmed = true;
                        firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_REMOVED\r\n");
                    }
                    _ => firmware::fail(b"RFDBG_RADIO_U5_BLE_REMOVE_ERR\r\n"),
                }
            } else if response_command == Some(id) {
                match (pairing_mode(), result) {
                    (
                        PairingMode::Passkey,
                        Ok(hisi_rf::ws63::BleOperation::PairingResponseAccepted),
                    ) => {
                        response_completed = true;
                        firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_PASSKEY_ACCEPTED\r\n");
                    }
                    (
                        PairingMode::Reject,
                        Ok(hisi_rf::ws63::BleOperation::PairingResponseAccepted),
                    ) => {
                        response_completed = true;
                        firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_REJECT_ACCEPTED\r\n");
                    }
                    (PairingMode::Stale, Err(error))
                        if error.kind() == hisi_rf::ws63::BleOperationErrorKind::StaleLifecycle =>
                    {
                        response_completed = true;
                        firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_STALE_REJECTED\r\n");
                    }
                    _ => firmware::fail(b"RFDBG_RADIO_U5_BLE_PASSKEY_RESPONSE_ERR\r\n"),
                }
            } else if result.is_err() {
                firmware::fail(b"RFDBG_RADIO_U5_BLE_COMMAND_ERR\r\n");
            }
        }
        while let Some(event) = parts.ble.try_next_event() {
            match event {
                hisi_rf::ws63::BleEvent::ScanReady { scanner } => {
                    scanning = Some(scanner);
                }
                hisi_rf::ws63::BleEvent::PeerObserved {
                    peer: found,
                    payload,
                    ..
                } if peer.is_none() && payload.as_bytes() == PEER_PAYLOAD => {
                    peer = Some(found);
                    let scanner = scanning.take().unwrap_or_else(|| {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_SCAN_LIFECYCLE_ERR\r\n")
                    });
                    firmware::wait_with_runner(scanner.stop(), || progress(&mut parts))
                        .unwrap_or_else(|_| {
                            firmware::fail(b"RFDBG_RADIO_U5_BLE_SCAN_STOP_ERR\r\n")
                        });
                    parts.ble.try_connect(found).unwrap_or_else(|_| {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_CONNECT_QUEUE_ERR\r\n")
                    });
                    firmware::log(b"RFDBG_RADIO_U5_BLE_SCAN_MATCH\r\n");
                }
                hisi_rf::ws63::BleEvent::Connected { connection: link } => {
                    // The restored bond supplies keys, but the central still has to
                    // initiate the security procedure for the new connection.
                    pair_command = Some(parts.ble.try_pair(&link).unwrap_or_else(|_| {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_PAIR_QUEUE_ERR\r\n")
                    }));
                    connection = Some(link);
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_CONNECTED\r\n");
                }
                hisi_rf::ws63::BleEvent::PairingComplete { result: Ok(()), .. } => {
                    if negative_pairing_mode() {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_NEGATIVE_PAIRED_ERR\r\n");
                    }
                    paired = true;
                    if restored && state_command.is_none() {
                        let link = connection.as_ref().unwrap_or_else(|| {
                            firmware::fail(b"RFDBG_RADIO_U5_BLE_STATE_LINK_ERR\r\n")
                        });
                        state_command = Some(
                            parts
                                .ble
                                .try_query_pairing_state(link.peer())
                                .unwrap_or_else(|_| {
                                    firmware::fail(b"RFDBG_RADIO_U5_BLE_STATE_QUEUE_ERR\r\n")
                                }),
                        );
                    }
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_PAIRED\r\n");
                }
                hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Ok(()), .. } => {
                    if negative_pairing_mode() {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_NEGATIVE_AUTH_ERR\r\n");
                    }
                    authenticated = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_AUTH_OK\r\n");
                }
                hisi_rf::ws63::BleEvent::PasskeyInputRequested { responder, .. } => {
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_PASSKEY_INPUT\r\n");
                    match pairing_mode() {
                        PairingMode::Passkey => pending_responder = Some(responder),
                        PairingMode::Reject => {
                            response_command = Some(
                                parts
                                    .ble
                                    .try_respond_to_pairing(
                                        responder,
                                        hisi_rf::ws63::PairingResponse::Reject,
                                    )
                                    .unwrap_or_else(|_| {
                                        firmware::fail(b"RFDBG_RADIO_U5_BLE_PASSKEY_QUEUE_ERR\r\n")
                                    }),
                            );
                        }
                        PairingMode::Stale => {
                            let link = connection.take().unwrap_or_else(|| {
                                firmware::fail(b"RFDBG_RADIO_U5_BLE_STATE_LINK_ERR\r\n")
                            });
                            firmware::wait_with_runner(link.disconnect(), || progress(&mut parts))
                                .unwrap_or_else(|_| {
                                    firmware::fail(b"RFDBG_RADIO_U5_BLE_STALE_DISCONNECT_ERR\r\n")
                                });
                            response_command = Some(
                                parts
                                    .ble
                                    .try_respond_to_pairing(
                                        responder,
                                        hisi_rf::ws63::PairingResponse::Passkey(
                                            hisi_rf::ble::Passkey::try_new(0).unwrap(),
                                        ),
                                    )
                                    .unwrap_or_else(|_| {
                                        firmware::fail(b"RFDBG_RADIO_U5_BLE_PASSKEY_QUEUE_ERR\r\n")
                                    }),
                            );
                        }
                    }
                }
                hisi_rf::ws63::BleEvent::Disconnected { .. } => {
                    disconnected = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_NEGATIVE_DISCONNECTED\r\n");
                }
                hisi_rf::ws63::BleEvent::VendorManagedBondObserved { .. } => {
                    if negative_pairing_mode() {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_NEGATIVE_BOND_ERR\r\n");
                    }
                    observed = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_OBSERVED\r\n");
                }
                hisi_rf::ws63::BleEvent::PairingComplete { result: Err(_), .. }
                | hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Err(_), .. }
                    if negative_pairing_mode() =>
                {
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_NEGATIVE_SECURITY_END\r\n");
                }
                hisi_rf::ws63::BleEvent::BackendError { .. }
                | hisi_rf::ws63::BleEvent::PairingComplete { result: Err(_), .. }
                | hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Err(_), .. } => {
                    firmware::fail(b"RFDBG_RADIO_U5_BLE_CENTRAL_ERR\r\n")
                }
                _ => {}
            }
        }
        if response_command.is_none()
            && pairing_mode() == PairingMode::Passkey
            && let Some(passkey) = passkey_input.poll(&uart)
            && let Some(responder) = pending_responder.take()
        {
            response_command = Some(
                parts
                    .ble
                    .try_respond_to_pairing(
                        responder,
                        hisi_rf::ws63::PairingResponse::Passkey(passkey),
                    )
                    .unwrap_or_else(|_| {
                        firmware::fail(b"RFDBG_RADIO_U5_BLE_PASSKEY_QUEUE_ERR\r\n")
                    }),
            );
        }
        if negative_pairing_mode() && response_completed && disconnected && !negative_reported {
            assert_negative_conservation(&parts);
            match pairing_mode() {
                PairingMode::Reject => firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_REJECT_OK\r\n"),
                PairingMode::Stale => firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_STALE_OK\r\n"),
                PairingMode::Passkey => unreachable!(),
            }
            negative_reported = true;
        }
        if peer.is_some()
            && connection.is_some()
            && (pair_command.is_some() || state_command.is_some())
            && pair_accepted
            && paired
            && state_confirmed
            && (authenticated || restored)
            && observed
            && (!restored || !cfg!(feature = "u5-bond-removal-hil") || removal_confirmed)
        {
            let diagnostics = parts.runner.bond_observation_diagnostics();
            if diagnostics.received
                != diagnostics.processed + diagnostics.dropped + diagnostics.pending
                || diagnostics.dropped != 0
                || diagnostics.pending != 0
            {
                firmware::fail(b"RFDBG_RADIO_U5_BLE_BOND_CONSERVATION_ERR\r\n");
            }
            if restored && cfg!(feature = "u5-bond-removal-hil") {
                firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_REMOVE_OK\r\n");
            }
            firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_OK\r\n");
            loop {
                progress(&mut parts);
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PairingMode {
    Passkey,
    Reject,
    Stale,
}

const fn pairing_mode() -> PairingMode {
    if cfg!(feature = "u5-pairing-reject-hil") {
        PairingMode::Reject
    } else if cfg!(feature = "u5-pairing-stale-hil") {
        PairingMode::Stale
    } else {
        PairingMode::Passkey
    }
}

const fn negative_pairing_mode() -> bool {
    !matches!(pairing_mode(), PairingMode::Passkey)
}

fn assert_negative_conservation(parts: &hisi_rf::ws63::RadioParts) {
    let events = parts.ble.event_diagnostics();
    let bonds = parts.runner.bond_observation_diagnostics();
    if events.accepted != events.consumed
        || events.dropped != 0
        || events.pending != 0
        || bonds.received != bonds.processed + bonds.dropped + bonds.pending
        || bonds.dropped != 0
        || bonds.pending != 0
    {
        firmware::fail(b"RFDBG_RADIO_U5_BLE_NEGATIVE_CONSERVATION_ERR\r\n");
    }
}

fn restore_bonds(parts: &mut hisi_rf::ws63::RadioParts) -> bool {
    match parts.runner.restore_vendor_managed_bonds() {
        Ok(0) => {
            firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_EMPTY\r\n");
            false
        }
        Ok(_) => {
            firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_RESTORED\r\n");
            true
        }
        Err(_) => firmware::fail(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_RESTORE_ERR\r\n"),
    }
}

fn submit_security(parts: &mut hisi_rf::ws63::RadioParts) {
    let config = hisi_rf::ble::SecurityConfig::new(
        hisi_rf::ble::Bonding::Enabled,
        hisi_rf::ble::IoCapability::KeyboardOnly,
        hisi_rf::ble::SecurityRequirement::SecureConnectionsAuthenticated,
    );
    let command = parts.ble.try_configure_security(config).unwrap();
    wait_command(parts, command);
}

struct PasskeyInput {
    matched_prefix: usize,
    digits: [u8; 6],
    digit_count: usize,
}

impl PasskeyInput {
    const PREFIX: &'static [u8] = b"U5PASS=";

    const fn new() -> Self {
        Self {
            matched_prefix: 0,
            digits: [0; 6],
            digit_count: 0,
        }
    }

    fn poll(
        &mut self,
        uart: &hisi_hal::uart::Uart<'_, hisi_hal::peripherals::Uart0<'_>>,
    ) -> Option<hisi_rf::ble::Passkey> {
        while let Some(byte) = uart.read_byte() {
            if self.matched_prefix < Self::PREFIX.len() {
                self.matched_prefix = if byte == Self::PREFIX[self.matched_prefix] {
                    self.matched_prefix + 1
                } else {
                    0
                };
                continue;
            }
            if byte.is_ascii_digit() && self.digit_count < self.digits.len() {
                self.digits[self.digit_count] = byte - b'0';
                self.digit_count += 1;
                continue;
            }
            let passkey = (byte == b'\n' || byte == b'\r')
                .then(|| {
                    self.digits
                        .iter()
                        .fold(0u32, |value, digit| value * 10 + u32::from(*digit))
                })
                .and_then(hisi_rf::ble::Passkey::try_new)
                .filter(|_| self.digit_count == self.digits.len());
            self.matched_prefix = 0;
            self.digit_count = 0;
            if passkey.is_some() {
                return passkey;
            }
        }
        None
    }
}

fn start_scan(parts: &mut hisi_rf::ws63::RadioParts) {
    let interval = hisi_rf::ble::ScanInterval::try_from_units(0x20).unwrap();
    let config = hisi_rf::ble::ScanConfig::new(
        hisi_rf::ble::ScanTiming::try_new(interval, interval).unwrap(),
        hisi_rf::ble::ScanMode::Passive,
        false,
    );
    let command = parts.ble.try_start_scanning(config).unwrap();
    wait_command(parts, command);
}

fn wait_command(parts: &mut hisi_rf::ws63::RadioParts, command: hisi_rf::ProtocolCommandId) {
    loop {
        progress(parts);
        if let Some(completion) = parts.ble.try_take_completion().unwrap() {
            if completion.id() == command && completion.into_result().is_ok() {
                return;
            }
            firmware::fail(b"RFDBG_RADIO_U5_BLE_COMMAND_ERR\r\n");
        }
    }
}

fn progress(parts: &mut hisi_rf::ws63::RadioParts) {
    if parts.runner.run_once().is_err() {
        firmware::fail(b"RFDBG_RADIO_U5_BLE_RUNNER_ERR\r\n");
    }
    while parts.runner.run_event_once() {}
    if parts.runner.dropped_events() != 0 || parts.ble.event_diagnostics().dropped != 0 {
        firmware::fail(b"RFDBG_RADIO_U5_BLE_EVENT_DROP\r\n");
    }
    firmware::drive_scheduler();
}
