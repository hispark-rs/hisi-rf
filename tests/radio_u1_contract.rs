//! Facade-owned U1 composition contract for BLE and SLE migration profiles.

#![cfg(any(feature = "profile-ble-dual-role", feature = "profile-sle-ssap"))]

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
}

#[cfg(feature = "profile-ble-dual-role")]
#[test]
fn ble_profile_exposes_only_the_ble_part() {
    assert_type::<hisi_rf::ws63::BleController>();
}

#[cfg(feature = "profile-sle-ssap")]
#[test]
fn sle_profile_exposes_only_the_sle_part() {
    assert_type::<hisi_rf::ws63::SleController>();
}
