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
    let query = source[paired..]
        .find("try_query_pairing_state")
        .map(|offset| paired + offset)
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

#[test]
fn removal_fixture_removes_then_queries_not_paired_on_both_peers() {
    for (role, source) in [
        (
            "central",
            include_str!("../examples/ws63_ble_u5_bond_central.rs"),
        ),
        (
            "peripheral",
            include_str!("../examples/ws63_ble_u5_bond_peripheral.rs"),
        ),
    ] {
        let remove_request = source
            .find("try_remove_bond")
            .unwrap_or_else(|| panic!("{role} must remove its local bond"));
        let remove_completion = source
            .find("remove_command == Some(id)")
            .unwrap_or_else(|| panic!("{role} must handle bond-removal completion"));
        let query_after_remove = source[remove_completion..]
            .find("try_query_pairing_state")
            .map(|offset| remove_completion + offset)
            .unwrap_or_else(|| panic!("{role} must query state after removal"));
        let not_paired = source[query_after_remove..]
            .find("PairingState::NotPaired")
            .unwrap_or_else(|| panic!("{role} must require NotPaired after removal"));

        assert!(remove_request > 0);
        assert!(remove_completion < query_after_remove);
        assert!(not_paired > 0);
        assert!(source.contains("BOND_REMOVE_OK"));
    }
}
