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
    let interval = hisi_rf::sle::SeekInterval::try_from_units(100).unwrap();
    let config = hisi_rf::sle::SeekConfig::new(
        hisi_rf::sle::SeekTiming::try_new(interval, interval).unwrap(),
        false,
    );
    let command = parts
        .sle
        .try_start_seek(config)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U2_SLE_SEEK_QUEUE_ERR\r\n"));
    let mut accepted = false;
    let mut ready = false;
    loop {
        if parts.runner.run_once().is_err() {
            firmware::fail(b"RFDBG_RADIO_U2_SLE_SEEK_RUNNER_ERR\r\n");
        }
        if let Some(completion) = parts
            .sle
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U2_SLE_SEEK_CORRELATION_ERR\r\n"))
        {
            if completion.id() != command || completion.into_result().is_err() {
                firmware::fail(b"RFDBG_RADIO_U2_SLE_SEEK_COMMAND_ERR\r\n");
            }
            accepted = true;
        }
        while let Some(event) = parts.runner.next_hil_event() {
            match event {
                hisi_rf::ws63::U2HilEvent::SeekReady => ready = true,
                hisi_rf::ws63::U2HilEvent::PeerObserved if accepted && ready => {
                    firmware::log(b"RFDBG_RADIO_U2_SLE_SEEK_OK\r\n")
                }
                hisi_rf::ws63::U2HilEvent::BackendError { .. } => {
                    firmware::fail(b"RFDBG_RADIO_U2_SLE_SEEK_LIFECYCLE_ERR\r\n")
                }
                _ => {}
            }
        }
        if parts.runner.dropped_events() != 0 {
            firmware::fail(b"RFDBG_RADIO_U2_SLE_EVENT_DROP\r\n");
        }
        firmware::drive_scheduler();
    }
}
