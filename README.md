# hisi-rf

`hisi-rf` is the application-facing radio facade for the hispark-rs ecosystem.
It re-exports the chip-neutral controller, runner, configuration, event, and L2
contracts from `hisi-rf-core`, then selects a safe chip composition root through
an explicit `chip-*` feature.

```toml
[dependencies]
hisi-rf = {
    version = "0.1.0-alpha.6",
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

This crate is an early alpha. The current public surface may change while WS63
connectivity parity is established on real silicon.
