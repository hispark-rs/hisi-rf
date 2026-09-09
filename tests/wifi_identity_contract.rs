use hisi_rf::{
    OperationTimeout, Passphrase, ScanResult, Security, Ssid, StationConfig, UnicastMacAddress,
    WifiL2Capabilities,
};

const ADDRESS: UnicastMacAddress = match UnicastMacAddress::try_from_bytes([2, 0, 0, 0, 0, 1]) {
    Some(address) => address,
    None => panic!("invalid fixture address"),
};

#[test]
fn facade_owns_the_typed_station_identity_contract() {
    let capabilities = WifiL2Capabilities::from_station_address(ADDRESS);
    assert_eq!(capabilities.station_address(), ADDRESS);
    assert_eq!(capabilities.station_mac_address(), [2, 0, 0, 0, 0, 1]);
    for invalid in [[0; 6], [0xff; 6], [1, 0, 0, 0, 0, 1]] {
        assert!(UnicastMacAddress::try_from_bytes(invalid).is_none());
        assert!(WifiL2Capabilities::try_new(invalid).is_none());
    }
}

#[test]
fn facade_configuration_debug_never_discloses_a_passphrase() {
    // Public synthetic test data, not network credentials.
    let bytes = b"facade-secret-fixture";
    let passphrase = Passphrase::try_from_ascii(bytes).unwrap();
    assert_eq!(format!("{passphrase:?}"), "Passphrase([REDACTED])");
    let mut scan = ScanResult::empty();
    scan.ssid = Ssid::try_from_bytes(b"facade-fixture").unwrap();
    scan.bssid = ADDRESS.into_bytes();
    scan.channel = 1;
    scan.security = Security::Wpa2Personal;
    let config = StationConfig::wpa2_personal(
        &scan,
        passphrase,
        OperationTimeout::try_from_millis(1_000).unwrap(),
    )
    .unwrap();
    let debug = format!("{config:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains(core::str::from_utf8(bytes).unwrap()));
    assert!(!debug.contains(&format!("{bytes:?}")));
    assert_eq!(config.passphrase.expose_secret(), bytes);
}

#[test]
fn facade_rejects_non_printable_and_out_of_range_passphrases() {
    for invalid in [b"short".as_slice(), b"newline\n", b"nonascii\x80"] {
        assert!(Passphrase::try_from_ascii(invalid).is_none());
    }
    assert!(Passphrase::try_from_ascii(&[b'a'; 64]).is_none());
    assert!(Passphrase::try_from_ascii(&[b'a'; 63]).is_some());
}
