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
generate chip-ws63,profile-sle-ssap "$tmp/sle.txt"

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

if grep -E 'hisi_rf_ws63|ws63_radio_sys|hisi_rf_rtos_driver|BleB[123]|SleS[123]' \
    "$tmp/ble.txt" "$tmp/sle.txt"; then
    echo "ERROR: the public radio facade exposes a WS63 implementation type" >&2
    exit 1
fi

echo "hisi-rf public API matches the Wi-Fi, BLE U5, and SLE U4 facade baselines"
