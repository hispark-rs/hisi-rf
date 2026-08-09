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
    for cycle in 0..3 {
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
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U4_BLE_QUEUE_ERR\r\n"));
        let advertiser = 'wait: loop {
            progress(&mut parts);
            let invalid_completion = parts
                .ble
                .try_take_completion()
                .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U4_BLE_CORRELATION_ERR\r\n"))
                .map(|completion| completion.id() != command || completion.into_result().is_err())
                .unwrap_or(false);
            if invalid_completion {
                firmware::fail(b"RFDBG_RADIO_U4_BLE_COMMAND_ERR\r\n");
            }
            while let Some(event) = parts.ble.try_next_event() {
                match event {
                    hisi_rf::ws63::BleEvent::AdvertisingStarted { advertiser }
                        if advertiser.operation() == command =>
                    {
                        break 'wait advertiser;
                    }
                    hisi_rf::ws63::BleEvent::BackendError { .. } => {
                        firmware::fail(b"RFDBG_RADIO_U4_BLE_LIFECYCLE_ERR\r\n")
                    }
                    _ => {}
                }
            }
        };

        if cycle == 1 {
            drop(advertiser);
            for _ in 0..20 {
                progress(&mut parts);
                firmware::drive_scheduler();
            }
        } else {
            let result = firmware::wait_with_runner(advertiser.stop(), || progress(&mut parts));
            if result.is_err() {
                firmware::fail(b"RFDBG_RADIO_U4_BLE_STOP_ERR\r\n");
            }
        }
    }
    firmware::log(b"RFDBG_RADIO_U4_BLE_ADV_LIFECYCLE_OK\r\n");
    loop {
        progress(&mut parts);
        firmware::drive_scheduler();
    }
}

fn progress(parts: &mut hisi_rf::ws63::RadioParts) {
    if parts.runner.run_once().is_err() {
        firmware::fail(b"RFDBG_RADIO_U4_BLE_RUNNER_ERR\r\n");
    }
    while parts.runner.run_event_once() {}
    if parts.runner.dropped_events() != 0 || parts.ble.event_diagnostics().dropped != 0 {
        firmware::fail(b"RFDBG_RADIO_U4_BLE_EVENT_DROP\r\n");
    }
    firmware::drive_scheduler();
}
