//! HiSilicon radio facade.
//!
//! Applications select exactly one `chip-*` feature. The facade re-exports the
//! chip-neutral API from [`hisi_rf_core`] and exposes only the selected chip's
//! safe composition root; raw sys/blob/runtime-driver crates stay transitive.

#![no_std]

#[cfg(all(
    feature = "chip-ws63",
    feature = "smoltcp",
    any(feature = "wpa2-personal", feature = "wpa3-personal")
))]
mod ws63_diagnostics;

#[cfg(not(feature = "chip-ws63"))]
compile_error!("select exactly one chip feature, for example `chip-ws63`");

#[cfg(all(feature = "wpa2-personal", feature = "wpa3-personal"))]
compile_error!("select exactly one Personal security profile");

#[cfg(all(
    any(
        feature = "profile-wifi-wpa2-softap",
        feature = "profile-wifi-wpa3-softap"
    ),
    any(feature = "wpa2-personal", feature = "wpa3-personal")
))]
compile_error!("select either the WS63 SoftAP profile or one station profile");

#[cfg(all(
    feature = "profile-wifi-wpa2-softap",
    feature = "profile-wifi-wpa3-softap"
))]
compile_error!("select exactly one WS63 SoftAP security profile");

#[cfg(all(
    any(feature = "wpa2-personal", feature = "wpa3-personal"),
    not(feature = "smoltcp")
))]
compile_error!(
    "the current WS63 Personal profile requires `smoltcp`; an Embassy Net profile is not available yet"
);

pub use hisi_rf_core::{
    BackendError, BackendErrorClass, BackendTimeout, BlockingRunnerDiagnostics, ConnectionInfo,
    DIAGNOSTIC_SCHEMA, DIAGNOSTIC_TRACE_CAPACITY, Diagnostic, DiagnosticCode, DiagnosticStage,
    DiagnosticTrace, DiagnosticTraceEntry, DiagnosticTraceKind, Error, EventDiagnostics,
    ManagementFrameProtection, OperationTimeout, Passphrase, PersonalSecurity, RadioConfig,
    RecoveryAction, SaePwe, ScanConfig, ScanOutcome, ScanResult, Security, Ssid, StationConfig,
    WifiBackend, WifiConfig, WifiDevice, WifiEvent, WifiL2Capabilities,
};
/// Event capacity selected by the public WS63 application profiles.
pub const EVENT_CAPACITY: usize = 8;

/// Chip-neutral controller for the selected public profile.
pub type RadioController<B, D> = hisi_rf_core::RadioController<B, D, EVENT_CAPACITY>;
/// Chip-neutral parts for the selected public profile.
pub type RadioParts<B, D> = hisi_rf_core::RadioParts<B, D, EVENT_CAPACITY>;
/// Chip-neutral runner for the selected public profile.
pub type RadioRunner<B> = hisi_rf_core::RadioRunner<B, EVENT_CAPACITY>;
/// Wi-Fi controller for the selected public profile.
pub type WifiController = hisi_rf_core::WifiController<EVENT_CAPACITY>;
/// Wi-Fi control and data-plane parts for the selected public profile.
pub type WifiParts<D> = hisi_rf_core::WifiParts<D, EVENT_CAPACITY>;

/// WS63 SoftAP composition selected through the public facade.
///
/// This module is mutually exclusive with the station-oriented `ws63` module
/// below because each firmware image contains exactly one hostap target role.
#[cfg(all(
    feature = "chip-ws63",
    any(
        feature = "profile-wifi-wpa2-softap",
        feature = "profile-wifi-wpa3-softap"
    )
))]
pub mod ws63 {
    pub use hisi_rf_ws63::{
        ACCESS_POINT_ARENA_BYTES, AccessPoint, AccessPointArenaStorage, AccessPointConfig,
        AccessPointControlStorage, AccessPointDiagnostics, AccessPointInitError,
        AccessPointNetworkDevice, AccessPointResources, AccessPointStorage,
        InstalledAccessPointStorage, NativeAuthenticatorError, declare_access_point_storage,
        init_access_point, netif, netif_smoltcp,
    };
    #[cfg(feature = "profile-wifi-wpa3-softap")]
    pub use hisi_rf_ws63::{
        hardware_p256_curve_diagnostic_snapshot, hardware_p256_diagnostic_snapshot,
        hardware_p256_field_diagnostic_snapshot,
    };
}

#[cfg(feature = "incremental-backend-experiment")]
pub use hisi_rf_core::{
    CancelDirective, CancelOutcome, CommandArbiter, CommandArbiterAction, CommandArbiterError,
    CommandSequence, FairWakeSelector, IncrementalBackendDriver, IncrementalCompletion,
    IncrementalDriverError, IncrementalDriverEvent, IncrementalRadioRunnerError,
    IncrementalRequest, IncrementalRunnerDiagnostics, IncrementalRunnerState, IncrementalWaitError,
    IncrementalWaitIntent, IncrementalWaitPlatform, IncrementalWifiBackend, OperationId,
    OperationLifecycle, OperationStateError, OperationTracker, PendingCommand, PollDisposition,
    RunnerStateError, RunnerStep, RunnerTransition, SubmitError, WaitSet, WakeReason, WorkBudget,
    WorkReport,
};
#[cfg(feature = "incremental-backend-experiment")]
/// Incremental parts for the selected public profile.
pub type IncrementalRadioParts<B, D> = hisi_rf_core::IncrementalRadioParts<B, D, EVENT_CAPACITY>;
#[cfg(feature = "incremental-backend-experiment")]
/// Incremental runner for the selected public profile.
pub type IncrementalRadioRunner<B> = hisi_rf_core::IncrementalRadioRunner<B, EVENT_CAPACITY>;

/// WS63 safe resources and radio composition root.
#[cfg(all(
    feature = "chip-ws63",
    feature = "smoltcp",
    any(feature = "wpa2-personal", feature = "wpa3-personal")
))]
pub mod ws63 {
    pub use crate::declare_radio_storage;
    pub use crate::ws63_diagnostics::{
        RADIO_DIAGNOSTICS_SCHEMA, RadioDiagnosticsSnapshot, RunnerDiagnosticsSnapshot,
        WaitDiagnosticsSnapshot,
    };
    #[allow(deprecated)]
    pub use hisi_rf_ws63::SELECTED_TASK_STACK_ARENA_BYTES;
    pub use hisi_rf_ws63::declare_radio_arena;
    #[cfg(target_arch = "riscv32")]
    #[doc(hidden)]
    pub use hisi_rf_ws63::upstream_supplicant_driver_event_diagnostic_snapshot;
    #[cfg(all(
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    pub use hisi_rf_ws63::{
        ArenaAdmissionError, AssociationIoctlMetrics, AssociationTimingDiagnostics,
        BlockingBackendMetrics, BlockingBootstrapMetrics, BlockingOperationMetrics, BootstrapStage,
        BootstrapStageMetrics, CryptoReady, DataPathDiagnostics, DhcpDiagnostics, InitError,
        InitErrorKind, InstalledRadioArena, L2ProtocolDiagnostics, MissingCrypto, MissingPke,
        PkeNotRequired, PkeReady, RadioArena, RadioArenaStorage, ResourceReport, Resources,
        ResourcesBuilder, RfHeapMetrics, RxQueueDiagnostics, SELECTED_MINIMUM_TASK_STACK_BYTES,
        SELECTED_RF_ARENA_BYTES, SELECTED_RUNTIME_ARENA_BYTES, ScanDiagnostics, SelectedProfile,
        WifiDevice, WifiRxToken, WifiTxToken, WifiWpa2Smoltcp, WifiWpa3Smoltcp,
        association_timing_diagnostics, blocking_backend_metrics, rf_heap_metrics,
    };
    #[cfg(feature = "ws63-station-pm-diagnostics")]
    #[doc(hidden)]
    pub use hisi_rf_ws63::{
        StationPowerSaveDiagnosticError, disable_station_power_save_for_diagnostics,
    };
    #[doc(hidden)]
    pub use hisi_rf_ws63::{
        upstream_supplicant_eapol_diagnostic_snapshot,
        upstream_supplicant_event_diagnostic_snapshot,
    };

    /// Implementation details used only by the facade storage macro.
    #[doc(hidden)]
    pub mod __private {
        pub use hisi_rf_ws63::RadioArenaStorage;
    }

    /// Event capacity owned by the selected WS63 profile storage.
    pub const EVENT_CAPACITY: usize = crate::EVENT_CAPACITY;

    /// Caller-owned control storage for the selected WS63 profile.
    pub struct Storage {
        inner: hisi_rf_ws63::Storage<SelectedProfile, EVENT_CAPACITY>,
    }

    impl Storage {
        /// Construct unclaimed storage suitable for a static item.
        pub const fn new() -> Self {
            Self {
                inner: hisi_rf_ws63::Storage::new(),
            }
        }

        /// Return the selected profile's deterministic resource report.
        pub const fn report(&self) -> ResourceReport {
            self.inner.report()
        }
    }

    #[cfg(any(
        not(feature = "incremental-backend-experiment"),
        feature = "incremental-embassy-wait"
    ))]
    const SELECTED_RESOURCE_REPORT: ResourceReport =
        hisi_rf_ws63::resource_report::<SelectedProfile, EVENT_CAPACITY>();

    /// Capture the complete public diagnostic view from task-split Wi-Fi handles.
    #[cfg(any(
        not(feature = "incremental-backend-experiment"),
        feature = "incremental-embassy-wait"
    ))]
    pub fn diagnostics(
        controller: &crate::WifiController,
        device: &WifiDevice,
    ) -> RadioDiagnosticsSnapshot {
        #[cfg(not(feature = "incremental-backend-experiment"))]
        {
            RadioDiagnosticsSnapshot::blocking(controller, device, SELECTED_RESOURCE_REPORT)
        }
        #[cfg(all(
            feature = "incremental-backend-experiment",
            feature = "incremental-embassy-wait"
        ))]
        {
            RadioDiagnosticsSnapshot::incremental(controller, device, SELECTED_RESOURCE_REPORT)
        }
    }

    impl Default for Storage {
        fn default() -> Self {
            Self::new()
        }
    }

    /// One caller-owned radio composition with a linker-placed arena.
    pub struct RadioStorage {
        control: &'static Storage,
        arena: &'static RadioArenaStorage<{ SELECTED_RF_ARENA_BYTES }>,
    }

    impl RadioStorage {
        /// Construct the composition from backing stores emitted by the macro.
        #[doc(hidden)]
        pub const fn __from_parts(
            control: &'static Storage,
            arena: &'static RadioArenaStorage<{ SELECTED_RF_ARENA_BYTES }>,
        ) -> Self {
            Self { control, arena }
        }

        /// Install the caller-owned composition exactly once.
        pub fn install(&'static self) -> Result<InstalledRadioStorage, ArenaAdmissionError> {
            let arena = self.arena.claim_for::<SelectedProfile>()?.install()?;
            Ok(InstalledRadioStorage {
                control: self.control,
                arena,
            })
        }

        /// Return the selected profile's deterministic resource report.
        pub const fn report(&self) -> ResourceReport {
            self.control.report()
        }
    }

    /// Installed storage capability for the selected WS63 profile.
    pub struct InstalledRadioStorage {
        control: &'static Storage,
        arena: InstalledRadioArena<SelectedProfile>,
    }

    impl InstalledRadioStorage {
        /// Allocate one zeroed RTOS block from the installed composition.
        ///
        /// # Safety
        ///
        /// The pointer must be released only through [`Self::deallocate`].
        pub unsafe fn allocate(size: usize) -> *mut u8 {
            // SAFETY: this method preserves the backend allocator's contract.
            unsafe { InstalledRadioArena::<SelectedProfile>::allocate(size) }
        }

        /// Release one RTOS allocation.
        ///
        /// # Safety
        ///
        /// `pointer` must be null or a live allocation returned by
        /// [`Self::allocate`] that has not already been released.
        pub unsafe fn deallocate(pointer: *mut u8) {
            // SAFETY: the caller upholds the backend deallocation contract.
            unsafe { InstalledRadioArena::<SelectedProfile>::deallocate(pointer) };
        }

        /// Split storage at the post-RTOS radio initialization boundary.
        pub fn into_init_parts(self) -> (&'static Storage, InstalledRadioArena<SelectedProfile>) {
            (self.control, self.arena)
        }
    }

    /// WS63 controller for the selected public profile.
    pub struct RadioController(hisi_rf_ws63::RadioController<SelectedProfile, EVENT_CAPACITY>);

    impl RadioController {
        /// Start the mandatory backend runner and return fixed-profile Wi-Fi parts.
        pub fn start_runner(self) -> Result<WifiParts, InitError> {
            let parts = self.0.start_runner()?;
            Ok(WifiParts {
                controller: parts.controller,
                device: parts.device,
            })
        }
    }

    /// WS63 Wi-Fi parts for the selected public profile.
    pub struct WifiParts {
        /// Async control plane with profile-owned event capacity.
        pub controller: crate::WifiController,
        /// Opaque WS63 L2 device.
        pub device: WifiDevice,
    }

    impl WifiParts {
        /// Capture all public blocking-profile diagnostics in one secret-free view.
        #[cfg(not(feature = "incremental-backend-experiment"))]
        pub fn diagnostics(&self) -> RadioDiagnosticsSnapshot {
            diagnostics(&self.controller, &self.device)
        }
    }

    #[cfg(all(
        feature = "incremental-backend-experiment",
        feature = "incremental-embassy-wait"
    ))]
    impl WifiParts {
        /// Capture all public incremental-profile diagnostics after task split.
        pub fn diagnostics(&self) -> RadioDiagnosticsSnapshot {
            diagnostics(&self.controller, &self.device)
        }
    }

    #[cfg(all(
        not(feature = "incremental-backend-experiment"),
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// Initialize the selected WS63 public profile.
    pub fn init(
        config: crate::RadioConfig,
        resources: Resources<SelectedProfile>,
        storage: &'static Storage,
    ) -> Result<RadioController, InitError> {
        hisi_rf_ws63::init(config, resources, &storage.inner).map(RadioController)
    }

    #[cfg(all(
        feature = "incremental-backend-experiment",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// Incremental WS63 controller for the selected public profile.
    #[cfg_attr(
        not(feature = "incremental-embassy-wait"),
        allow(dead_code, reason = "split is enabled by incremental-embassy-wait")
    )]
    pub struct IncrementalRadioController(
        hisi_rf_ws63::IncrementalRadioController<SelectedProfile, EVENT_CAPACITY>,
    );

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    impl IncrementalRadioController {
        /// Split into fixed-profile Wi-Fi parts and a bounded runner.
        pub fn split(self, budget: crate::WorkBudget) -> IncrementalRadioParts {
            let parts = self.0.split(budget);
            IncrementalRadioParts {
                wifi: WifiParts {
                    controller: parts.wifi.controller,
                    device: parts.wifi.device,
                },
                runner: IncrementalRadioRunner(parts.runner),
            }
        }
    }

    #[cfg(all(
        feature = "incremental-backend-experiment",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// Initialize the selected experimental incremental WS63 profile.
    pub fn init(
        config: crate::RadioConfig,
        resources: Resources<SelectedProfile>,
        storage: &'static Storage,
    ) -> Result<IncrementalRadioController, InitError> {
        hisi_rf_ws63::init_incremental(config, resources, &storage.inner)
            .map(IncrementalRadioController)
    }

    /// Migration alias for the selected incremental initialization path.
    #[cfg(all(
        feature = "incremental-backend-experiment",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    #[deprecated(since = "0.1.0-alpha.53", note = "use hisi_rf::ws63::init")]
    pub fn init_incremental_after_blocking_bootstrap(
        config: crate::RadioConfig,
        resources: Resources<SelectedProfile>,
        storage: &'static Storage,
    ) -> Result<IncrementalRadioController, InitError> {
        #[allow(deprecated)]
        hisi_rf_ws63::init_incremental_after_blocking_bootstrap(config, resources, &storage.inner)
            .map(IncrementalRadioController)
    }

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    pub use hisi_rf_ws63::Ws63IncrementalWaitDiagnostics;

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// Incremental WS63 parts for the selected public profile.
    pub struct IncrementalRadioParts {
        /// Async Wi-Fi control and data plane.
        pub wifi: WifiParts,
        /// Bounded incremental runner.
        pub runner: IncrementalRadioRunner,
    }

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    impl IncrementalRadioParts {
        /// Capture runner, wait, event, backend, L2, and resource diagnostics.
        pub fn diagnostics(&self) -> RadioDiagnosticsSnapshot {
            self.wifi.diagnostics()
        }
    }

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// Incremental WS63 runner for the selected public profile.
    pub struct IncrementalRadioRunner(hisi_rf_ws63::IncrementalRadioRunner<EVENT_CAPACITY>);

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    impl IncrementalRadioRunner {
        /// Advance at most one bounded driver action.
        pub fn run_once(
            &mut self,
            ready: crate::WaitSet,
        ) -> Result<crate::IncrementalDriverEvent, crate::IncrementalRadioRunnerError> {
            self.0.run_once(ready)
        }

        /// Snapshot immediate work, wake subscriptions, and the next deadline.
        pub fn wait_intent(&self) -> crate::IncrementalWaitIntent {
            self.0.wait_intent()
        }

        /// Wait for one subscribed WS63 source.
        pub async fn wait_ready(
            &mut self,
        ) -> Result<crate::WaitSet, crate::IncrementalWaitError<core::convert::Infallible>>
        {
            self.0.wait_ready().await
        }

        /// Monotonic deadline requested by the active operation.
        pub fn next_deadline_us(&self) -> Option<u64> {
            self.0.next_deadline_us()
        }

        /// Snapshot chip-neutral runner diagnostics.
        pub fn diagnostics(&self) -> crate::IncrementalRunnerDiagnostics {
            self.0.diagnostics()
        }

        /// Snapshot the WS63 wait bridge.
        pub fn wait_diagnostics(&self) -> Ws63IncrementalWaitDiagnostics {
            self.0.wait_diagnostics()
        }
    }
}

/// Declare one caller-owned WS63 radio composition for the selected profile.
#[cfg(all(
    feature = "chip-ws63",
    feature = "smoltcp",
    any(feature = "wpa2-personal", feature = "wpa3-personal")
))]
#[macro_export]
macro_rules! declare_radio_storage {
    ($(#[$meta:meta])* $vis:vis static $name:ident) => {
        $(#[$meta])*
        $vis static $name: $crate::ws63::RadioStorage = {
            static CONTROL: $crate::ws63::Storage = $crate::ws63::Storage::new();
            #[cfg_attr(
                target_arch = "riscv32",
                unsafe(link_section = ".hisi.shared-arena")
            )]
            static ARENA: $crate::ws63::__private::RadioArenaStorage<
                { $crate::ws63::SELECTED_RF_ARENA_BYTES },
            > = $crate::ws63::__private::RadioArenaStorage::new();
            $crate::ws63::RadioStorage::__from_parts(&CONTROL, &ARENA)
        };
    };
}

#[cfg(all(
    test,
    feature = "chip-ws63",
    feature = "incremental-backend-experiment",
    feature = "smoltcp",
    any(feature = "wpa2-personal", feature = "wpa3-personal")
))]
mod tests {
    type TestProfile = super::ws63::SelectedProfile;
    type TestStorage = super::ws63::Storage;
    type TestController = super::ws63::IncrementalRadioController;
    type TestInit = fn(
        super::RadioConfig,
        super::ws63::Resources<TestProfile>,
        &'static TestStorage,
    ) -> Result<TestController, super::ws63::InitError>;

    #[test]
    fn facade_exposes_the_explicit_ws63_incremental_lifecycle() {
        let _: TestInit = super::ws63::init;
        let _: Option<super::IncrementalRunnerDiagnostics> = None;
        let _: Option<super::ws63::DhcpDiagnostics> = None;
        let _: Option<super::ws63::DataPathDiagnostics> = None;
        let _: Option<super::ws63::RxQueueDiagnostics> = None;
        let _: Option<super::ws63::AssociationTimingDiagnostics> = None;
    }

    #[cfg(feature = "incremental-embassy-wait")]
    #[test]
    fn facade_exposes_the_explicit_ws63_embassy_wait_bridge() {
        let _: Option<super::ws63::Ws63IncrementalWaitDiagnostics> = None;
    }
}
