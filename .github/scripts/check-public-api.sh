#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="$ROOT/.github/public-api/ws63-incremental.txt"
BLE_BASELINE="$ROOT/.github/public-api/ws63-ble-u5.txt"
SLE_BASELINE="$ROOT/.github/public-api/ws63-sle-u4.txt"
EXPECTED_VERSION="cargo-public-api 0.52.0"

if ! cargo public-api --version >/dev/null 2>&1; then
    echo "ERROR: cargo-public-api 0.52.0 is required" >&2
    echo "Install it with: cargo install cargo-public-api --version 0.52.0 --locked" >&2
    exit 1
fi

actual_version="$(cargo public-api --version)"
if [ "$actual_version" != "$EXPECTED_VERSION" ]; then
    echo "ERROR: expected $EXPECTED_VERSION, found $actual_version" >&2
    exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

generate() {
    local features="$1"
    local output="$2"
    cargo public-api -sss --color=never \
        --target x86_64-unknown-linux-gnu \
        --features "$features" \
        > "$output"
    if [ ! -s "$output" ]; then
        echo "ERROR: cargo-public-api produced an empty API snapshot" >&2
        exit 1
    fi
}

cd "$ROOT"
cargo metadata --locked --format-version 1 --no-deps >/dev/null
generate chip-ws63,profile-wifi-wpa2-smoltcp,incremental-embassy-wait "$tmp/wpa2.txt"
generate chip-ws63,profile-wifi-wpa3-smoltcp,incremental-embassy-wait "$tmp/wpa3.txt"
generate chip-ws63,profile-ble-dual-role "$tmp/ble.txt"
generate chip-ws63,profile-ble-peripheral "$tmp/ble-peripheral.txt"
generate chip-ws63,profile-ble-central "$tmp/ble-central.txt"
generate chip-ws63,profile-sle-ssap "$tmp/sle.txt"
generate chip-ws63,profile-sle-announce "$tmp/sle-announce.txt"
generate chip-ws63,profile-sle-seek "$tmp/sle-seek.txt"

if ! diff -u "$tmp/wpa2.txt" "$tmp/wpa3.txt"; then
    echo "ERROR: named WS63 security profiles expose different facade APIs" >&2
    exit 1
fi

if ! diff -u "$BASELINE" "$tmp/wpa2.txt"; then
    echo "ERROR: the hisi-rf public API changed" >&2
    echo "Review the diff; update the baseline only as part of an intentional API change." >&2
    exit 1
fi

if ! diff -u "$BLE_BASELINE" "$tmp/ble.txt"; then
    echo "ERROR: the BLE U4 facade API changed" >&2
    exit 1
fi

if ! diff -u "$SLE_BASELINE" "$tmp/sle.txt"; then
    echo "ERROR: the SLE U4 facade API changed" >&2
    exit 1
fi

require_api() {
    local output="$1"
    local pattern="$2"
    if ! grep -F "$pattern" "$output" >/dev/null; then
        echo "ERROR: expected API is missing from $(basename "$output"): $pattern" >&2
        exit 1
    fi
}

reject_api() {
    local output="$1"
    local pattern="$2"
    if grep -F "$pattern" "$output" >/dev/null; then
        echo "ERROR: role-inapplicable API leaked into $(basename "$output"): $pattern" >&2
        exit 1
    fi
}

require_api "$tmp/ble-peripheral.txt" "BleController::try_start_advertising"
reject_api "$tmp/ble-peripheral.txt" "BleController::try_start_scanning"
reject_api "$tmp/ble-peripheral.txt" "BleController::try_connect"
require_api "$tmp/ble-central.txt" "BleController::try_start_scanning"
require_api "$tmp/ble-central.txt" "BleController::try_connect"
reject_api "$tmp/ble-central.txt" "BleController::try_start_advertising"
reject_api "$tmp/ble-central.txt" "BleController::try_register_gatt_server"
require_api "$tmp/sle-announce.txt" "SleController::try_start_announce"
reject_api "$tmp/sle-announce.txt" "SleController::try_start_seek"
reject_api "$tmp/sle-announce.txt" "SleController::try_register_ssap_server"
require_api "$tmp/sle-seek.txt" "SleController::try_start_seek"
reject_api "$tmp/sle-seek.txt" "SleController::try_start_announce"
reject_api "$tmp/sle-seek.txt" "SleController::try_register_ssap_server"

if grep -E 'hisi_rf_ws63|ws63_radio_sys|hisi_rf_rtos_driver|BleB[123]|SleS[123]' \
    "$tmp/ble.txt" "$tmp/ble-peripheral.txt" "$tmp/ble-central.txt" \
    "$tmp/sle.txt" "$tmp/sle-announce.txt" "$tmp/sle-seek.txt"; then
    echo "ERROR: the public radio facade exposes a WS63 implementation type" >&2
    exit 1
fi

echo "hisi-rf public API matches the Wi-Fi, BLE, SLE, and role-profile contracts"
