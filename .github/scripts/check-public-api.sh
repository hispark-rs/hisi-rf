#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="$ROOT/.github/public-api/ws63-incremental.txt"
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

host="$(rustc -vV | sed -n 's/^host: //p')"
if [ -z "$host" ]; then
    echo "ERROR: rustc did not report a host target" >&2
    exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

generate() {
    local profile="$1"
    local output="$2"
    cargo public-api -sss --color=never \
        --target "$host" \
        --features "chip-ws63,$profile,incremental-embassy-wait" \
        > "$output"
}

cd "$ROOT"
cargo metadata --locked --format-version 1 --no-deps >/dev/null
generate profile-wifi-wpa2-smoltcp "$tmp/wpa2.txt"
generate profile-wifi-wpa3-smoltcp "$tmp/wpa3.txt"

if ! diff -u "$tmp/wpa2.txt" "$tmp/wpa3.txt"; then
    echo "ERROR: named WS63 security profiles expose different facade APIs" >&2
    exit 1
fi

if ! diff -u "$BASELINE" "$tmp/wpa2.txt"; then
    echo "ERROR: the hisi-rf public API changed" >&2
    echo "Review the diff; update the baseline only as part of an intentional API change." >&2
    exit 1
fi

echo "hisi-rf public API matches the WS63 incremental facade baseline"
