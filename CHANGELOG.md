# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.5] - 2026-07-20

### Added

- Added a complete WS63 firmware example that uses only the public
  `hisi_rf::ws63` composition root; the facade CI links it on Linux, macOS, and
  Windows with stock `rust-lld`.

### Changed

- Updated the WS63 backend to `0.1.0-alpha.3`. Normalized radio archives,
  native upstream hostap, ROM/NVS fallbacks, and the relocatable ROM patch
  table are now fully transitive implementation details of `hisi-rf`.

## [0.1.0-alpha.4] - 2026-07-20

### Fixed

- Updated the WS63 backend to `0.1.0-alpha.2`, which supports the
  feature-minimal RV32 chip-selection graph.

## [0.1.0-alpha.3] - 2026-07-20

### Added

- Added the explicit `chip-ws63` composition root and safe
  `hisi_rf::ws63::{Resources, RadioController, init}` re-exports.
- Added one-way `wifi`, `smoltcp`, `wpa2-personal`, and `wpa3-personal`
  feature forwarding to the selected WS63 backend.

### Changed

- A chip must now be selected explicitly; the facade never guesses from the
  target triple or a default feature.
- WPA2-Personal and WPA3-Personal are mutually exclusive, and the current
  Personal profiles require the available smoltcp data-plane integration.

## [0.1.0-alpha.2] - 2026-07-20

### Changed

- Moved the chip-neutral implementation into `hisi-rf-core 0.1.0-alpha.1` and
  re-exported it without changing existing `hisi_rf::*` source paths.
- Reduced this crate to the application-facing facade in preparation for
  feature-selected chip composition roots.

### Added

- Typed WPA3-Personal station configuration with mandatory PMF and explicit SAE
  password-element policy.
- Explicit WPA2/WPA3-Personal transition scan classification; callers choose
  PSK or SAE instead of discovery silently downgrading to WPA2.

## [0.1.0-alpha.1] - 2026-07-14

### Added

- Chip-neutral `RadioController`, `RadioParts`, and mandatory `RadioRunner`.
- Typed Wi-Fi scan/station configuration and secret passphrase ownership.
- Bounded Wi-Fi event queue with observable overflow diagnostics.
- Separate `WifiController` and L2 `WifiDevice` ownership.
- Optional delegation to `smoltcp::phy::Device`.

[Unreleased]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.5...HEAD
[0.1.0-alpha.5]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.4...v0.1.0-alpha.5
[0.1.0-alpha.4]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.3...v0.1.0-alpha.4
[0.1.0-alpha.3]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.2...v0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.1
