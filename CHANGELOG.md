# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.17] - 2026-07-23

### Added

- Re-exported allocation-free WS63 RF heap metrics through the safe facade for
  HIL calibration without exposing the backend crate to applications.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.14`, including synchronized host tests
  and runtime RF heap usage observations.

## [0.1.0-alpha.16] - 2026-07-23

### Fixed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.13`, whose C `memalign` boundary now
  preserves checked power-of-two alignment for supplicant and DMA-capable
  allocations.

## [0.1.0-alpha.15] - 2026-07-23

### Added

- Re-exported source-aware numeric traces for vendor, IEEE 802.11, and hostap
  failures through the chip-neutral facade.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.5` and `hisi-rf-ws63` to
  `0.1.0-alpha.12`, including first-EAPOL and PMF timeout classification.

## [0.1.0-alpha.14] - 2026-07-23

### Added

- Re-exported the initialized WS63 station MAC from the safe composition root
  for standard L2/IP stack configuration.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.11`.

## [0.1.0-alpha.13] - 2026-07-23

### Added

- WS63 initialized controllers expose `start_runner()`, which starts the
  mandatory runner from caller-owned profile storage without exposing the
  runtime-driver crate to applications.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.10`; its owner-bound reservation now
  covers the public runner and the five workers observed in the pinned payload.

## [0.1.0-alpha.12] - 2026-07-23

### Added

- Re-exported stable cancellation/resource diagnostic classes and bounded
  required/available resource trace fields.
- WS63 `InitError` now provides the same allocation-free, actionable,
  secret-free diagnostic schema as runtime radio operations.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.4` and `hisi-rf-ws63` to
  `0.1.0-alpha.9`.

## [0.1.0-alpha.11] - 2026-07-22

### Added

- Added `ws63_resource_report`, so applications and CI can emit the selected
  profile's versioned resource contract through the public facade.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.8`; WS63 initialization now reserves
  the profile's dynamic task capacity before claiming storage or hardware.

## [0.1.0-alpha.10] - 2026-07-22

### Added

- Re-exported `hisi-rf-error/v2` protocol stages, fixed-capacity numeric traces,
  trace truncation state, and backend profile revisions through the facade.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.3` and `hisi-rf-ws63` to
  `0.1.0-alpha.7`, preserving raw WS63/hostap status outside UART logs.

## [0.1.0-alpha.9] - 2026-07-22

### Added

- Re-exported the allocation-free `hisi-rf-error/v1` diagnostic schema,
  including stable machine codes, operation stages, recovery actions, lossless
  backend codes, and deterministic secret-free JSON.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.2` and `hisi-rf-ws63` to
  `0.1.0-alpha.6`.

## [0.1.0-alpha.8] - 2026-07-22

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.5`, which performs a typed dynamic-task
  capacity preflight before claiming caller storage or touching radio hardware.

## [0.1.0-alpha.7] - 2026-07-22

### Added

- Re-exported profile-typed caller storage and deterministic resource reports
  through `hisi_rf::ws63`.

### Changed

- Removed the chip-neutral raw `init`, `RadioResources`, and `RadioState`
  re-exports from the application facade so WS63 applications cannot bypass
  the safe composition root.
- Updated `hisi-rf-ws63` to `0.1.0-alpha.4`.

## [0.1.0-alpha.6] - 2026-07-20

### Added

- Added `profile-wifi-wpa2-smoltcp` and `profile-wifi-wpa3-smoltcp` as the
  application-facing, complete Wi-Fi profile selections. Chip selection stays
  explicit, and no unimplemented Embassy Net profile is advertised.

### CI

- Added a crates.io-only external WS63 consumer fixture with no path dependency,
  workspace patch, consumer `build.rs`, or direct sys/blob/runtime-driver
  dependency. Linux, macOS, and Windows now perform both clean online and clean
  offline final firmware links for WPA2 and WPA3 profiles.

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

[Unreleased]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.15...HEAD
[0.1.0-alpha.15]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.15
[0.1.0-alpha.14]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.14
[0.1.0-alpha.13]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.13
[0.1.0-alpha.12]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.12
[0.1.0-alpha.11]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.10...v0.1.0-alpha.11
[0.1.0-alpha.10]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.10
[0.1.0-alpha.9]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.8...v0.1.0-alpha.9
[0.1.0-alpha.8]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.7...v0.1.0-alpha.8
[0.1.0-alpha.7]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.6...v0.1.0-alpha.7
[0.1.0-alpha.6]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.5...v0.1.0-alpha.6
[0.1.0-alpha.5]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.4...v0.1.0-alpha.5
[0.1.0-alpha.4]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.3...v0.1.0-alpha.4
[0.1.0-alpha.3]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.2...v0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.1
