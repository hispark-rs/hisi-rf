#![no_std]
#![no_main]

use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

#[path = "support/ws63_u2_firmware.rs"]
mod firmware;

const CCC: hisi_rf::ble::GattDescriptorDefinition =
    hisi_rf::ble::GattDescriptorDefinition::try_new(
        hisi_rf::ble::GattUuid::Uuid16(0x2902),
        hisi_rf::ble::GattPermissions::READ.union(hisi_rf::ble::GattPermissions::WRITE),
        &[0, 0],
        2,
    )
    .unwrap();
const CHARACTERISTIC: hisi_rf::ble::GattCharacteristicDefinition =
    hisi_rf::ble::GattCharacteristicDefinition::try_new(
        hisi_rf::ble::GattUuid::Uuid16(0xcdef),
        hisi_rf::ble::GattPermissions::READ.union(hisi_rf::ble::GattPermissions::WRITE),
        hisi_rf::ble::GattProperties::READ
            .union(hisi_rf::ble::GattProperties::WRITE)
            .union(hisi_rf::ble::GattProperties::NOTIFY)
            .union(hisi_rf::ble::GattProperties::INDICATE),
        b"U3",
        32,
        &[CCC],
    )
    .unwrap();
const SERVICE: hisi_rf::ble::GattServiceDefinition = hisi_rf::ble::GattServiceDefinition::try_new(
    hisi_rf::ble::GattUuid::Uuid16(0xabcd),
    true,
    &[CHARACTERISTIC],
)
.unwrap();
const DATABASE: hisi_rf::ble::GattServerDefinition =
    hisi_rf::ble::GattServerDefinition::try_new(hisi_rf::ble::GattUuid::Uuid16(0xb301), &[SERVICE])
        .unwrap();

#[entry]
fn main() -> ! {
    firmware::run(run)
}

fn run(mut parts: hisi_rf::ws63::RadioParts) -> ! {
    let registration = parts
        .ble
        .try_register_gatt_server(DATABASE)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_BLE_GATT_QUEUE_ERR\r\n"));
    loop {
        if parts.runner.run_once().is_err() {
            firmware::fail(b"RFDBG_RADIO_U3_BLE_GATT_RUNNER_ERR\r\n");
        }
        if let Some(completion) = parts
            .ble
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_BLE_GATT_CORRELATION_ERR\r\n"))
        {
            if completion.id() != registration
                || !matches!(
                    completion.into_result(),
                    Ok(hisi_rf::ws63::BleOperation::GattServerRegistered(_))
                )
            {
                firmware::fail(b"RFDBG_RADIO_U3_BLE_GATT_COMMAND_ERR\r\n");
            }
            break;
        }
        firmware::drive_scheduler();
    }
    firmware::log(b"RFDBG_RADIO_U3_BLE_GATT_REGISTERED\r\n");

    let interval = hisi_rf::ble::AdvertisingInterval::try_from_units(0x20).unwrap();
    let config = hisi_rf::ble::AdvertisingConfig::new(
        hisi_rf::ble::AdvertisingTiming::try_new(interval, interval).unwrap(),
        hisi_rf::ble::AdvertisingChannels::ALL,
        hisi_rf::ble::AdvertisingPayload::try_from_slice(hisi_rf::ws63::U2_HIL_PEER_PAYLOAD)
            .unwrap(),
    );
    let command = parts
        .ble
        .try_start_advertising(config)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_BLE_ADV_QUEUE_ERR\r\n"));
    let mut accepted = false;
    let mut started = false;
    loop {
        if parts.runner.run_once().is_err() {
            firmware::fail(b"RFDBG_RADIO_U3_BLE_ADV_RUNNER_ERR\r\n");
        }
        if let Some(completion) = parts
            .ble
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_BLE_ADV_CORRELATION_ERR\r\n"))
        {
            if completion.id() != command || completion.into_result().is_err() {
                firmware::fail(b"RFDBG_RADIO_U3_BLE_ADV_COMMAND_ERR\r\n");
            }
            accepted = true;
        }
        while let Some(event) = parts.runner.next_hil_event() {
            match event {
                hisi_rf::ws63::U2HilEvent::AdvertisingStarted => started = true,
                hisi_rf::ws63::U2HilEvent::BackendError { .. } => {
                    firmware::fail(b"RFDBG_RADIO_U3_BLE_LIFECYCLE_ERR\r\n")
                }
                _ => {}
            }
        }
        if accepted && started {
            firmware::log(b"RFDBG_RADIO_U3_BLE_GATT_OK\r\n");
            accepted = false;
            started = false;
        }
        if parts.runner.dropped_events() != 0 {
            firmware::fail(b"RFDBG_RADIO_U3_BLE_EVENT_DROP\r\n");
        }
        firmware::drive_scheduler();
    }
}
