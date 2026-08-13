#[test]
fn restored_pairing_state_is_queried_only_after_authentication() {
    let source = include_str!("../examples/ws63_ble_u5_bond_central.rs");
    let connected = source
        .find("BleEvent::Connected")
        .expect("U5 central must handle connection events");
    let secure = source[connected..]
        .find("try_pair")
        .map(|offset| connected + offset)
        .expect("every U5 connection must initiate its security procedure");
    let authenticated = source
        .find("BleEvent::AuthenticationComplete")
        .expect("U5 central must handle authentication events");
    let query = source
        .find("try_query_pairing_state")
        .expect("U5 central must verify the restored pairing state");

    assert!(connected < authenticated);
    assert!(secure < authenticated);
    assert!(authenticated < query);
}
