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
        let interval = hisi_rf::sle::AnnounceInterval::try_from_units(0xc8).unwrap();
        let config = hisi_rf::sle::AnnounceConfig::new(
            hisi_rf::sle::AnnounceTiming::try_new(interval, interval).unwrap(),
            hisi_rf::sle::AnnounceChannels::ALL,
            hisi_rf::sle::AnnouncePayload::try_from_slice(hisi_rf::ws63::U2_HIL_PEER_PAYLOAD)
                .unwrap(),
            hisi_rf::sle::AnnouncePayload::try_from_slice(b"HISI-U4-SLE").unwrap(),
        );
        let command = parts
            .sle
            .try_start_announce(config)
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U4_SLE_QUEUE_ERR\r\n"));
        let announcer = 'wait: loop {
            progress(&mut parts);
            let invalid_completion = parts
                .sle
                .try_take_completion()
                .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U4_SLE_CORRELATION_ERR\r\n"))
                .map(|completion| completion.id() != command || completion.into_result().is_err())
                .unwrap_or(false);
            if invalid_completion {
                firmware::fail(b"RFDBG_RADIO_U4_SLE_COMMAND_ERR\r\n");
            }
            while let Some(event) = parts.sle.try_next_event() {
                match event {
                    hisi_rf::ws63::SleEvent::AnnounceStarted { announcer }
                        if announcer.operation() == command =>
                    {
                        break 'wait announcer;
                    }
                    hisi_rf::ws63::SleEvent::BackendError { .. } => {
                        firmware::fail(b"RFDBG_RADIO_U4_SLE_LIFECYCLE_ERR\r\n")
                    }
                    _ => {}
                }
            }
        };

        if cycle == 1 {
            drop(announcer);
            for _ in 0..20 {
                progress(&mut parts);
                firmware::drive_scheduler();
            }
        } else {
            let result = firmware::wait_with_runner(announcer.stop(), || progress(&mut parts));
            if result.is_err() {
                firmware::fail(b"RFDBG_RADIO_U4_SLE_STOP_ERR\r\n");
            }
        }
    }
    firmware::log(b"RFDBG_RADIO_U4_SLE_ANNOUNCE_LIFECYCLE_OK\r\n");
    loop {
        progress(&mut parts);
        firmware::drive_scheduler();
    }
}

fn progress(parts: &mut hisi_rf::ws63::RadioParts) {
    if parts.runner.run_once().is_err() {
        firmware::fail(b"RFDBG_RADIO_U4_SLE_RUNNER_ERR\r\n");
    }
    while parts.runner.run_event_once() {}
    if parts.runner.dropped_events() != 0 || parts.sle.event_diagnostics().dropped != 0 {
        firmware::fail(b"RFDBG_RADIO_U4_SLE_EVENT_DROP\r\n");
    }
}
