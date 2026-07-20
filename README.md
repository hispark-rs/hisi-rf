# hisi-rf

`hisi-rf` is the application-facing radio facade for the hispark-rs ecosystem.
The current migration release re-exports the chip-neutral controller, runner,
configuration, event, and L2 contracts from `hisi-rf-core` without changing
existing `hisi_rf::*` paths.

Chip repositories implement `WifiBackend`; applications drive TCP/IP through
`embassy-net` or the optional `smoltcp::phy::Device` adapter. A later alpha adds
the feature-selected WS63 composition root after its backend crate is packaged.
Vendor archives, ROM symbols, schedulers, TLS, NVS formats, and image packaging
remain outside the facade API.

This crate is an early alpha. The current public surface may change while WS63
connectivity parity is established on real silicon.
