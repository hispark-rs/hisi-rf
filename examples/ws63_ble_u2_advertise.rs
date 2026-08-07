#![no_std]
#![no_main]

use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

#[path = "support/ws63_u2_firmware.rs"]
mod firmware;

#[entry]
fn main() -> ! {
    firmware::run(run)
}

fn run(mut parts: hisi_rf::ws63::RadioParts) -> ! {
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
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U2_BLE_ADV_QUEUE_ERR\r\n"));
    let mut accepted = false;
    let mut started = false;
    loop {
        if parts.runner.run_once().is_err() {
            firmware::fail(b"RFDBG_RADIO_U2_BLE_ADV_RUNNER_ERR\r\n");
        }
        if let Some(completion) = parts
            .ble
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U2_BLE_ADV_CORRELATION_ERR\r\n"))
        {
            if completion.id() != command || completion.into_result().is_err() {
                firmware::fail(b"RFDBG_RADIO_U2_BLE_ADV_COMMAND_ERR\r\n");
            }
            accepted = true;
        }
        while let Some(event) = parts.runner.next_hil_event() {
            match event {
                hisi_rf::ws63::U2HilEvent::AdvertisingStarted => started = true,
                hisi_rf::ws63::U2HilEvent::BackendError { .. } => {
                    firmware::fail(b"RFDBG_RADIO_U2_BLE_ADV_LIFECYCLE_ERR\r\n")
                }
                _ => {}
            }
        }
        if accepted && started {
            firmware::log(b"RFDBG_RADIO_U2_BLE_ADV_OK\r\n");
            accepted = false;
            started = false;
        }
        if parts.runner.dropped_events() != 0 {
            firmware::fail(b"RFDBG_RADIO_U2_BLE_EVENT_DROP\r\n");
        }
        firmware::drive_scheduler();
    }
}
