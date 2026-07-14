//! Chip-neutral radio control and L2 data-plane contracts.
//!
//! [`init`] claims caller-provided static state and returns an exclusive
//! [`RadioController`]. Splitting it yields a [`WifiController`], a
//! [`WifiDevice`], and the mandatory [`RadioRunner`]. Only the runner calls the
//! chip backend; control methods merely enqueue commands and await completion.

#![no_std]

mod state;
mod wifi;

pub use wifi::{
    BackendError, BackendErrorClass, ConnectionInfo, EventDiagnostics, Passphrase, RadioConfig,
    RadioController, RadioParts, RadioResources, RadioRunner, RadioState, ScanConfig, ScanOutcome,
    ScanResult, Security, Ssid, StationConfig, WifiBackend, WifiConfig, WifiController, WifiDevice,
    WifiEvent, WifiParts, init,
};

/// Failure to establish or use the radio control plane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The supplied [`RadioState`] has already been claimed.
    AlreadyInitialized,
    /// The chip backend rejected an operation.
    Backend(BackendError),
    /// A backend completion did not match the outstanding command.
    Protocol,
}
