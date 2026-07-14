# hisi-rf

`hisi-rf` is the chip-neutral radio API for the hispark-rs ecosystem. The first
vertical slice provides Wi-Fi control, L2 device ownership, a mandatory
background runner, and bounded events without owning an IP stack.

Chip repositories implement `WifiBackend`; applications drive TCP/IP through
`embassy-net` or the optional `smoltcp::phy::Device` adapter. Vendor archives,
ROM symbols, schedulers, TLS, NVS formats, and image packaging stay outside this
crate.

This crate is an early alpha. The current public surface may change while WS63
connectivity parity is established on real silicon.

