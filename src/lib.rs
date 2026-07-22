//! HiSilicon radio facade.
//!
//! Applications select exactly one `chip-*` feature. The facade re-exports the
//! chip-neutral API from [`hisi_rf_core`] and exposes only the selected chip's
//! safe composition root; raw sys/blob/runtime-driver crates stay transitive.

#![no_std]

#[cfg(not(feature = "chip-ws63"))]
compile_error!("select exactly one chip feature, for example `chip-ws63`");

#[cfg(all(feature = "wpa2-personal", feature = "wpa3-personal"))]
compile_error!("select exactly one Personal security profile");

#[cfg(all(
    any(feature = "wpa2-personal", feature = "wpa3-personal"),
    not(feature = "smoltcp")
))]
compile_error!(
    "the current WS63 Personal profile requires `smoltcp`; an Embassy Net profile is not available yet"
);

pub use hisi_rf_core::{
    BackendError, BackendErrorClass, ConnectionInfo, DIAGNOSTIC_SCHEMA, DIAGNOSTIC_TRACE_CAPACITY,
    Diagnostic, DiagnosticCode, DiagnosticStage, DiagnosticTrace, DiagnosticTraceEntry,
    DiagnosticTraceKind, Error, EventDiagnostics, ManagementFrameProtection, Passphrase,
    PersonalSecurity, RadioConfig, RadioController, RadioParts, RadioRunner, RecoveryAction,
    SaePwe, ScanConfig, ScanOutcome, ScanResult, Security, Ssid, StationConfig, WifiBackend,
    WifiConfig, WifiController, WifiDevice, WifiEvent, WifiParts,
};

#[cfg(feature = "incremental-backend-experiment")]
pub use hisi_rf_core::{
    CancelDirective, CancelOutcome, CommandArbiter, CommandArbiterAction, CommandArbiterError,
    CommandSequence, FairWakeSelector, IncrementalBackendDriver, IncrementalCompletion,
    IncrementalDriverError, IncrementalDriverEvent, IncrementalRequest, IncrementalRunnerState,
    IncrementalWifiBackend, OperationId, OperationLifecycle, OperationStateError, OperationTracker,
    PendingCommand, PollDisposition, RunnerStateError, RunnerStep, RunnerTransition, SubmitError,
    WaitSet, WakeReason, WorkBudget, WorkReport,
};

/// WS63 safe resources and radio composition root.
#[cfg(feature = "chip-ws63")]
pub mod ws63 {
    #[cfg(all(
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    pub use hisi_rf_ws63::{
        InitError, RadioController, ResourceReport, Resources, RfHeapMetrics, SelectedProfile,
        Storage, WifiWpa2Smoltcp, WifiWpa3Smoltcp, init, rf_heap_metrics, station_mac_address,
    };
}
