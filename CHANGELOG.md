# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Advanced the unified aggregate diagnostic schema to
  `hisi-rf-radio-diagnostics/v4`. The data-path member now covers smoltcp,
  vendor bridge, DMAC completion, host RX, MAC receive-engine, and radio IRQ
  progress while remaining allocation-free and secret-redacted.

## [0.1.0-alpha.57] - 2026-07-29

### Fixed

- Made the incremental unified diagnostic snapshot readable from
  `WifiParts::diagnostics()` after the runner and Wi-Fi handles have been moved
  into separate executor tasks. Applications no longer need a global
  `Mutex<Cell<Option<_>>>` to retain runner/wait counters.

### Changed

- Updated to `hisi-rf-core 0.1.0-alpha.19` and `hisi-rf-ws63
  0.1.0-alpha.47` for instance-owned runner and non-consuming wait-bridge
  snapshots.
- Advanced the aggregate diagnostic schema to
  `hisi-rf-radio-diagnostics/v2`; the new `control` member preserves
  command-channel and blocking-to-incremental migration counters after the
  runner has moved into its executor task.

## [0.1.0-alpha.56] - 2026-07-29

### Added

- Added `RadioDiagnosticsSnapshot` as the facade-owned, allocation-free,
  secret-redacted view of runner, wait bridge, event queue, blocking backend,
  L2/DHCP, and resource metrics.
- Added `WifiParts::diagnostics()` for the blocking profile and
  `IncrementalRadioParts::diagnostics()` for the bounded incremental profile,
  eliminating application-side assembly through process-global mutable state.

### Changed

- Updated to `hisi-rf-ws63 0.1.0-alpha.46` for storage-independent resource
  metadata. The aggregate schema references `hisi-rf-error/v3` and
  `hisi-rf-resource-report/v6` rather than copying their classifications.

## [0.1.0-alpha.55] - 2026-07-29

### Changed

- Updated to `hisi-rf-core 0.1.0-alpha.18` and `hisi-rf-ws63
  0.1.0-alpha.45`.
- Station MAC and immutable L2 capability access now belongs to the selected
  profile's `WifiDevice`; the WS63 facade no longer exposes a process-global
  station MAC function.

## [0.1.0-alpha.54] - 2026-07-29

### Added

- Re-exported the typed `OperationTimeout` and `BackendTimeout` contracts.
  Applications can now keep protocol deadlines, backend lifecycle bounds, and
  their own outer wait deadlines distinct at the type level.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.17` and `hisi-rf-ws63` to
  `0.1.0-alpha.44`, including dropped-future cancellation and the
  `hisi-rf-error/v3` operation/backend timeout split.

## [0.1.0-alpha.53] - 2026-07-29

### Changed

- Removed event-capacity const generics from the facade's common controller,
  parts, runner, and WS63 initialization signatures. The selected profile owns
  an eight-event queue through caller-owned storage and its resource report;
  backend maintainers use the independently versioned chip backend for generic
  capacities.
- Migrated the crates.io-only external consumer to the published
  `hisi-rf 0.1.0-alpha.52` and the single `declare_radio_storage!`
  composition contract.

## [0.1.0-alpha.52] - 2026-07-29

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.43`, whose host-generated resource
  report now models the actual WS63 RV32 storage layout and aligned arena
  backing bytes.

## [0.1.0-alpha.51] - 2026-07-29

### Added

- Re-exported the WS63 `RadioStorage`, `InstalledRadioStorage`, and
  `declare_radio_storage!` composition contract. Applications now declare and
  admit one caller-owned radio storage object while the backend preserves the
  correct BSS/NOLOAD split internally.

### Changed

- Moved the crates.io-only consumer's security choice into explicit local
  WPA2/WPA3 profile features, migrated it to the profile-aware resource
  builder, and made deprecated API use a compile error in that fixture.
- Updated `hisi-rf-ws63` to `0.1.0-alpha.42` and exposed its deterministic
  `hisi-rf-resource-report/v6` accounting.

## [0.1.0-alpha.50] - 2026-07-29

### Changed

- Made `hisi_rf::ws63::init` the stable composition-root name for both backend
  selections. With `incremental-backend-experiment`, it returns the bounded
  incremental controller; without that explicit feature it retains the
  validated blocking controller.
- Updated `hisi-rf-ws63` to `0.1.0-alpha.41`. The implementation-specific
  `init_incremental_after_blocking_bootstrap` alias remains deprecated for one
  alpha migration cycle.

## [0.1.0-alpha.49] - 2026-07-29

### Added

- Re-exported the profile-aware WS63 typestate resource builder. WPA2 callers
  no longer surrender an unused PKE token, while WPA3 cannot construct its
  resources without one.
- Added a pinned `cargo-public-api` snapshot gate for the complete WS63
  incremental facade. WPA2 and WPA3 named profiles must expose the same public
  API, and every intentional API change now produces a reviewable CI diff.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.40`.

## [0.1.0-alpha.48] - 2026-07-28

### Added

- Re-exported structured association ioctl timing diagnostics through the WS63
  facade for application-level response-bound evidence.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.38`.

## [0.1.0-alpha.47] - 2026-07-28

### Added

- Re-exported counter-only DHCP and receive-queue diagnostics through the WS63
  facade. Final connectivity applications can attribute packet loss while
  keeping `hisi-rf-ws63` and its netif implementation transitive.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.37`.

## [0.1.0-alpha.46] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.36`, carrying credential-free
  cancellation and timeout injection through the public controller, facade
  channels, incremental runner, and WS63 backend. QEMU and real WS63 now prove
  the same `operation.cancelled` and connect-stage `backend.timeout` output.

## [0.1.0-alpha.45] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.35`, carrying the real upstream
  supplicant `NEW_KEY`/`SET_KEY`/`DEL_KEY` lifecycle and fail-closed rollback
  evidence through the public facade release train.

## [0.1.0-alpha.44] - 2026-07-28

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.16` and `hisi-rf-ws63` to
  `0.1.0-alpha.34`. Incremental operation start, cancellation, and WS63 vendor
  actions now advance only through bounded poll turns.

## [0.1.0-alpha.43] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.33`. The WS63 composition API now
  exposes only facade-owned Wi-Fi/device/token types, an opaque initialization
  error, and standard smoltcp contracts; runtime-driver and concrete backend
  types no longer appear in public signatures.
- Kept runtime selection with the application: enabling the incremental
  Embassy wait bridge no longer selects `hisi-rtos` through the backend's
  normal dependency graph.

### Added

- Extended dependency-boundary CI to resolve the incremental Embassy feature
  and reject any concrete runtime selected by the facade graph.

## [0.1.0-alpha.42] - 2026-07-28

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.15` and `hisi-rf-ws63` to
  `0.1.0-alpha.32`, carrying the incremental local-continuation fix and the
  adversarial cancellation resource-conservation coverage into the public
  single-dependency facade.

## [0.1.0-alpha.41] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.31`, carrying production
  incremental-driver evidence for terminal cancellation, backend timeout, and
  operation-slot recovery on QEMU and real WS63 silicon.

## [0.1.0-alpha.40] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.30`, carrying target parity evidence
  for the production association-rejection and first-EAPOL-timeout diagnostic
  builders.

## [0.1.0-alpha.39] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.29`, carrying the credential-free
  cancellation and backend-timeout target parity fixture.

## [0.1.0-alpha.38] - 2026-07-28

### Added

- Updated `hisi-rf-ws63` to `0.1.0-alpha.28`, exposing actionable
  `hisi-rf-error/v2` diagnostics for caller-owned arena admission failures
  through the public facade.

## [0.1.0-alpha.37] - 2026-07-28

### Added

- Re-exported the caller-owned WS63 radio arena, one-shot claim/install
  capability, selected-profile arena size, and declaration macro through the
  chip-selecting facade.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.27` and the firmware fixture to
  `hisi-riscv-rt 0.5.7`. The fixture now reserves its 296 KiB RF arena in the
  runtime-owned shared-arena linker region and uses the fixed 32 KiB radio main
  stack profile.

## [0.1.0-alpha.36] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.26`,
  `hisi-rf-rtos-driver` to `0.1.0-alpha.17`, and `hisi-rtos` to
  `0.1.0-alpha.13`, carrying the A5R resource-lifecycle conformance fixes and
  the latest WS63 diagnostics.

## [0.1.0-alpha.35] - 2026-07-28

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.25`, carrying the bounded
  transition-mode recovery diagnostics and the 100 ms incremental-runner
  work-budget evidence while keeping the blocking backend as the default.
- Mirrored the backend's explicit `incremental-embassy-wait` capability instead
  of exposing its executor/time-dependent runner types from the bounded
  backend feature alone.

## [0.1.0-alpha.34] - 2026-07-26

### Added

- Re-exported the opt-in incremental runner and WS63 wait-bridge diagnostics
  through the single-dependency facade. Applications can now capture bounded
  work, wake-source, callback/L2 signal, timer, terminal, and error counters
  without importing the chip backend or runtime-driver crates.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.14` and `hisi-rf-ws63` to
  `0.1.0-alpha.24`. The validated blocking backend remains the default.

## [0.1.0-alpha.33] - 2026-07-26

### Fixed

- Switched to `hisi-rf-ws63 0.1.0-alpha.23`, which embeds mask-ROM fallback
  addresses as transitive ELF symbols. Removed the ineffective facade
  build-script relay: Cargo linker arguments apply to the package that emits
  them and cannot be forwarded through a library dependency to an application.

## [0.1.0-alpha.32] - 2026-07-26

### Fixed

- Relayed the WS63 mask-ROM fallback linker script exported by
  `hisi-rf-ws63 0.1.0-alpha.22`, preserving the complete native link contract
  when applications depend only on the user-facing facade.

## [0.1.0-alpha.31] - 2026-07-26

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.21`. The opt-in incremental WS63
  runner now owns its callback, L2 receive, and monotonic timer wait platform;
  applications no longer construct a chip-specific wait adapter.
- Kept the blocking backend as the default while the incremental backend
  remains an explicitly selected experiment.

## [0.1.0-alpha.30] - 2026-07-23

### Added

- Re-exported the hidden WS63 bootstrap-stage identifiers and metrics through
  the safe composition root so A5B can measure each blocking prerequisite
  without exposing raw sys/blob crates.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.19`. Stage boundaries are diagnostic
  observations only; vendor bootstrap remains blocking and the default backend
  is unchanged.

## [0.1.0-alpha.29] - 2026-07-23

### Added

- Forwarded the non-default WS63 incremental experiment through the facade and
  exposed its explicitly named blocking-bootstrap/owned-runner lifecycle from
  `hisi_rf::ws63`.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.18`. The default blocking profile is
  unchanged; callers must opt in to the experimental feature and drive its
  bounded runner with an explicit work budget and wait platform.

## [0.1.0-alpha.28] - 2026-07-23

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.17`, including the non-default,
  bounded scan/connect/disconnect implementation and exact v9 poll accounting.
  The WS63 implementation remains internal until its blocking initialization
  prerequisite has a coherent facade lifecycle; the default backend is unchanged.

## [0.1.0-alpha.27] - 2026-07-23

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.16`, consuming the versioned v9
  supplicant poll ABI and exact bounded-work accounting. The partial WS63
  incremental adapter remains internal and non-default until initialize and
  scan are genuinely incremental.

## [0.1.0-alpha.26] - 2026-07-23

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.13`, exposing current and high-water
  occupancy for the fixed-capacity control command channel.

## [0.1.0-alpha.25] - 2026-07-23

### Added

- Re-exported secret-free WS63 blocking backend operation, sleep, and native
  supplicant poll metrics through the safe composition root.

### Changed

- Updated `hisi-rf-ws63` to `0.1.0-alpha.15`.

## [0.1.0-alpha.24] - 2026-07-23

### Added

- Re-exported blocking-runner work counters and event-queue high-water
  diagnostics for A5B migration measurements.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.12`.

## [0.1.0-alpha.23] - 2026-07-23

### Added

- Re-exported the executor-neutral incremental wait-platform contract and its
  typed fail-closed error so applications can compose command, backend, L2,
  and timer wakes without depending on `hisi-rf-core` directly.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.11`.

## [0.1.0-alpha.22] - 2026-07-23

### Added

- Re-exported the incremental platform wait-intent snapshot so executors can
  compose command, backend, L2, and timer wakes without depending on
  `hisi-rf-core` directly.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.10`.

## [0.1.0-alpha.21] - 2026-07-23

### Added

- Re-exported the opt-in incremental async-facade runner and split result while
  preserving the existing `WifiController`, L2 device, and bounded event API.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.9`; facade command intake now applies
  bounded backpressure while an active plus pending replacement are retained.

## [0.1.0-alpha.20] - 2026-07-23

### Added

- Re-exported the opt-in executable incremental backend driver and bounded
  active/pending command arbiter. The blocking WS63 backend remains the default.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.8`.

## [0.1.0-alpha.19] - 2026-07-23

### Added

- Re-exported the opt-in deterministic incremental runner state, fair wake
  selector, cancellation directives, and bounded transition results.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.7`. The blocking WS63 backend remains
  the default and does not implement the experimental runner yet.

## [0.1.0-alpha.18] - 2026-07-23

### Added

- Added the opt-in `incremental-backend-experiment` facade feature, forwarding
  the bounded A5B operation protocol without changing the default WS63 backend.

### Changed

- Updated `hisi-rf-core` to `0.1.0-alpha.6`.

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

[Unreleased]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.55...HEAD
[0.1.0-alpha.55]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.54...v0.1.0-alpha.55
[0.1.0-alpha.54]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.53...v0.1.0-alpha.54
[0.1.0-alpha.53]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.52...v0.1.0-alpha.53
[0.1.0-alpha.52]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.51...v0.1.0-alpha.52
[0.1.0-alpha.51]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.50...v0.1.0-alpha.51
[0.1.0-alpha.50]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.49...v0.1.0-alpha.50
[0.1.0-alpha.49]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.48...v0.1.0-alpha.49
[0.1.0-alpha.48]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.47...v0.1.0-alpha.48
[0.1.0-alpha.47]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.46...v0.1.0-alpha.47
[0.1.0-alpha.46]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.45...v0.1.0-alpha.46
[0.1.0-alpha.45]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.44...v0.1.0-alpha.45
[0.1.0-alpha.44]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.43...v0.1.0-alpha.44
[0.1.0-alpha.43]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.42...v0.1.0-alpha.43
[0.1.0-alpha.42]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.41...v0.1.0-alpha.42
[0.1.0-alpha.41]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.40...v0.1.0-alpha.41
[0.1.0-alpha.40]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.39...v0.1.0-alpha.40
[0.1.0-alpha.39]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.38...v0.1.0-alpha.39
[0.1.0-alpha.38]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.37...v0.1.0-alpha.38
[0.1.0-alpha.37]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.36...v0.1.0-alpha.37
[0.1.0-alpha.36]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.35...v0.1.0-alpha.36
[0.1.0-alpha.35]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.34...v0.1.0-alpha.35
[0.1.0-alpha.34]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.33...v0.1.0-alpha.34
[0.1.0-alpha.33]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.32...v0.1.0-alpha.33
[0.1.0-alpha.32]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.31...v0.1.0-alpha.32
[0.1.0-alpha.31]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.30...v0.1.0-alpha.31
[0.1.0-alpha.30]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.29...v0.1.0-alpha.30
[0.1.0-alpha.29]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.28...v0.1.0-alpha.29
[0.1.0-alpha.28]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.27...v0.1.0-alpha.28
[0.1.0-alpha.27]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.26...v0.1.0-alpha.27
[0.1.0-alpha.26]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.25...v0.1.0-alpha.26
[0.1.0-alpha.25]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.24...v0.1.0-alpha.25
[0.1.0-alpha.24]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.23...v0.1.0-alpha.24
[0.1.0-alpha.23]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.22...v0.1.0-alpha.23
[0.1.0-alpha.22]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.21...v0.1.0-alpha.22
[0.1.0-alpha.21]: https://github.com/hispark-rs/hisi-rf/compare/v0.1.0-alpha.20...v0.1.0-alpha.21
[0.1.0-alpha.20]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.20
[0.1.0-alpha.19]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.19
[0.1.0-alpha.18]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.18
[0.1.0-alpha.17]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.17
[0.1.0-alpha.16]: https://github.com/hispark-rs/hisi-rf/releases/tag/v0.1.0-alpha.16
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
