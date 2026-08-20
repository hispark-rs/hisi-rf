#![no_std]
#![no_main]

use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

#[path = "support/ws63_u2_firmware.rs"]
#[allow(dead_code)]
mod firmware;

const PAYLOAD: &[u8] = &[
    2, 0x01, 0x06, 9, 0x09, b'H', b'I', b'S', b'I', b'U', b'5', b'P', b'R',
];

#[entry]
fn main() -> ! {
    firmware::run(run)
}

fn run(mut parts: hisi_rf::ws63::RadioParts) -> ! {
    let restored = restore_bonds(&mut parts);
    submit_security(&mut parts);
    let interval = hisi_rf::ble::AdvertisingInterval::try_from_units(0x20).unwrap();
    let config = hisi_rf::ble::AdvertisingConfig::new(
        hisi_rf::ble::AdvertisingTiming::try_new(interval, interval).unwrap(),
        hisi_rf::ble::AdvertisingChannels::ALL,
        hisi_rf::ble::AdvertisingPayload::try_from_slice(PAYLOAD).unwrap(),
    );
    let command = parts
        .ble
        .try_start_advertising(config)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U5_BLE_ADV_QUEUE_ERR\r\n"));
    let mut advertising = None;
    let mut connection: Option<hisi_rf::ws63::BleConnection> = None;
    let mut paired = restored;
    let mut authenticated = false;
    let mut security_completed = false;
    let mut observed = restored;
    let mut remove_command = None;
    let mut removal_confirmed = false;
    let mut reported = false;

    loop {
        progress(&mut parts);
        if let Some(completion) = parts
            .ble
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U5_BLE_COMPLETION_ERR\r\n"))
        {
            let id = completion.id();
            let result = completion.into_result();
            if id == command {
                if result.is_err() {
                    firmware::fail(b"RFDBG_RADIO_U5_BLE_ADV_COMMAND_ERR\r\n");
                }
            } else if remove_command == Some(id) {
                match result {
                    Ok(hisi_rf::ws63::BleOperation::BondRemoved) => {
                        removal_confirmed = true;
                        firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_REMOVED\r\n");
                    }
                    _ => firmware::fail(b"RFDBG_RADIO_U5_BLE_REMOVE_ERR\r\n"),
                }
            } else {
                firmware::fail(b"RFDBG_RADIO_U5_BLE_COMMAND_ERR\r\n");
            }
        }
        while let Some(event) = parts.ble.try_next_event() {
            match event {
                hisi_rf::ws63::BleEvent::AdvertisingStarted { advertiser } => {
                    advertising = Some(advertiser);
                    firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_READY\r\n");
                }
                hisi_rf::ws63::BleEvent::Connected { connection: link } => {
                    connection = Some(link);
                    firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_CONNECTED\r\n");
                }
                hisi_rf::ws63::BleEvent::PairingComplete { result: Ok(()), .. } => {
                    paired = true;
                    security_completed = true;
                    if restored {
                        firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_RESTORED_ACTIVE\r\n");
                    }
                    firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_PAIRED\r\n");
                }
                hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Ok(()), .. } => {
                    authenticated = true;
                    security_completed = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_AUTH_OK\r\n");
                }
                hisi_rf::ws63::BleEvent::PasskeyDisplayed { passkey, .. } => {
                    firmware::log_passkey(
                        b"RFDBG_RADIO_U5_BLE_PERIPHERAL_PASSKEY_DISPLAY=",
                        passkey,
                    );
                }
                hisi_rf::ws63::BleEvent::VendorManagedBondObserved { .. } => {
                    observed = true;
                    firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_OBSERVED\r\n");
                }
                hisi_rf::ws63::BleEvent::BackendError { .. }
                | hisi_rf::ws63::BleEvent::PairingComplete { result: Err(_), .. }
                | hisi_rf::ws63::BleEvent::AuthenticationComplete { result: Err(_), .. } => {
                    firmware::fail(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_ERR\r\n")
                }
                _ => {}
            }
        }
        if advertising.is_some()
            && connection.is_some()
            && paired
            && security_completed
            && (authenticated || restored)
            && observed
            && !reported
        {
            let diagnostics = parts.runner.bond_observation_diagnostics();
            if diagnostics.received
                != diagnostics.processed + diagnostics.dropped + diagnostics.pending
                || diagnostics.dropped != 0
                || diagnostics.pending != 0
            {
                firmware::fail(b"RFDBG_RADIO_U5_BLE_BOND_CONSERVATION_ERR\r\n");
            }
            if restored && cfg!(feature = "u5-bond-removal-hil") && remove_command.is_none() {
                let peer = connection
                    .as_ref()
                    .unwrap_or_else(|| firmware::fail(b"RFDBG_RADIO_U5_BLE_STATE_LINK_ERR\r\n"))
                    .peer();
                remove_command = Some(parts.ble.try_remove_bond(peer).unwrap_or_else(|_| {
                    firmware::fail(b"RFDBG_RADIO_U5_BLE_REMOVE_QUEUE_ERR\r\n")
                }));
                continue;
            }
            if !restored || !cfg!(feature = "u5-bond-removal-hil") || removal_confirmed {
                if restored && cfg!(feature = "u5-bond-removal-hil") {
                    firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_REMOVE_OK\r\n");
                }
                firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_OK\r\n");
                reported = true;
            }
        }
    }
}

fn restore_bonds(parts: &mut hisi_rf::ws63::RadioParts) -> bool {
    match parts.runner.restore_vendor_managed_bonds() {
        Ok(0) => {
            firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_EMPTY\r\n");
            false
        }
        Ok(_) => {
            firmware::log(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_RESTORED\r\n");
            true
        }
        Err(_) => firmware::fail(b"RFDBG_RADIO_U5_BLE_PERIPHERAL_BOND_RESTORE_ERR\r\n"),
    }
}

fn submit_security(parts: &mut hisi_rf::ws63::RadioParts) {
    let config = hisi_rf::ble::SecurityConfig::new(
        hisi_rf::ble::Bonding::Enabled,
        hisi_rf::ble::IoCapability::DisplayOnly,
        hisi_rf::ble::SecurityRequirement::SecureConnectionsAuthenticated,
    );
    let command = parts
        .ble
        .try_configure_security(config)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U5_BLE_SECURITY_QUEUE_ERR\r\n"));
    loop {
        progress(parts);
        if let Some(completion) = parts.ble.try_take_completion().unwrap() {
            if completion.id() == command && completion.into_result().is_ok() {
                return;
            }
            firmware::fail(b"RFDBG_RADIO_U5_BLE_SECURITY_ERR\r\n");
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
