#![no_std]
#![no_main]

use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

#[path = "support/ws63_u2_firmware.rs"]
#[allow(dead_code)]
mod firmware;

const PEER_PAYLOAD: &[u8] = &[
    2, 0x01, 0x06, 9, 0x09, b'H', b'I', b'S', b'I', b'U', b'5', b'P', b'R',
];

#[entry]
fn main() -> ! {
    firmware::run(run)
}

fn run(mut parts: hisi_rf::ws63::RadioParts) -> ! {
    let restored = restore_bonds(&mut parts);
    submit_security(&mut parts);
    start_scan(&mut parts);
    let mut scanning = None;
    let mut peer = None;
    let mut connection = None;
    let mut pair_command = None;
    let mut state_command = None;
    let mut pair_accepted = restored;
    let mut paired = restored;
    let mut state_confirmed = !restored;
    let mut authenticated = false;
    let mut observed = restored;

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
                        state_confirmed = true;
                        firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_RESTORED_ACTIVE\r\n");
                    }
                    _ => firmware::fail(b"RFDBG_RADIO_U5_BLE_RESTORED_STATE_ERR\r\n"),
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
                    if !restored {
                        pair_command = Some(parts.ble.try_pair(&link).unwrap_or_else(|_| {
                            firmware::fail(b"RFDBG_RADIO_U5_BLE_PAIR_QUEUE_ERR\r\n")
                        }));
                    }
                    connection = Some(link);
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_CONNECTED\r\n");
                }
                hisi_rf::ws63::BleEvent::PairingComplete { result: Ok(()), .. } => {
                    paired = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_PAIRED\r\n");
                }
                hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Ok(()), .. } => {
                    authenticated = true;
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
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_AUTH_OK\r\n");
                }
                hisi_rf::ws63::BleEvent::VendorManagedBondObserved { .. } => {
                    observed = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_OBSERVED\r\n");
                }
                hisi_rf::ws63::BleEvent::BackendError { .. }
                | hisi_rf::ws63::BleEvent::PairingComplete { result: Err(_), .. }
                | hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Err(_), .. } => {
                    firmware::fail(b"RFDBG_RADIO_U5_BLE_CENTRAL_ERR\r\n")
                }
                _ => {}
            }
        }
        if peer.is_some()
            && connection.is_some()
            && (pair_command.is_some() || state_command.is_some())
            && pair_accepted
            && paired
            && state_confirmed
            && authenticated
            && observed
        {
            let diagnostics = parts.runner.bond_observation_diagnostics();
            if diagnostics.received
                != diagnostics.processed + diagnostics.dropped + diagnostics.pending
                || diagnostics.dropped != 0
                || diagnostics.pending != 0
            {
                firmware::fail(b"RFDBG_RADIO_U5_BLE_BOND_CONSERVATION_ERR\r\n");
            }
            firmware::log(b"RFDBG_RADIO_U5_BLE_CENTRAL_BOND_OK\r\n");
            loop {
                progress(&mut parts);
            }
        }
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
        hisi_rf::ble::IoCapability::NoInputNoOutput,
        hisi_rf::ble::SecurityRequirement::Encrypted,
    );
    let command = parts.ble.try_configure_security(config).unwrap();
    wait_command(parts, command);
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
