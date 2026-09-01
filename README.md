# hisi-rf

`hisi-rf` is the application-facing radio facade for the hispark-rs ecosystem.
It re-exports the chip-neutral controller, runner, configuration, event, and L2
contracts from `hisi-rf-core`, then selects a safe chip composition root through
an explicit `chip-*` feature.

```toml
[dependencies]
hisi-rf = {
    version = "0.1.0-alpha.98",
    features = ["chip-ws63", "profile-wifi-wpa2-smoltcp"]
}
```

Chip repositories implement the bounded radio backend; applications drive
TCP/IP through `embassy-net` or the optional `smoltcp::phy::Device` adapter.
WS63 applications construct uniquely owned resources through `hisi_rf::ws63`;
vendor archives, ROM symbols, schedulers, TLS, NVS formats, and image packaging
remain outside the facade API. Application code should prefer the named
`profile-wifi-wpa2-smoltcp` or `profile-wifi-wpa3-smoltcp` composition. The
orthogonal `wifi`/`smoltcp`/security features remain available for maintainer
matrices. An Embassy Net profile will be added only with a working backend.

BLE and SLE expose role-specific compositions through
`profile-ble-peripheral`, `profile-ble-central`, `profile-ble-dual-role`,
`profile-sle-announce`, `profile-sle-seek`, and `profile-sle-ssap`. Selecting a
narrow profile removes role-inapplicable controller methods at compile time.
All six currently use the same pinned WS63 BGLE archive and therefore report
the same physical task/arena budget; the facade does not invent memory savings
that the target archive cannot yet deliver. They share this caller-owned shape:

```rust,ignore
hisi_rf::declare_radio_storage!(static RADIO_STORAGE);

let installed = RADIO_STORAGE.install()?;
let controller = hisi_rf::ws63::init(resources, installed)?;
let parts = controller.split();
let _protocol = parts.ble; // `parts.sle` for `profile-sle-ssap`
let runner = parts.runner;
```

The same storage object reports the exact admission facts without allocation:

```rust,ignore
let report = RADIO_STORAGE.report();
assert_eq!(report.profile, hisi_rf::RadioProfile::BlePeripheral);
assert_eq!(report.dynamic_task_slots, 4);
```

These profiles provide typed GAP/GATT/SSAP commands, bounded event streams, and
generation-tagged lifecycle guards. The surface remains alpha until its stable
graduation review; applications must not bypass the facade and name the
internal `BleB*` or `SleS*` stage API.

The profile owns its bounded state and crypto DMA scratch through explicit
application storage:

```rust,ignore
hisi_rf::ws63::declare_radio_storage!(
    static RADIO_STORAGE,
    events = 4
);
```

Before starting `hisi-rtos`, call `RADIO_STORAGE.install()` once and pass its
`RuntimeAllocator` capability to the runtime selected by the application.
After RTOS startup, consume the installed storage with `resources(...)`; the
facade binds the arena and bounded control state to the selected security
profile without exposing backend arena or profile types.

The storage-bound controller splits into the Wi-Fi handles and the mandatory
bounded runner:

```rust,ignore
let installed = RADIO_STORAGE.install()?;
let allocator = installed.runtime_allocator();
start_selected_runtime(allocator)?;
let resources = bind_profile_resources(installed, peripherals);
let controller = hisi_rf::ws63::init(config, resources)?;
let parts = controller.split(
    hisi_rf::WorkBudget::try_new(8, 100_000).expect("non-zero work budget")
);
let hisi_rf::ws63::RadioParts { mut wifi, mut runner } = parts;

// The application executor keeps this future alive for the radio lifetime.
async fn run_radio(runner: &mut hisi_rf::ws63::RadioRunner) -> ! {
    loop {
        let ready = runner.wait_ready().await.expect("radio wait failed");
        runner.run_once(ready).expect("radio runner failed");
    }
}

let scan = wifi.controller.scan(scan_config, &mut results).await?;
let station_mac = wifi.device.station_mac_address()
    .ok_or("station netif has not been initialized")?;
```

The application does not import `hisi-rf-rtos-driver`, `ws63-radio-sys`, or a
chip backend type. Starting the runtime itself remains explicit application
policy rather than a hidden side effect of radio initialization.
The station MAC accessor becomes available after radio initialization and lets
the application configure a standard IP stack without importing backend netif
internals.

`RadioStorage::report()` provides allocation-free, versioned resource metadata.
The same contract can be emitted without naming the chip backend crate:

```console
cargo run --example ws63_resource_report --target <host-triple> \
  --features chip-ws63,profile-wifi-wpa2-smoltcp
```
Task-stack, supplicant-arena, and final-image totals remain marked uncalibrated
until the runtime and HIL admission contracts can supply them truthfully.

Public [`hisi_rf::Error`](https://docs.rs/hisi-rf/latest/hisi_rf/enum.Error.html)
values expose `diagnostic()`, a versioned, allocation-free view with a stable
machine code, stage, recovery action, documentation anchor, optional raw
backend code, immutable profile revision, and a four-entry numeric trace. Its
JSON form reports trace truncation and cannot contain SSIDs, passphrases, or key
material because those values are not part of the diagnostic type.

This crate is an early alpha. The current public surface may change while WS63
connectivity parity is established on real silicon.
