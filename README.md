# hisi-rf

`hisi-rf` is the application-facing radio facade for the hispark-rs ecosystem.
It re-exports the chip-neutral controller, runner, configuration, event, and L2
contracts from `hisi-rf-core`, then selects a safe chip composition root through
an explicit `chip-*` feature.

```toml
[dependencies]
hisi-rf = {
    version = "0.1.0-alpha.9",
    features = ["chip-ws63", "profile-wifi-wpa2-smoltcp"]
}
```

Chip repositories implement `WifiBackend`; applications drive TCP/IP through
`embassy-net` or the optional `smoltcp::phy::Device` adapter. WS63 applications
construct uniquely owned resources through `hisi_rf::ws63`; vendor archives,
ROM symbols, schedulers, TLS, NVS formats, and image packaging remain outside
the facade API. Application code should prefer the named
`profile-wifi-wpa2-smoltcp` or `profile-wifi-wpa3-smoltcp` composition. The
orthogonal `wifi`/`smoltcp`/security features remain available for maintainer
matrices. An Embassy Net profile will be added only with a working backend.

The profile owns its bounded state and crypto DMA scratch through explicit
application storage:

```rust,ignore
static RADIO_STORAGE: hisi_rf::ws63::Storage<hisi_rf::ws63::SelectedProfile, 4> =
    hisi_rf::ws63::Storage::new();
```

`Storage::report()` provides allocation-free, versioned resource metadata.
Task-stack, supplicant-arena, and final-image totals remain marked uncalibrated
until the runtime and HIL admission contracts can supply them truthfully.

Public [`hisi_rf::Error`](https://docs.rs/hisi-rf/latest/hisi_rf/enum.Error.html)
values expose `diagnostic()`, a versioned, allocation-free view with a stable
machine code, stage, recovery action, documentation anchor, and optional raw
backend code. Its JSON form cannot contain SSIDs, passphrases, or key material
because those values are not part of the diagnostic type.

This crate is an early alpha. The current public surface may change while WS63
connectivity parity is established on real silicon.
