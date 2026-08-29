//! Facade-owned U2 control contract for BLE and SLE migration profiles.

#![cfg(any(
    feature = "profile-ble-peripheral",
    feature = "profile-ble-central",
    feature = "profile-ble-dual-role",
    feature = "profile-sle-announce",
    feature = "profile-sle-seek",
    feature = "profile-sle-ssap"
))]

hisi_rf::declare_radio_storage!(static RADIO_STORAGE);

fn assert_type<T>() {}

#[test]
fn facade_owns_the_radio_lifecycle() {
    let _: &'static hisi_rf::ws63::RadioStorage = &RADIO_STORAGE;
    assert_type::<hisi_rf::ws63::Resources>();
    assert_type::<hisi_rf::ws63::InstalledRadioStorage>();
    assert_type::<hisi_rf::ws63::RadioController>();
    assert_type::<hisi_rf::ws63::RadioParts>();
    assert_type::<hisi_rf::ws63::RadioRunner>();
    let _ = hisi_rf::ws63::RadioController::split;
    let _ = hisi_rf::ws63::RadioRunner::dropped_events;
    let _ = hisi_rf::ws63::RadioRunner::run_once;
    let _ = hisi_rf::ws63::RadioRunner::run_event_once;
    assert_type::<hisi_rf::ProtocolCommandId>();
    assert_type::<hisi_rf::ProtocolEventDiagnostics>();
    assert_type::<hisi_rf::ProtocolError>();
    assert_type::<hisi_rf::RadioProfile>();
    assert_type::<hisi_rf::RadioResourceReport>();

    let report = RADIO_STORAGE.report();
    assert_eq!(report.schema, 1);
    assert_eq!(report.arena_bytes, hisi_rf::ws63::RADIO_ARENA_BYTES);
    assert!(report.control_bytes > 0);
    assert_eq!(report.dynamic_task_slots, 4);
    assert_eq!(report.task_stack_bytes, 10_240);
    assert_eq!(report.minimum_task_stack_bytes, 512);
    assert_eq!(report.backend_event_capacity, 32);
    assert_eq!(report.public_event_capacity, hisi_rf::EVENT_CAPACITY);
    assert_eq!(
        report.minimum_owned_bytes(),
        report.arena_bytes + report.control_bytes
    );
}

#[cfg(any(
    feature = "profile-ble-peripheral",
    feature = "profile-ble-central",
    feature = "profile-ble-dual-role"
))]
#[test]
fn ble_profile_exposes_only_the_ble_part() {
    assert_type::<hisi_rf::ws63::BleController>();
    assert_type::<hisi_rf::ws63::BleOperation>();
    assert_type::<hisi_rf::ws63::BleOperationError>();
    assert_type::<hisi_rf::ws63::BleEvent>();
    assert_type::<hisi_rf::ws63::Advertiser>();
    assert_type::<hisi_rf::ws63::Scanner>();
    assert_type::<hisi_rf::ble::AdvertisingConfig>();
    assert_type::<hisi_rf::ble::ScanConfig>();
    #[cfg(any(feature = "profile-ble-peripheral", feature = "profile-ble-dual-role"))]
    let _ = hisi_rf::ws63::BleController::try_start_advertising;
    #[cfg(any(feature = "profile-ble-central", feature = "profile-ble-dual-role"))]
    let _ = hisi_rf::ws63::BleController::try_start_scanning;
    let _ = hisi_rf::ws63::BleController::try_take_completion;
    let _ = hisi_rf::ws63::BleController::try_next_event;
    let _ = hisi_rf::ws63::BleController::next_event;
    let _ = hisi_rf::ws63::BleController::event_diagnostics;
    let _ = hisi_rf::ws63::Advertiser::operation;
    let _ = hisi_rf::ws63::Advertiser::stop;
    let _ = hisi_rf::ws63::Scanner::operation;
    let _ = hisi_rf::ws63::Scanner::stop;
}

#[cfg(any(
    feature = "profile-sle-announce",
    feature = "profile-sle-seek",
    feature = "profile-sle-ssap"
))]
#[test]
fn sle_profile_exposes_only_the_sle_part() {
    assert_type::<hisi_rf::ws63::SleController>();
    assert_type::<hisi_rf::ws63::SleOperation>();
    assert_type::<hisi_rf::ws63::SleOperationError>();
    assert_type::<hisi_rf::ws63::SleEvent>();
    assert_type::<hisi_rf::ws63::Announcer>();
    assert_type::<hisi_rf::ws63::Seeker>();
    assert_type::<hisi_rf::sle::AnnounceConfig>();
    assert_type::<hisi_rf::sle::SeekConfig>();
    #[cfg(any(feature = "profile-sle-announce", feature = "profile-sle-ssap"))]
    let _ = hisi_rf::ws63::SleController::try_start_announce;
    #[cfg(any(feature = "profile-sle-seek", feature = "profile-sle-ssap"))]
    let _ = hisi_rf::ws63::SleController::try_start_seek;
    let _ = hisi_rf::ws63::SleController::try_take_completion;
    let _ = hisi_rf::ws63::SleController::try_next_event;
    let _ = hisi_rf::ws63::SleController::next_event;
    let _ = hisi_rf::ws63::SleController::event_diagnostics;
    let _ = hisi_rf::ws63::Announcer::operation;
    let _ = hisi_rf::ws63::Announcer::stop;
    let _ = hisi_rf::ws63::Seeker::operation;
    let _ = hisi_rf::ws63::Seeker::stop;
}
