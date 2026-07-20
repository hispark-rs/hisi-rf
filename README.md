# hisi-rf

`hisi-rf` is the application-facing radio facade for the hispark-rs ecosystem.
It re-exports the chip-neutral controller, runner, configuration, event, and L2
contracts from `hisi-rf-core`, then selects a safe chip composition root through
an explicit `chip-*` feature.

```toml
[dependencies]
hisi-rf = {
    version = "0.1.0-alpha.5",
    features = ["chip-ws63", "wifi", "wpa2-personal", "smoltcp"]
}
```

Chip repositories implement `WifiBackend`; applications drive TCP/IP through
`embassy-net` or the optional `smoltcp::phy::Device` adapter. WS63 applications
construct uniquely owned resources through `hisi_rf::ws63`; vendor archives,
ROM symbols, schedulers, TLS, NVS formats, and image packaging remain outside
the facade API. The current Personal profiles use the available smoltcp data
plane; an Embassy Net profile is planned separately.

This crate is an early alpha. The current public surface may change while WS63
connectivity parity is established on real silicon.
