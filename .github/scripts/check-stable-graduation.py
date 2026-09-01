#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Keep the U8 graduation decision aligned with public API snapshots."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REVIEW = ROOT / ".github" / "stable-graduation.toml"
BACKEND_TOKENS = ("hisi_rf_ws63", "ws63_radio_sys", "hisi_rf_rtos_driver")
UNSAFE_ALLOCATOR_TOKENS = (
    "pub unsafe fn hisi_rf::ws63::InstalledRadioStorage::allocate",
    "pub unsafe fn hisi_rf::ws63::InstalledRadioStorage::deallocate",
)
RAW_EVENT_TOKENS = (
    "BackendError::stage: u8",
    "BackendError::status: u32",
)


def presence(actual: bool) -> str:
    return "present" if actual else "absent"


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def check_snapshot(surface: dict[str, object]) -> None:
    name = str(surface["name"])
    snapshot_value = surface.get("snapshot")
    if not isinstance(snapshot_value, str):
        fail(f"{name}: non-hidden surface requires a public API snapshot")
    snapshot = ROOT / snapshot_value
    if not snapshot.is_file():
        fail(f"{name}: snapshot does not exist: {snapshot_value}")
    text = snapshot.read_text(encoding="utf-8")

    backend = presence(any(token in text for token in BACKEND_TOKENS))
    if backend != surface.get("backend_types"):
        fail(
            f"{name}: backend boundary changed: expected "
            f"{surface.get('backend_types')}, found {backend}"
        )

    unsafe_allocator = presence(all(token in text for token in UNSAFE_ALLOCATOR_TOKENS))
    if unsafe_allocator != surface.get("unsafe_allocator_hooks"):
        fail(
            f"{name}: allocator surface changed: expected "
            f"{surface.get('unsafe_allocator_hooks')}, found {unsafe_allocator}"
        )

    raw_event = presence(all(token in text for token in RAW_EVENT_TOKENS))
    if raw_event != surface.get("raw_backend_event"):
        fail(
            f"{name}: backend event shape changed: expected "
            f"{surface.get('raw_backend_event')}, found {raw_event}"
        )


def main() -> None:
    with REVIEW.open("rb") as source:
        review = tomllib.load(source)

    metadata = review.get("review")
    if not isinstance(metadata, dict) or metadata.get("schema") != 1:
        fail("stable-graduation.toml must use review schema 1")
    if metadata.get("decision") != "no-public-graduation":
        fail("changing the U8 decision requires updating this executable contract")

    surfaces = review.get("surface")
    if not isinstance(surfaces, list):
        fail("stable-graduation.toml must define surfaces")
    expected = {"wifi-station", "ble", "sle", "coexistence"}
    names = {surface.get("name") for surface in surfaces if isinstance(surface, dict)}
    if names != expected:
        fail(f"expected surfaces {sorted(expected)}, found {sorted(str(name) for name in names)}")

    for surface in surfaces:
        if not isinstance(surface, dict):
            fail("surface entries must be tables")
        name = str(surface["name"])
        decision = surface.get("decision")
        blockers = surface.get("blockers")
        if decision not in {"blocked", "hidden"}:
            fail(f"{name}: unsupported decision {decision!r}")
        if not isinstance(blockers, list) or not blockers:
            fail(f"{name}: blocked/hidden surfaces require explicit blockers")
        if name == "coexistence":
            source = (ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
            if "pub mod __coexistence" not in source or "#[doc(hidden)]" not in source:
                fail("coexistence: maintainer fixture is no longer explicitly doc-hidden")
            continue
        check_snapshot(surface)

    print("U8 stable-graduation decision matches the facade API snapshots")


if __name__ == "__main__":
    main()
