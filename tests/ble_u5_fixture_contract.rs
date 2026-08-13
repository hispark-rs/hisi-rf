#[test]
fn restored_pairing_state_is_queried_only_after_security_completion() {
    let source = include_str!("../examples/ws63_ble_u5_bond_central.rs");
    let connected = source
        .find("BleEvent::Connected")
        .expect("U5 central must handle connection events");
    let secure = source[connected..]
        .find("try_pair")
        .map(|offset| connected + offset)
        .expect("every U5 connection must initiate its security procedure");
    let paired = source
        .find("BleEvent::PairingComplete")
        .expect("U5 central must handle pairing completion");
    let query = source
        .find("try_query_pairing_state")
        .expect("U5 central must verify the restored pairing state");

    assert!(connected < paired);
    assert!(secure < paired);
    assert!(paired < query);
}

#[test]
fn restored_peripheral_requires_current_connection_security_completion() {
    let source = include_str!("../examples/ws63_ble_u5_bond_peripheral.rs");
    assert!(source.contains("let mut security_completed = false"));
    assert!(source.contains("&& security_completed"));
    assert!(source.contains("security_completed = true"));
}
