//! HiSilicon radio facade.
//!
//! This migration release re-exports the complete chip-neutral API from
//! [`hisi_rf_core`], preserving existing `hisi_rf::*` source paths while chip
//! backends move behind feature-selected composition roots.

#![no_std]

pub use hisi_rf_core::{
    BackendError, BackendErrorClass, ConnectionInfo, Error, EventDiagnostics,
    ManagementFrameProtection, Passphrase, PersonalSecurity, RadioConfig, RadioController,
    RadioParts, RadioResources, RadioRunner, RadioState, SaePwe, ScanConfig, ScanOutcome,
    ScanResult, Security, Ssid, StationConfig, WifiBackend, WifiConfig, WifiController, WifiDevice,
    WifiEvent, WifiParts, init,
};
