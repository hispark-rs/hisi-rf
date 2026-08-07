#![no_std]
#![no_main]

use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

#[path = "support/ws63_u2_firmware.rs"]
mod firmware;

const DESCRIPTOR: hisi_rf::sle::SsapDescriptorDefinition =
    hisi_rf::sle::SsapDescriptorDefinition::try_new(
        hisi_rf::sle::SsapUuid::Uuid16(0x600d),
        hisi_rf::sle::SsapPermissions::READ.union(hisi_rf::sle::SsapPermissions::WRITE),
        b"U3 descriptor",
        32,
    )
    .unwrap();
const PROPERTY: hisi_rf::sle::SsapPropertyDefinition =
    hisi_rf::sle::SsapPropertyDefinition::try_new(
        hisi_rf::sle::SsapUuid::Uuid16(0x600c),
        hisi_rf::sle::SsapPermissions::READ.union(hisi_rf::sle::SsapPermissions::WRITE),
        hisi_rf::sle::SsapOperations::READ
            .union(hisi_rf::sle::SsapOperations::WRITE)
            .union(hisi_rf::sle::SsapOperations::NOTIFY),
        b"U3",
        32,
        &[DESCRIPTOR],
    )
    .unwrap();
const SERVICE: hisi_rf::sle::SsapServiceDefinition = hisi_rf::sle::SsapServiceDefinition::try_new(
    hisi_rf::sle::SsapUuid::Uuid16(0x600b),
    &[PROPERTY],
)
.unwrap();
const DATABASE: hisi_rf::sle::SsapServerDefinition =
    hisi_rf::sle::SsapServerDefinition::try_new(hisi_rf::sle::SsapUuid::Uuid16(0x600a), &[SERVICE])
        .unwrap();

#[entry]
fn main() -> ! {
    firmware::run(run)
}

fn run(mut parts: hisi_rf::ws63::RadioParts) -> ! {
    let registration = parts
        .sle
        .try_register_ssap_server(DATABASE)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_SLE_SSAP_QUEUE_ERR\r\n"));
    loop {
        if parts.runner.run_once().is_err() {
            firmware::fail(b"RFDBG_RADIO_U3_SLE_SSAP_RUNNER_ERR\r\n");
        }
        if let Some(completion) = parts
            .sle
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_SLE_SSAP_CORRELATION_ERR\r\n"))
        {
            if completion.id() != registration
                || !matches!(
                    completion.into_result(),
                    Ok(hisi_rf::ws63::SleOperation::SsapServerRegistered(_))
                )
            {
                firmware::fail(b"RFDBG_RADIO_U3_SLE_SSAP_COMMAND_ERR\r\n");
            }
            break;
        }
        firmware::drive_scheduler();
    }
    firmware::log(b"RFDBG_RADIO_U3_SLE_SSAP_REGISTERED\r\n");

    let interval = hisi_rf::sle::AnnounceInterval::try_from_units(0xc8).unwrap();
    let config = hisi_rf::sle::AnnounceConfig::new(
        hisi_rf::sle::AnnounceTiming::try_new(interval, interval).unwrap(),
        hisi_rf::sle::AnnounceChannels::ALL,
        hisi_rf::sle::AnnouncePayload::try_from_slice(hisi_rf::ws63::U2_HIL_PEER_PAYLOAD).unwrap(),
        hisi_rf::sle::AnnouncePayload::try_from_slice(b"HISI-U3-SLE").unwrap(),
    );
    let command = parts
        .sle
        .try_start_announce(config)
        .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_SLE_ANNOUNCE_QUEUE_ERR\r\n"));
    let mut accepted = false;
    let mut started = false;
    loop {
        if parts.runner.run_once().is_err() {
            firmware::fail(b"RFDBG_RADIO_U3_SLE_ANNOUNCE_RUNNER_ERR\r\n");
        }
        if let Some(completion) = parts
            .sle
            .try_take_completion()
            .unwrap_or_else(|_| firmware::fail(b"RFDBG_RADIO_U3_SLE_ANNOUNCE_CORRELATION_ERR\r\n"))
        {
            if completion.id() != command || completion.into_result().is_err() {
                firmware::fail(b"RFDBG_RADIO_U3_SLE_ANNOUNCE_COMMAND_ERR\r\n");
            }
            accepted = true;
        }
        while let Some(event) = parts.runner.next_hil_event() {
            match event {
                hisi_rf::ws63::U2HilEvent::AnnounceStarted => started = true,
                hisi_rf::ws63::U2HilEvent::BackendError { .. } => {
                    firmware::fail(b"RFDBG_RADIO_U3_SLE_LIFECYCLE_ERR\r\n")
                }
                _ => {}
            }
        }
        if accepted && started {
            firmware::log(b"RFDBG_RADIO_U3_SLE_SSAP_OK\r\n");
            accepted = false;
            started = false;
        }
        if parts.runner.dropped_events() != 0 {
            firmware::fail(b"RFDBG_RADIO_U3_SLE_EVENT_DROP\r\n");
        }
        firmware::drive_scheduler();
    }
}
