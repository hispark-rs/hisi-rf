//! HiSilicon radio facade.
//!
//! Applications select exactly one `chip-*` feature. The facade re-exports the
//! chip-neutral API from [`hisi_rf_core`] and exposes only the selected chip's
//! safe composition root; raw sys/blob/runtime-driver crates stay transitive.

#![no_std]

#[cfg(test)]
extern crate std;

#[cfg(all(
    feature = "chip-ws63",
    feature = "smoltcp",
    any(feature = "wpa2-personal", feature = "wpa3-personal")
))]
mod ws63_diagnostics;

#[cfg(not(feature = "chip-ws63"))]
compile_error!("select exactly one chip feature, for example `chip-ws63`");

#[cfg(all(
    feature = "chip-ws63",
    not(any(
        feature = "profile-wifi-wpa2-smoltcp",
        feature = "profile-wifi-wpa3-smoltcp",
        feature = "profile-wifi-wpa2-softap",
        feature = "profile-wifi-wpa3-softap",
        feature = "profile-ble-dual-role",
        feature = "profile-sle-ssap"
    ))
))]
compile_error!("select exactly one WS63 named profile, for example `profile-wifi-wpa2-smoltcp`");

#[cfg(all(
    feature = "profile-ble-dual-role",
    any(
        feature = "profile-sle-ssap",
        feature = "profile-wifi-wpa2-smoltcp",
        feature = "profile-wifi-wpa3-smoltcp",
        feature = "profile-wifi-wpa2-softap",
        feature = "profile-wifi-wpa3-softap"
    )
))]
compile_error!("the BLE migration profile must be the only selected radio profile");

#[cfg(all(
    feature = "profile-sle-ssap",
    any(
        feature = "profile-wifi-wpa2-smoltcp",
        feature = "profile-wifi-wpa3-smoltcp",
        feature = "profile-wifi-wpa2-softap",
        feature = "profile-wifi-wpa3-softap"
    )
))]
compile_error!("the SLE migration profile must be the only selected radio profile");

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

#[cfg(feature = "ble")]
pub use hisi_rf_core::ble;
#[cfg(feature = "sle")]
pub use hisi_rf_core::sle;
pub use hisi_rf_core::{
    BackendError, BackendErrorClass, BackendTimeout, BlockingRunnerDiagnostics, ConnectionInfo,
    DIAGNOSTIC_SCHEMA, DIAGNOSTIC_TRACE_CAPACITY, Diagnostic, DiagnosticCode, DiagnosticStage,
    DiagnosticTrace, DiagnosticTraceEntry, DiagnosticTraceKind, Error, EventDiagnostics,
    ManagementFrameProtection, OperationTimeout, Passphrase, PersonalSecurity, RadioConfig,
    RecoveryAction, SaePwe, ScanConfig, ScanOutcome, ScanResult, Security, Ssid, StationConfig,
    WifiConfig, WifiDevice, WifiEvent, WifiL2Capabilities,
};

/// Generation-tagged identity of one accepted protocol command.
#[cfg(any(feature = "ble", feature = "sle"))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProtocolCommandId(hisi_rf_core::control::ControlId);

#[cfg(any(feature = "ble", feature = "sle"))]
impl ProtocolCommandId {
    /// Return the stable non-zero representation used by diagnostics.
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

/// Conservation snapshot for the public unsolicited-event queue.
#[cfg(any(feature = "ble", feature = "sle"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolEventDiagnostics {
    /// Events accepted from the radio runner.
    pub accepted: u32,
    /// Events consumed by the protocol controller.
    pub consumed: u32,
    /// Events rejected because the bounded queue was full.
    pub dropped: u32,
    /// Events waiting for the protocol controller.
    pub pending: usize,
    /// Largest observed queue occupancy.
    pub high_water: usize,
}

#[cfg(any(feature = "ble", feature = "sle"))]
fn protocol_event_diagnostics(
    value: hisi_rf_core::control::EventQueueDiagnostics,
) -> ProtocolEventDiagnostics {
    ProtocolEventDiagnostics {
        accepted: value.accepted,
        consumed: value.consumed,
        dropped: value.dropped,
        pending: value.pending,
        high_water: value.high_water,
    }
}

/// Backpressure that preserves ownership of a request the runner did not accept.
#[cfg(any(feature = "ble", feature = "sle"))]
#[derive(Debug)]
pub struct ProtocolBusy<T> {
    request: T,
}

#[cfg(any(feature = "ble", feature = "sle"))]
impl<T> ProtocolBusy<T> {
    /// Recover the request for retry or cancellation.
    pub fn into_inner(self) -> T {
        self.request
    }
}

/// One terminal runner result correlated with its accepted command.
#[cfg(any(feature = "ble", feature = "sle"))]
#[derive(Debug)]
pub struct ProtocolCompletion<T, E> {
    id: ProtocolCommandId,
    result: Result<T, E>,
}

#[cfg(any(feature = "ble", feature = "sle"))]
impl<T, E> ProtocolCompletion<T, E> {
    /// Correlation identity assigned when the request entered the mailbox.
    pub const fn id(&self) -> ProtocolCommandId {
        self.id
    }

    /// Recover the operation-specific result.
    pub fn into_result(self) -> Result<T, E> {
        self.result
    }
}

/// A controller/runner ownership invariant was violated.
#[cfg(any(feature = "ble", feature = "sle"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    /// A completion did not match the controller's live command generation.
    StaleCompletion,
    /// The runner could not publish the result for its active command.
    CompletionOwnership,
}
/// Event capacity selected by the public WS63 application profiles.
pub const EVENT_CAPACITY: usize = 8;

/// Wi-Fi controller for the selected public profile.
pub type WifiController = hisi_rf_core::WifiController<EVENT_CAPACITY>;

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
    #[doc(hidden)]
    pub use hisi_rf_ws63::{
        osal_queue::{
            EventDiagnostic as OsalEventDiagnostic, event_diagnostics as osal_event_diagnostics,
        },
        osal_wait::{
            WaitDiagnostic as OsalWaitDiagnostic, wait_diagnostics as osal_wait_diagnostics,
        },
    };
}

/// WS63 BLE U4 composition preview.
///
/// This profile establishes facade-owned storage, initialization, typed GAP
/// command submission, and runner ownership. The returned completion means the
/// WS63 host synchronously accepted or rejected the request. Static typed GATT
/// registration and bounded asynchronous lifecycle events are supported;
/// generation-tagged cancellation and active lifecycle guards are supported.
/// Applications must not bypass this facade to depend on internal stage APIs.
#[cfg(all(feature = "chip-ws63", feature = "profile-ble-dual-role"))]
pub mod ws63 {
    pub use crate::declare_radio_storage;

    #[doc(hidden)]
    pub enum BleCommand {
        StartAdvertising(crate::ble::AdvertisingConfig),
        StartScanning(crate::ble::ScanConfig),
        RegisterGattServer(crate::ble::GattServerDefinition),
        ConfigureSecurity(crate::ble::SecurityConfig),
        Pair(crate::ble::BluetoothAddress),
        QueryPairingState(crate::ble::BluetoothAddress),
        RemoveBond(crate::ble::BluetoothAddress),
    }

    type BleControlState =
        hisi_rf_core::control::ControlState<BleCommand, Result<BleOperation, BleOperationError>>;
    type BleEventState = hisi_rf_core::control::EventState<BleEvent, { crate::EVENT_CAPACITY }>;
    type BleLifecycleState = hisi_rf_core::control::LifecycleState<BleOperationError>;

    /// Caller-owned bytes shared by the pinned BLE controller and host tasks.
    pub const RADIO_ARENA_BYTES: usize = hisi_rf_ws63::BLE_B1_ARENA_BYTES;
    /// Smallest stack admitted by the pinned BLE task inventory.
    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    pub const MINIMUM_TASK_STACK_BYTES: usize = hisi_rf_ws63::BLE_B1_MINIMUM_TASK_STACK_BYTES;

    /// Advertising payload used only by the paired U2 facade HIL fixture.
    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    pub const U2_HIL_PEER_PAYLOAD: &[u8] = &[
        2, 0x01, 0x06, 8, 0x09, b'H', b'I', b'S', b'I', b'U', b'2', b'B',
    ];

    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    pub use hisi_rf_ws63::set_log_sink;

    /// Facade-owned lifecycle observations for the temporary U2 HIL gate.
    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum U2HilEvent {
        AdvertisingStarted,
        ScanReady,
        PeerObserved,
        BackendError { stage: u8, status: u32 },
    }

    #[doc(hidden)]
    pub mod __private {
        pub use hisi_rf_ws63::{BleB1ArenaStorage as ArenaStorage, BleB1ControlStorage};
        pub type ProtocolControlStorage = super::BleControlState;
        pub type ProtocolEventStorage = super::BleEventState;
        pub type ProtocolLifecycleStorage = super::BleLifecycleState;
    }

    /// Caller-owned BLE composition storage.
    pub struct RadioStorage {
        inner: hisi_rf_ws63::BleB1Storage<RADIO_ARENA_BYTES>,
        control: &'static BleControlState,
        events: &'static BleEventState,
        advertising: &'static BleLifecycleState,
        scanning: &'static BleLifecycleState,
    }

    impl RadioStorage {
        /// Join the statically allocated control state and arena.
        #[doc(hidden)]
        pub const fn __from_parts(
            control: &'static __private::BleB1ControlStorage,
            arena: &'static __private::ArenaStorage<RADIO_ARENA_BYTES>,
            protocol: &'static __private::ProtocolControlStorage,
            events: &'static __private::ProtocolEventStorage,
            advertising: &'static __private::ProtocolLifecycleStorage,
            scanning: &'static __private::ProtocolLifecycleStorage,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::BleB1Storage::from_parts(control, arena),
                control: protocol,
                events,
                advertising,
                scanning,
            }
        }

        /// Claim and install this process-lifetime storage once.
        pub fn install(&'static self) -> Result<InstalledRadioStorage, InitError> {
            self.inner
                .install()
                .map(|inner| InstalledRadioStorage {
                    inner,
                    control: self.control,
                    events: self.events,
                    advertising: self.advertising,
                    scanning: self.scanning,
                })
                .map_err(|_| InitError::new())
        }
    }

    /// Installed BLE storage capability.
    pub struct InstalledRadioStorage {
        inner: hisi_rf_ws63::InstalledBleB1Storage,
        control: &'static BleControlState,
        events: &'static BleEventState,
        advertising: &'static BleLifecycleState,
        scanning: &'static BleLifecycleState,
    }

    impl InstalledRadioStorage {
        /// Allocate one zeroed runtime object from the installed arena.
        ///
        /// # Safety
        ///
        /// The pointer must be returned only through [`Self::deallocate`].
        pub unsafe fn allocate(size: usize) -> *mut u8 {
            // SAFETY: this facade preserves the backend allocator contract.
            unsafe { hisi_rf_ws63::InstalledBleB1Storage::allocate(size) }
        }

        /// Release an allocation from this composition.
        ///
        /// # Safety
        ///
        /// `pointer` must be null or a live allocation returned by
        /// [`Self::allocate`] that has not already been released.
        pub unsafe fn deallocate(pointer: *mut u8) {
            // SAFETY: the caller upholds the backend deallocation contract.
            unsafe { hisi_rf_ws63::InstalledBleB1Storage::deallocate(pointer) };
        }
    }

    /// Uniquely owned HAL capabilities required by this BLE profile.
    pub struct Resources {
        inner: hisi_rf_ws63::BleB1Resources,
    }

    impl Resources {
        /// Bind the WS63 eFuse and crypto peripherals to the radio lifecycle.
        pub const fn new(
            efuse: hisi_hal::peripherals::Efuse<'static>,
            km: hisi_hal::peripherals::Km<'static>,
            spacc: hisi_hal::peripherals::Spacc<'static>,
            pke: hisi_hal::peripherals::Pke<'static>,
            trng: hisi_hal::peripherals::Trng<'static>,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::BleB1Resources::new(efuse, km, spacc, pke, trng),
            }
        }
    }

    /// Opaque U1 initialization failure.
    ///
    /// U6 replaces this preview with the shared actionable error taxonomy.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct InitError {
        _private: (),
    }

    impl InitError {
        const fn new() -> Self {
            Self { _private: () }
        }
    }

    /// Synchronous WS63 acceptance stage for one BLE GAP command.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum BleOperationErrorKind {
        /// The asynchronous BLE host enable operation failed.
        Enable,
        /// The controller rejected the advertising payload.
        SetAdvertisingData,
        /// The controller rejected advertising timing or channel parameters.
        SetAdvertisingParameters,
        /// The controller rejected the start-advertising request.
        StartAdvertising,
        /// The controller rejected scan timing or mode parameters.
        SetScanParameters,
        /// The controller rejected the start-scan request.
        StartScanning,
        /// The controller rejected the advertising stop request.
        StopAdvertising,
        /// The controller rejected the scan stop request.
        StopScanning,
        /// Another generation still owns this lifecycle.
        LifecycleBusy,
        /// A late callback or handle no longer names the active generation.
        StaleLifecycle,
        /// The static GATT database could not be registered.
        GattDatabase,
        /// The BLE host rejected the typed security policy.
        ConfigureSecurity,
        /// The BLE host rejected a pairing request.
        Pair,
        /// Link authentication failed after pairing was accepted.
        Authentication,
        /// The BLE host could not report the peer pairing state.
        QueryPairingState,
        /// The BLE host could not remove the stored peer relationship.
        RemoveBond,
        /// The BLE host returned an unreviewed pairing-state value.
        UnknownPairingState,
        /// The selected WS63 profile cannot represent this valid generic config.
        UnsupportedConfiguration,
        /// This operation is unavailable in a host-only build.
        UnsupportedTarget,
    }

    /// Fail-closed BLE command rejection with an optional vendor status.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct BleOperationError {
        kind: BleOperationErrorKind,
        vendor_status: Option<u32>,
    }

    impl BleOperationError {
        /// Return the operation stage that rejected the request.
        pub const fn kind(&self) -> BleOperationErrorKind {
            self.kind
        }

        /// Return the raw vendor status when rejection came from the WS63 host.
        pub const fn vendor_status(&self) -> Option<u32> {
            self.vendor_status
        }
    }

    /// Command accepted or rejected synchronously by the WS63 BLE host.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum BleOperation {
        /// The WS63 host accepted the start-advertising request.
        AdvertisingRequested,
        /// The WS63 host accepted the start-scan request.
        ScanningRequested,
        /// The static GATT database was registered and started.
        GattServerRegistered(GattServerHandle),
        /// The typed BLE security policy was accepted.
        SecurityConfigured,
        /// Pairing was accepted for asynchronous processing.
        PairingRequested,
        /// Current state of the requested peer relationship.
        PairingState(crate::ble::PairingState),
        /// The stored peer relationship was removed.
        BondRemoved,
    }

    /// Active advertising ownership returned by the BLE event plane.
    #[must_use = "call stop().await or retain the advertiser; dropping it requests cleanup"]
    pub struct Advertiser {
        operation: crate::ProtocolCommandId,
        inner: hisi_rf_core::control::LifecycleGuard<BleOperationError>,
    }

    impl Advertiser {
        /// Command that created this advertising generation.
        pub const fn operation(&self) -> crate::ProtocolCommandId {
            self.operation
        }

        /// Stop advertising and wait for the matching backend callback.
        pub async fn stop(self) -> Result<(), BleOperationError> {
            map_ble_stop_result(self.inner.stop().await)
        }
    }

    impl core::fmt::Debug for Advertiser {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter
                .debug_struct("Advertiser")
                .field("operation", &self.operation)
                .finish_non_exhaustive()
        }
    }

    /// Active scan ownership returned by the BLE event plane.
    #[must_use = "call stop().await or retain the scanner; dropping it requests cleanup"]
    pub struct Scanner {
        operation: crate::ProtocolCommandId,
        inner: hisi_rf_core::control::LifecycleGuard<BleOperationError>,
    }

    impl Scanner {
        /// Command that created this scanning generation.
        pub const fn operation(&self) -> crate::ProtocolCommandId {
            self.operation
        }

        /// Stop scanning and wait for the runner's synchronous backend result.
        pub async fn stop(self) -> Result<(), BleOperationError> {
            map_ble_stop_result(self.inner.stop().await)
        }
    }

    impl core::fmt::Debug for Scanner {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter
                .debug_struct("Scanner")
                .field("operation", &self.operation)
                .finish_non_exhaustive()
        }
    }

    fn map_ble_stop_result(
        result: Result<(), hisi_rf_core::control::LifecycleStopError<BleOperationError>>,
    ) -> Result<(), BleOperationError> {
        result.map_err(|error| match error {
            hisi_rf_core::control::LifecycleStopError::Stale => BleOperationError {
                kind: BleOperationErrorKind::StaleLifecycle,
                vendor_status: None,
            },
            hisi_rf_core::control::LifecycleStopError::Backend(error) => error,
        })
    }

    /// Facade-owned BLE lifecycle event copied out of vendor callback context.
    #[derive(Debug)]
    pub enum BleEvent {
        /// Advertising entered the active state for this command generation.
        AdvertisingStarted {
            /// Unique active advertising capability.
            advertiser: Advertiser,
        },
        /// Scanning became active for this command generation.
        ScanReady {
            /// Unique active scan capability.
            scanner: Scanner,
        },
        /// Pairing reached a terminal result for the copied peer identity.
        PairingComplete {
            /// Validated peer address copied from the vendor callback.
            peer: crate::ble::BluetoothAddress,
            /// Success or the exact backend status mapped to the pairing stage.
            result: Result<(), BleOperationError>,
        },
        /// Authentication completed without exposing long-term key bytes.
        AuthenticationComplete {
            /// Validated peer address copied from the vendor callback.
            peer: crate::ble::BluetoothAddress,
            /// Whether the backend reported bond material for internal storage.
            key_material_present: bool,
            /// Success or the exact backend status mapped to the authentication stage.
            result: Result<(), BleOperationError>,
        },
        /// An asynchronous callback reported a backend failure.
        BackendError {
            /// Correlated command when the runner still owns its lifecycle.
            operation: Option<crate::ProtocolCommandId>,
            /// Stable facade stage identifier.
            stage: u8,
            /// Lossless vendor status.
            status: u32,
        },
    }

    struct BleLifecycle {
        operation: Option<crate::ProtocolCommandId>,
        generation: Option<hisi_rf_core::control::LifecycleId>,
        stopping: Option<hisi_rf_core::control::LifecycleId>,
        runner: hisi_rf_core::control::LifecycleRunner<BleOperationError>,
    }

    struct BleLifecycles {
        advertising: BleLifecycle,
        scanning: BleLifecycle,
    }

    impl BleLifecycle {
        fn new(runner: hisi_rf_core::control::LifecycleRunner<BleOperationError>) -> Self {
            Self {
                operation: None,
                generation: None,
                stopping: None,
                runner,
            }
        }

        fn clear(&mut self) {
            self.operation = None;
            self.generation = None;
            self.stopping = None;
        }
    }

    impl BleLifecycles {
        fn new(
            advertising: hisi_rf_core::control::LifecycleRunner<BleOperationError>,
            scanning: hisi_rf_core::control::LifecycleRunner<BleOperationError>,
        ) -> Self {
            Self {
                advertising: BleLifecycle::new(advertising),
                scanning: BleLifecycle::new(scanning),
            }
        }
    }

    /// Unforgeable facade handle for one registered static GATT server.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct GattServerHandle {
        server_id: u8,
        service_handle: u16,
        value_handle: u16,
        ccc_handle: u16,
    }

    trait BleBackend {
        fn command_ready(&self) -> Result<bool, BleOperationError>;
        fn start_advertising(
            &mut self,
            config: crate::ble::AdvertisingConfig,
        ) -> Result<(), BleOperationError>;
        fn start_scanning(
            &mut self,
            config: crate::ble::ScanConfig,
        ) -> Result<(), BleOperationError>;
        fn stop_advertising(&mut self) -> Result<(), BleOperationError>;
        fn stop_scanning(&mut self) -> Result<(), BleOperationError>;
        fn register_gatt_server(
            &mut self,
            definition: crate::ble::GattServerDefinition,
        ) -> Result<GattServerHandle, BleOperationError>;
        fn configure_security(
            &mut self,
            config: crate::ble::SecurityConfig,
        ) -> Result<(), BleOperationError>;
        fn pair(&mut self, peer: crate::ble::BluetoothAddress) -> Result<(), BleOperationError>;
        fn pairing_state(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<crate::ble::PairingState, BleOperationError>;
        fn remove_bond(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<(), BleOperationError>;
    }

    impl BleBackend for hisi_rf_ws63::BleB1Controller {
        fn command_ready(&self) -> Result<bool, BleOperationError> {
            match self.enable_status() {
                None => Ok(false),
                Some(0) => Ok(true),
                Some(status) => Err(BleOperationError {
                    kind: BleOperationErrorKind::Enable,
                    vendor_status: Some(status),
                }),
            }
        }

        fn start_advertising(
            &mut self,
            config: crate::ble::AdvertisingConfig,
        ) -> Result<(), BleOperationError> {
            self.start_advertising_config(config).map_err(map_ble_error)
        }

        fn start_scanning(
            &mut self,
            config: crate::ble::ScanConfig,
        ) -> Result<(), BleOperationError> {
            self.start_scanning_config(config).map_err(map_ble_error)
        }

        fn stop_advertising(&mut self) -> Result<(), BleOperationError> {
            self.stop_advertising().map_err(map_ble_error)
        }

        fn stop_scanning(&mut self) -> Result<(), BleOperationError> {
            self.stop_scanning().map_err(map_ble_error)
        }

        fn register_gatt_server(
            &mut self,
            definition: crate::ble::GattServerDefinition,
        ) -> Result<GattServerHandle, BleOperationError> {
            self.register_gatt_server_definition(definition)
                .map(|handles| GattServerHandle {
                    server_id: handles.server_id,
                    service_handle: handles.service_handle,
                    value_handle: handles.value_handle,
                    ccc_handle: handles.ccc_handle,
                })
                .map_err(map_ble_gatt_error)
        }

        fn configure_security(
            &mut self,
            config: crate::ble::SecurityConfig,
        ) -> Result<(), BleOperationError> {
            self.configure_security(config)
                .map_err(map_ble_security_error)
        }

        fn pair(&mut self, peer: crate::ble::BluetoothAddress) -> Result<(), BleOperationError> {
            self.pair(peer).map_err(map_ble_security_error)
        }

        fn pairing_state(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<crate::ble::PairingState, BleOperationError> {
            self.pairing_state(peer).map_err(map_ble_security_error)
        }

        fn remove_bond(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<(), BleOperationError> {
            self.remove_bond(peer).map_err(map_ble_security_error)
        }
    }

    fn map_ble_error(error: hisi_rf_ws63::BleB2Error) -> BleOperationError {
        use hisi_rf_ws63::BleB2Error as E;
        let (kind, vendor_status) = match error {
            E::AdvertisingDataTooLong { .. } => {
                (BleOperationErrorKind::UnsupportedConfiguration, None)
            }
            E::SetAdvertisingData(status) => {
                (BleOperationErrorKind::SetAdvertisingData, Some(status))
            }
            E::SetAdvertisingParameters(status) => (
                BleOperationErrorKind::SetAdvertisingParameters,
                Some(status),
            ),
            E::StartAdvertising(status) => (BleOperationErrorKind::StartAdvertising, Some(status)),
            E::SetScanParameters(status) => {
                (BleOperationErrorKind::SetScanParameters, Some(status))
            }
            E::StartScanning(status) => (BleOperationErrorKind::StartScanning, Some(status)),
            E::StopAdvertising(status) => (BleOperationErrorKind::StopAdvertising, Some(status)),
            E::StopScanning(status) => (BleOperationErrorKind::StopScanning, Some(status)),
            E::DuplicateFilteringUnsupported => {
                (BleOperationErrorKind::UnsupportedConfiguration, None)
            }
            E::UnsupportedTarget => (BleOperationErrorKind::UnsupportedTarget, None),
        };
        BleOperationError {
            kind,
            vendor_status,
        }
    }

    fn map_ble_gatt_error(error: hisi_rf_ws63::BleB3Error) -> BleOperationError {
        use hisi_rf_ws63::BleB3Error as E;
        let (kind, vendor_status) = match error {
            E::UnsupportedDatabase | E::ValueTooLong { .. } => {
                (BleOperationErrorKind::UnsupportedConfiguration, None)
            }
            E::UnsupportedTarget => (BleOperationErrorKind::UnsupportedTarget, None),
            E::RegisterServer(status)
            | E::StopScanning(status)
            | E::AddService(status)
            | E::AddCharacteristic(status)
            | E::AddDescriptor(status)
            | E::StartService(status)
            | E::RegisterClient(status)
            | E::Connect(status)
            | E::NotifyOrIndicate(status)
            | E::DiscoverService(status)
            | E::DiscoverCharacteristic(status)
            | E::DiscoverDescriptor(status)
            | E::Write(status)
            | E::Disconnect(status) => (BleOperationErrorKind::GattDatabase, Some(status)),
        };
        BleOperationError {
            kind,
            vendor_status,
        }
    }

    fn map_ble_security_error(error: hisi_rf_ws63::BleSecurityError) -> BleOperationError {
        use hisi_rf_ws63::BleSecurityError as E;
        let (kind, vendor_status) = match error {
            E::Configure(status) => (BleOperationErrorKind::ConfigureSecurity, Some(status)),
            E::Pair(status) => (BleOperationErrorKind::Pair, Some(status)),
            E::Query(status) => (BleOperationErrorKind::QueryPairingState, Some(status)),
            E::UnknownPairingState(status) => {
                (BleOperationErrorKind::UnknownPairingState, Some(status))
            }
            E::RemoveBond(status) => (BleOperationErrorKind::RemoveBond, Some(status)),
            E::UnsupportedTarget => (BleOperationErrorKind::UnsupportedTarget, None),
        };
        BleOperationError {
            kind,
            vendor_status,
        }
    }

    fn run_ble_once(
        receiver: &mut hisi_rf_core::control::ControlReceiver<
            BleCommand,
            Result<BleOperation, BleOperationError>,
        >,
        backend: &mut impl BleBackend,
        lifecycles: &mut BleLifecycles,
    ) -> Result<bool, crate::ProtocolError> {
        let readiness = backend.command_ready();
        if matches!(readiness, Ok(false)) {
            return Ok(false);
        }
        let Some(command) = receiver.try_take_command() else {
            return Ok(false);
        };
        let id = command.id();
        let command = command.into_inner();
        let result = match readiness {
            Ok(true) => match command {
                BleCommand::StartAdvertising(config) => {
                    match lifecycles.advertising.runner.begin() {
                        Ok(generation) => match backend.start_advertising(config) {
                            Ok(()) => {
                                lifecycles.advertising.operation =
                                    Some(crate::ProtocolCommandId(id));
                                lifecycles.advertising.generation = Some(generation);
                                Ok(BleOperation::AdvertisingRequested)
                            }
                            Err(error) => {
                                let _ = lifecycles.advertising.runner.abort_start(generation);
                                Err(error)
                            }
                        },
                        Err(_) => Err(BleOperationError {
                            kind: BleOperationErrorKind::LifecycleBusy,
                            vendor_status: None,
                        }),
                    }
                }
                BleCommand::StartScanning(config) => match lifecycles.scanning.runner.begin() {
                    Ok(generation) => match backend.start_scanning(config) {
                        Ok(()) => {
                            lifecycles.scanning.operation = Some(crate::ProtocolCommandId(id));
                            lifecycles.scanning.generation = Some(generation);
                            Ok(BleOperation::ScanningRequested)
                        }
                        Err(error) => {
                            let _ = lifecycles.scanning.runner.abort_start(generation);
                            Err(error)
                        }
                    },
                    Err(_) => Err(BleOperationError {
                        kind: BleOperationErrorKind::LifecycleBusy,
                        vendor_status: None,
                    }),
                },
                BleCommand::RegisterGattServer(definition) => backend
                    .register_gatt_server(definition)
                    .map(BleOperation::GattServerRegistered),
                BleCommand::ConfigureSecurity(config) => backend
                    .configure_security(config)
                    .map(|()| BleOperation::SecurityConfigured),
                BleCommand::Pair(peer) => {
                    backend.pair(peer).map(|()| BleOperation::PairingRequested)
                }
                BleCommand::QueryPairingState(peer) => {
                    backend.pairing_state(peer).map(BleOperation::PairingState)
                }
                BleCommand::RemoveBond(peer) => backend
                    .remove_bond(peer)
                    .map(|()| BleOperation::BondRemoved),
            },
            Err(error) => Err(error),
            Ok(false) => unreachable!(),
        };
        receiver
            .complete(id, result)
            .map_err(|_| crate::ProtocolError::CompletionOwnership)?;
        Ok(true)
    }

    fn map_ble_lifecycle_event(
        event: hisi_rf_ws63::BleB2Event,
        lifecycles: &mut BleLifecycles,
    ) -> Option<BleEvent> {
        match event {
            hisi_rf_ws63::BleB2Event::AdvertisingState { status: 1, .. } => {
                let operation = lifecycles.advertising.operation?;
                let generation = lifecycles.advertising.generation?;
                lifecycles
                    .advertising
                    .runner
                    .activate(generation)
                    .ok()
                    .map(|inner| BleEvent::AdvertisingStarted {
                        advertiser: Advertiser { operation, inner },
                    })
            }
            hisi_rf_ws63::BleB2Event::AdvertisingState { status, .. } => {
                if let Some(generation) = lifecycles.advertising.generation {
                    let _ = lifecycles.advertising.runner.abort_start(generation);
                }
                let operation = lifecycles.advertising.operation;
                lifecycles.advertising.clear();
                Some(BleEvent::BackendError {
                    operation,
                    stage: 1,
                    status,
                })
            }
            hisi_rf_ws63::BleB2Event::AdvertisingStopped { status, .. } => {
                let generation = lifecycles.advertising.stopping.take()?;
                let result = if status == 0 {
                    Ok(())
                } else {
                    Err(BleOperationError {
                        kind: BleOperationErrorKind::StopAdvertising,
                        vendor_status: Some(status),
                    })
                };
                let _ = lifecycles.advertising.runner.finish(generation, result);
                lifecycles.advertising.clear();
                None
            }
            hisi_rf_ws63::BleB2Event::ScanParameters { status: 0 } => {
                let operation = lifecycles.scanning.operation?;
                let generation = lifecycles.scanning.generation?;
                lifecycles
                    .scanning
                    .runner
                    .activate(generation)
                    .ok()
                    .map(|inner| BleEvent::ScanReady {
                        scanner: Scanner { operation, inner },
                    })
            }
            hisi_rf_ws63::BleB2Event::ScanParameters { status } => {
                if let Some(generation) = lifecycles.scanning.generation {
                    let _ = lifecycles.scanning.runner.abort_start(generation);
                }
                let operation = lifecycles.scanning.operation;
                lifecycles.scanning.clear();
                Some(BleEvent::BackendError {
                    operation,
                    stage: 2,
                    status,
                })
            }
            hisi_rf_ws63::BleB2Event::Enabled { status } if status != 0 => {
                Some(BleEvent::BackendError {
                    operation: None,
                    stage: 0,
                    status,
                })
            }
            hisi_rf_ws63::BleB2Event::PairingComplete {
                address,
                address_type,
                status,
                ..
            } => {
                let Some(peer) = map_ble_peer(address, address_type) else {
                    return Some(BleEvent::BackendError {
                        operation: None,
                        stage: 5,
                        status: u32::MAX,
                    });
                };
                Some(BleEvent::PairingComplete {
                    peer,
                    result: ble_status_result(BleOperationErrorKind::Pair, status),
                })
            }
            hisi_rf_ws63::BleB2Event::AuthenticationComplete {
                address,
                address_type,
                status,
                key_material_present,
                ..
            } => {
                let Some(peer) = map_ble_peer(address, address_type) else {
                    return Some(BleEvent::BackendError {
                        operation: None,
                        stage: 6,
                        status: u32::MAX,
                    });
                };
                Some(BleEvent::AuthenticationComplete {
                    peer,
                    key_material_present,
                    result: ble_status_result(BleOperationErrorKind::Authentication, status),
                })
            }
            _ => None,
        }
    }

    fn map_ble_peer(address: [u8; 6], address_type: u8) -> Option<crate::ble::BluetoothAddress> {
        match address_type {
            0 => crate::ble::BluetoothAddress::public(address),
            1 => crate::ble::BluetoothAddress::random_static(address),
            _ => None,
        }
    }

    fn ble_status_result(
        kind: BleOperationErrorKind,
        status: u32,
    ) -> Result<(), BleOperationError> {
        if status == 0 {
            Ok(())
        } else {
            Err(BleOperationError {
                kind,
                vendor_status: Some(status),
            })
        }
    }

    fn run_ble_cancellation_once(
        backend: &mut impl BleBackend,
        lifecycles: &mut BleLifecycles,
    ) -> bool {
        if let Some(generation) = lifecycles.advertising.runner.try_take_cancel() {
            match backend.stop_advertising() {
                Ok(()) => lifecycles.advertising.stopping = Some(generation),
                Err(error) => {
                    let _ = lifecycles.advertising.runner.finish(generation, Err(error));
                    lifecycles.advertising.clear();
                }
            }
            return true;
        }
        if let Some(generation) = lifecycles.scanning.runner.try_take_cancel() {
            let result = backend.stop_scanning();
            let _ = lifecycles.scanning.runner.finish(generation, result);
            lifecycles.scanning.clear();
            return true;
        }
        false
    }

    /// Exclusive BLE composition before task ownership is split.
    pub struct RadioController {
        inner: hisi_rf_ws63::BleB1Controller,
        sender: hisi_rf_core::control::ControlSender<
            BleCommand,
            Result<BleOperation, BleOperationError>,
        >,
        receiver: hisi_rf_core::control::ControlReceiver<
            BleCommand,
            Result<BleOperation, BleOperationError>,
        >,
        event_producer: hisi_rf_core::control::EventProducer<BleEvent, { crate::EVENT_CAPACITY }>,
        event_consumer: hisi_rf_core::control::EventConsumer<BleEvent, { crate::EVENT_CAPACITY }>,
        advertising: hisi_rf_core::control::LifecycleRunner<BleOperationError>,
        scanning: hisi_rf_core::control::LifecycleRunner<BleOperationError>,
    }

    impl RadioController {
        /// Split the facade into the BLE handle and mandatory runner owner.
        pub fn split(self) -> RadioParts {
            RadioParts {
                ble: BleController {
                    sender: self.sender,
                    events: self.event_consumer,
                },
                runner: RadioRunner {
                    inner: self.inner,
                    receiver: self.receiver,
                    events: self.event_producer,
                    lifecycles: BleLifecycles::new(self.advertising, self.scanning),
                },
            }
        }
    }

    /// BLE protocol handle reserved for the U2-U4 typed control plane.
    pub struct BleController {
        sender: hisi_rf_core::control::ControlSender<
            BleCommand,
            Result<BleOperation, BleOperationError>,
        >,
        events: hisi_rf_core::control::EventConsumer<BleEvent, { crate::EVENT_CAPACITY }>,
    }

    impl BleController {
        /// Queue a typed advertising request without blocking the caller.
        ///
        /// A busy mailbox returns ownership of `config`. Success only means the
        /// runner accepted the command; call [`Self::try_take_completion`] for
        /// the synchronous WS63 host result.
        pub fn try_start_advertising(
            &mut self,
            config: crate::ble::AdvertisingConfig,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::AdvertisingConfig>>
        {
            self.sender
                .try_submit(BleCommand::StartAdvertising(config))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::StartAdvertising(config) => crate::ProtocolBusy { request: config },
                    _ => unreachable!(),
                })
        }

        /// Queue a typed scan request without blocking the caller.
        ///
        /// A busy mailbox returns ownership of `config`. Success only means the
        /// runner accepted the command; call [`Self::try_take_completion`] for
        /// the synchronous WS63 host result.
        pub fn try_start_scanning(
            &mut self,
            config: crate::ble::ScanConfig,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::ScanConfig>> {
            self.sender
                .try_submit(BleCommand::StartScanning(config))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::StartScanning(config) => crate::ProtocolBusy { request: config },
                    _ => unreachable!(),
                })
        }

        /// Queue registration of one validated static GATT database.
        pub fn try_register_gatt_server(
            &mut self,
            definition: crate::ble::GattServerDefinition,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::GattServerDefinition>>
        {
            self.sender
                .try_submit(BleCommand::RegisterGattServer(definition))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::RegisterGattServer(definition) => crate::ProtocolBusy {
                        request: definition,
                    },
                    _ => unreachable!(),
                })
        }

        /// Queue an explicit BLE pairing policy.
        pub fn try_configure_security(
            &mut self,
            config: crate::ble::SecurityConfig,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::SecurityConfig>>
        {
            self.sender
                .try_submit(BleCommand::ConfigureSecurity(config))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::ConfigureSecurity(config) => {
                        crate::ProtocolBusy { request: config }
                    }
                    _ => unreachable!(),
                })
        }

        /// Queue pairing with one validated peer.
        pub fn try_pair(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::BluetoothAddress>>
        {
            self.sender
                .try_submit(BleCommand::Pair(peer))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::Pair(peer) => crate::ProtocolBusy { request: peer },
                    _ => unreachable!(),
                })
        }

        /// Queue a pairing-state query for one validated peer.
        pub fn try_query_pairing_state(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::BluetoothAddress>>
        {
            self.sender
                .try_submit(BleCommand::QueryPairingState(peer))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::QueryPairingState(peer) => crate::ProtocolBusy { request: peer },
                    _ => unreachable!(),
                })
        }

        /// Queue removal of one stored peer relationship.
        pub fn try_remove_bond(
            &mut self,
            peer: crate::ble::BluetoothAddress,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::ble::BluetoothAddress>>
        {
            self.sender
                .try_submit(BleCommand::RemoveBond(peer))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    BleCommand::RemoveBond(peer) => crate::ProtocolBusy { request: peer },
                    _ => unreachable!(),
                })
        }

        /// Take the terminal result for the current command, if available.
        ///
        /// The operation variants report synchronous request acceptance, not
        /// an on-air advertising or scanning lifecycle transition.
        pub fn try_take_completion(
            &mut self,
        ) -> Result<
            Option<crate::ProtocolCompletion<BleOperation, BleOperationError>>,
            crate::ProtocolError,
        > {
            self.sender
                .try_take_completion()
                .map(|completion| {
                    completion.map(|completion| crate::ProtocolCompletion {
                        id: crate::ProtocolCommandId(completion.id()),
                        result: completion.into_inner(),
                    })
                })
                .map_err(|_| crate::ProtocolError::StaleCompletion)
        }

        /// Take the oldest unsolicited lifecycle event without waiting.
        pub fn try_next_event(&mut self) -> Option<BleEvent> {
            self.events.try_next_event()
        }

        /// Wait for the oldest unsolicited lifecycle event.
        pub async fn next_event(&mut self) -> BleEvent {
            self.events.next_event().await
        }

        /// Snapshot public event conservation counters.
        pub fn event_diagnostics(&self) -> crate::ProtocolEventDiagnostics {
            crate::protocol_event_diagnostics(self.events.diagnostics())
        }
    }

    /// Parts enabled by the BLE-only compile-time profile.
    pub struct RadioParts {
        /// BLE control-plane capability.
        pub ble: BleController,
        /// Mandatory owner of the WS63 controller/host runtime.
        pub runner: RadioRunner,
    }

    /// Unique process-lifetime owner of the internal BLE stage controller.
    pub struct RadioRunner {
        inner: hisi_rf_ws63::BleB1Controller,
        receiver: hisi_rf_core::control::ControlReceiver<
            BleCommand,
            Result<BleOperation, BleOperationError>,
        >,
        events: hisi_rf_core::control::EventProducer<BleEvent, { crate::EVENT_CAPACITY }>,
        lifecycles: BleLifecycles,
    }

    impl RadioRunner {
        /// Number of copied vendor events rejected by the bounded backend queue.
        pub fn dropped_events(&self) -> u32 {
            self.inner.dropped_events()
        }

        /// Execute at most one queued BLE command and publish its completion.
        ///
        /// Returns `Ok(false)` when no command is pending or the asynchronous
        /// BLE enable callback has not arrived yet. The queued command retains
        /// ownership in both cases. Applications should call this from their
        /// single long-lived radio runner task.
        pub fn run_once(&mut self) -> Result<bool, crate::ProtocolError> {
            if run_ble_cancellation_once(&mut self.inner, &mut self.lifecycles) {
                return Ok(true);
            }
            run_ble_once(&mut self.receiver, &mut self.inner, &mut self.lifecycles)
        }

        /// Copy at most one backend lifecycle event into the public queue.
        ///
        /// Command completions and unsolicited events use independent storage;
        /// a full event queue never consumes a completion.
        pub fn run_event_once(&mut self) -> bool {
            let Some(event) = self.inner.next_event() else {
                return false;
            };
            let event = map_ble_lifecycle_event(event, &mut self.lifecycles);
            if let Some(event) = event {
                let _ = self.events.try_publish(event);
            }
            true
        }

        /// Consume one backend lifecycle event for the temporary U2 HIL gate.
        #[cfg(feature = "u2-hil-diagnostics")]
        #[doc(hidden)]
        pub fn next_hil_event(&mut self) -> Option<U2HilEvent> {
            while let Some(event) = self.inner.next_event() {
                use hisi_rf_ws63::BleB2Event as E;
                let mapped = match event {
                    E::AdvertisingData { status, .. } if status != 0 => {
                        Some(U2HilEvent::BackendError { stage: 1, status })
                    }
                    E::AdvertisingParameters { status, .. } if status != 0 => {
                        Some(U2HilEvent::BackendError { stage: 2, status })
                    }
                    E::AdvertisingState { status: 1, .. } => Some(U2HilEvent::AdvertisingStarted),
                    E::ScanParameters { status: 0 } => Some(U2HilEvent::ScanReady),
                    E::ScanParameters { status } => {
                        Some(U2HilEvent::BackendError { stage: 3, status })
                    }
                    E::ScanResult { data_len, data, .. }
                        if data[..usize::from(data_len).min(data.len())]
                            == *U2_HIL_PEER_PAYLOAD =>
                    {
                        Some(U2HilEvent::PeerObserved)
                    }
                    E::Enabled { status } if status != 0 => {
                        Some(U2HilEvent::BackendError { stage: 0, status })
                    }
                    _ => None,
                };
                if mapped.is_some() {
                    return mapped;
                }
            }
            None
        }
    }

    /// Initialize the BLE-only WS63 composition.
    pub fn init(
        resources: Resources,
        storage: InstalledRadioStorage,
    ) -> Result<RadioController, InitError> {
        let (sender, receiver) = storage.control.claim().ok_or_else(InitError::new)?;
        let (event_producer, event_consumer) = storage.events.claim().ok_or_else(InitError::new)?;
        let advertising = storage.advertising.claim().ok_or_else(InitError::new)?;
        let scanning = storage.scanning.claim().ok_or_else(InitError::new)?;
        hisi_rf_ws63::init_ble_b1(resources.inner, storage.inner)
            .map(|inner| RadioController {
                inner,
                sender,
                receiver,
                event_producer,
                event_consumer,
                advertising,
                scanning,
            })
            .map_err(|_| InitError::new())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::future::Future;
        use std::boxed::Box;
        use std::task::{Context, Poll, Waker};

        struct FakeBackend {
            advertising: usize,
            scanning: usize,
            advertising_stops: usize,
            scanning_stops: usize,
            gatt_servers: usize,
            security_configurations: usize,
            pairing_requests: usize,
            pairing_queries: usize,
            bond_removals: usize,
            pairing_state: crate::ble::PairingState,
            ready: bool,
            enable_error: Option<u32>,
            reject_scanning: bool,
        }

        impl Default for FakeBackend {
            fn default() -> Self {
                Self {
                    advertising: 0,
                    scanning: 0,
                    advertising_stops: 0,
                    scanning_stops: 0,
                    gatt_servers: 0,
                    security_configurations: 0,
                    pairing_requests: 0,
                    pairing_queries: 0,
                    bond_removals: 0,
                    pairing_state: crate::ble::PairingState::NotPaired,
                    ready: false,
                    enable_error: None,
                    reject_scanning: false,
                }
            }
        }

        impl BleBackend for FakeBackend {
            fn command_ready(&self) -> Result<bool, BleOperationError> {
                match self.enable_error {
                    Some(status) => Err(BleOperationError {
                        kind: BleOperationErrorKind::Enable,
                        vendor_status: Some(status),
                    }),
                    None => Ok(self.ready),
                }
            }

            fn start_advertising(
                &mut self,
                _: crate::ble::AdvertisingConfig,
            ) -> Result<(), BleOperationError> {
                self.advertising += 1;
                Ok(())
            }

            fn start_scanning(
                &mut self,
                _: crate::ble::ScanConfig,
            ) -> Result<(), BleOperationError> {
                self.scanning += 1;
                if self.reject_scanning {
                    Err(BleOperationError {
                        kind: BleOperationErrorKind::StartScanning,
                        vendor_status: Some(0x1234),
                    })
                } else {
                    Ok(())
                }
            }

            fn stop_advertising(&mut self) -> Result<(), BleOperationError> {
                self.advertising_stops += 1;
                Ok(())
            }

            fn stop_scanning(&mut self) -> Result<(), BleOperationError> {
                self.scanning_stops += 1;
                Ok(())
            }

            fn register_gatt_server(
                &mut self,
                _: crate::ble::GattServerDefinition,
            ) -> Result<GattServerHandle, BleOperationError> {
                self.gatt_servers += 1;
                Ok(GattServerHandle {
                    server_id: 1,
                    service_handle: 2,
                    value_handle: 3,
                    ccc_handle: 4,
                })
            }

            fn configure_security(
                &mut self,
                _: crate::ble::SecurityConfig,
            ) -> Result<(), BleOperationError> {
                self.security_configurations += 1;
                Ok(())
            }

            fn pair(&mut self, _: crate::ble::BluetoothAddress) -> Result<(), BleOperationError> {
                self.pairing_requests += 1;
                Ok(())
            }

            fn pairing_state(
                &mut self,
                _: crate::ble::BluetoothAddress,
            ) -> Result<crate::ble::PairingState, BleOperationError> {
                self.pairing_queries += 1;
                Ok(self.pairing_state)
            }

            fn remove_bond(
                &mut self,
                _: crate::ble::BluetoothAddress,
            ) -> Result<(), BleOperationError> {
                self.bond_removals += 1;
                Ok(())
            }
        }

        fn ble_lifecycles() -> BleLifecycles {
            let advertising = Box::leak(Box::new(BleLifecycleState::new()))
                .claim()
                .unwrap();
            let scanning = Box::leak(Box::new(BleLifecycleState::new()))
                .claim()
                .unwrap();
            BleLifecycles::new(advertising, scanning)
        }

        fn advertising() -> crate::ble::AdvertisingConfig {
            let interval = crate::ble::AdvertisingInterval::try_from_units(0x20).unwrap();
            crate::ble::AdvertisingConfig::new(
                crate::ble::AdvertisingTiming::try_new(interval, interval).unwrap(),
                crate::ble::AdvertisingChannels::ALL,
                crate::ble::AdvertisingPayload::try_from_slice(b"facade").unwrap(),
            )
        }

        fn scanning() -> crate::ble::ScanConfig {
            let interval = crate::ble::ScanInterval::try_from_units(0x20).unwrap();
            crate::ble::ScanConfig::new(
                crate::ble::ScanTiming::try_new(interval, interval).unwrap(),
                crate::ble::ScanMode::Passive,
                false,
            )
        }

        fn gatt_database() -> crate::ble::GattServerDefinition {
            const CCC: crate::ble::GattDescriptorDefinition =
                crate::ble::GattDescriptorDefinition::try_new(
                    crate::ble::GattUuid::Uuid16(0x2902),
                    crate::ble::GattPermissions::READ.union(crate::ble::GattPermissions::WRITE),
                    &[0, 0],
                    2,
                )
                .unwrap();
            const CHARACTERISTIC: crate::ble::GattCharacteristicDefinition =
                crate::ble::GattCharacteristicDefinition::try_new(
                    crate::ble::GattUuid::Uuid16(0xcdef),
                    crate::ble::GattPermissions::READ.union(crate::ble::GattPermissions::WRITE),
                    crate::ble::GattProperties::READ.union(crate::ble::GattProperties::NOTIFY),
                    b"U3",
                    8,
                    &[CCC],
                )
                .unwrap();
            const SERVICE: crate::ble::GattServiceDefinition =
                crate::ble::GattServiceDefinition::try_new(
                    crate::ble::GattUuid::Uuid16(0xabcd),
                    true,
                    &[CHARACTERISTIC],
                )
                .unwrap();
            crate::ble::GattServerDefinition::try_new(
                crate::ble::GattUuid::Uuid16(0xb301),
                &[SERVICE],
            )
            .unwrap()
        }

        #[test]
        fn bounded_controller_and_runner_preserve_ble_command_ownership() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            let mut lifecycles = ble_lifecycles();
            let id = controller.try_start_advertising(advertising()).unwrap();
            let rejected = controller.try_start_scanning(scanning()).unwrap_err();
            assert_eq!(rejected.into_inner(), scanning());

            let mut backend = FakeBackend::default();
            assert!(!run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert!(controller.try_take_completion().unwrap().is_none());
            backend.ready = true;
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(backend.advertising, 1);
            assert_eq!(backend.scanning, 0);
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), id);
            assert_eq!(
                completion.into_result(),
                Ok(BleOperation::AdvertisingRequested)
            );
            let event = map_ble_lifecycle_event(
                hisi_rf_ws63::BleB2Event::AdvertisingState {
                    adv_id: 0,
                    status: 1,
                },
                &mut lifecycles,
            )
            .unwrap();
            producer.try_publish(event).unwrap();
            let BleEvent::AdvertisingStarted { advertiser } = controller.try_next_event().unwrap()
            else {
                panic!("expected advertising lifecycle guard");
            };
            assert_eq!(advertiser.operation(), id);
            drop(advertiser);
            assert!(run_ble_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.advertising_stops, 1);
            assert!(
                map_ble_lifecycle_event(
                    hisi_rf_ws63::BleB2Event::AdvertisingStopped {
                        adv_id: 0,
                        status: 0,
                    },
                    &mut lifecycles,
                )
                .is_none()
            );
            assert_eq!(
                controller.event_diagnostics(),
                crate::ProtocolEventDiagnostics {
                    accepted: 1,
                    consumed: 1,
                    dropped: 0,
                    pending: 0,
                    high_water: 1,
                }
            );

            backend.reject_scanning = true;
            let scan_id = controller.try_start_scanning(scanning()).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), scan_id);
            let error = completion.into_result().unwrap_err();
            assert_eq!(error.kind(), BleOperationErrorKind::StartScanning);
            assert_eq!(error.vendor_status(), Some(0x1234));
            assert!(!run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());

            backend.enable_error = Some(0x4321);
            let enable_id = controller.try_start_advertising(advertising()).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(backend.advertising, 1);
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), enable_id);
            let error = completion.into_result().unwrap_err();
            assert_eq!(error.kind(), BleOperationErrorKind::Enable);
            assert_eq!(error.vendor_status(), Some(0x4321));
        }

        #[test]
        fn full_ble_event_queue_does_not_consume_command_completion() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            for _ in 0..crate::EVENT_CAPACITY {
                producer
                    .try_publish(BleEvent::BackendError {
                        operation: None,
                        stage: 0,
                        status: 1,
                    })
                    .unwrap();
            }
            assert!(
                producer
                    .try_publish(BleEvent::BackendError {
                        operation: None,
                        stage: 0,
                        status: 1,
                    })
                    .is_err()
            );

            let id = controller.try_start_advertising(advertising()).unwrap();
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = ble_lifecycles();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), id);
            assert_eq!(
                completion.into_result(),
                Ok(BleOperation::AdvertisingRequested)
            );
            assert_eq!(
                controller.event_diagnostics().pending,
                crate::EVENT_CAPACITY
            );
            assert_eq!(controller.event_diagnostics().dropped, 1);
        }

        #[test]
        fn typed_gatt_database_crosses_only_the_facade_command_boundary() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (_producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            let mut lifecycles = ble_lifecycles();
            let id = controller
                .try_register_gatt_server(gatt_database())
                .unwrap();
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(backend.gatt_servers, 1);
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), id);
            assert_eq!(
                completion.into_result().unwrap(),
                BleOperation::GattServerRegistered(GattServerHandle {
                    server_id: 1,
                    service_handle: 2,
                    value_handle: 3,
                    ccc_handle: 4,
                })
            );
        }

        #[test]
        fn pairing_commands_and_events_keep_key_material_out_of_the_facade() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            let mut backend = FakeBackend {
                ready: true,
                pairing_state: crate::ble::PairingState::Paired,
                ..FakeBackend::default()
            };
            let mut lifecycles = ble_lifecycles();
            let peer = crate::ble::BluetoothAddress::public([1, 2, 3, 4, 5, 6]).unwrap();
            let security = crate::ble::SecurityConfig::new(
                crate::ble::Bonding::Enabled,
                crate::ble::IoCapability::NoInputNoOutput,
                crate::ble::SecurityRequirement::SecureConnectionsAuthenticated,
            );

            let configure = controller.try_configure_security(security).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), configure);
            assert_eq!(
                completion.into_result(),
                Ok(BleOperation::SecurityConfigured)
            );

            let pair = controller.try_pair(peer).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), pair);
            assert_eq!(completion.into_result(), Ok(BleOperation::PairingRequested));

            let query = controller.try_query_pairing_state(peer).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), query);
            assert_eq!(
                completion.into_result(),
                Ok(BleOperation::PairingState(crate::ble::PairingState::Paired))
            );

            let remove = controller.try_remove_bond(peer).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), remove);
            assert_eq!(completion.into_result(), Ok(BleOperation::BondRemoved));
            assert_eq!(backend.security_configurations, 1);
            assert_eq!(backend.pairing_requests, 1);
            assert_eq!(backend.pairing_queries, 1);
            assert_eq!(backend.bond_removals, 1);

            producer
                .try_publish(
                    map_ble_lifecycle_event(
                        hisi_rf_ws63::BleB2Event::AuthenticationComplete {
                            conn_id: 7,
                            address: peer.bytes(),
                            address_type: 0,
                            status: 0,
                            key_material_present: true,
                        },
                        &mut lifecycles,
                    )
                    .unwrap(),
                )
                .unwrap();
            let BleEvent::AuthenticationComplete {
                peer: event_peer,
                key_material_present,
                result,
            } = controller.try_next_event().unwrap()
            else {
                panic!("expected authentication completion");
            };
            assert_eq!(event_peer, peer);
            assert!(key_material_present);
            assert_eq!(result, Ok(()));
            assert_eq!(
                controller.event_diagnostics(),
                crate::ProtocolEventDiagnostics {
                    accepted: 1,
                    consumed: 1,
                    dropped: 0,
                    pending: 0,
                    high_water: 1,
                }
            );

            assert!(matches!(
                map_ble_lifecycle_event(
                    hisi_rf_ws63::BleB2Event::PairingComplete {
                        conn_id: 7,
                        address: peer.bytes(),
                        address_type: 9,
                        status: 0,
                    },
                    &mut lifecycles,
                ),
                Some(BleEvent::BackendError { stage: 5, .. })
            ));
        }

        #[test]
        fn advertising_guard_rejects_duplicate_start_and_waits_for_stop_callback() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = ble_lifecycles();

            let first = controller.try_start_advertising(advertising()).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(
                controller
                    .try_take_completion()
                    .unwrap()
                    .unwrap()
                    .into_result(),
                Ok(BleOperation::AdvertisingRequested)
            );

            let duplicate = controller.try_start_advertising(advertising()).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), duplicate);
            assert_eq!(
                completion.into_result().unwrap_err().kind(),
                BleOperationErrorKind::LifecycleBusy
            );
            assert_eq!(backend.advertising, 1);

            let event = map_ble_lifecycle_event(
                hisi_rf_ws63::BleB2Event::AdvertisingState {
                    adv_id: 0,
                    status: 1,
                },
                &mut lifecycles,
            )
            .unwrap();
            producer.try_publish(event).unwrap();
            let BleEvent::AdvertisingStarted { advertiser } = controller.try_next_event().unwrap()
            else {
                panic!("expected advertising lifecycle guard");
            };
            assert_eq!(advertiser.operation(), first);

            let mut stop = Box::pin(advertiser.stop());
            let waker = Waker::noop();
            let mut context = Context::from_waker(waker);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(run_ble_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.advertising_stops, 1);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(
                map_ble_lifecycle_event(
                    hisi_rf_ws63::BleB2Event::AdvertisingStopped {
                        adv_id: 0,
                        status: 0,
                    },
                    &mut lifecycles,
                )
                .is_none()
            );
            assert!(matches!(
                stop.as_mut().poll(&mut context),
                Poll::Ready(Ok(()))
            ));
        }

        #[test]
        fn rejected_active_event_drops_guard_and_requests_cleanup() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            for _ in 0..crate::EVENT_CAPACITY {
                producer
                    .try_publish(BleEvent::BackendError {
                        operation: None,
                        stage: 0,
                        status: 1,
                    })
                    .unwrap();
            }
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = ble_lifecycles();
            controller.try_start_advertising(advertising()).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let _ = controller.try_take_completion().unwrap().unwrap();
            let event = map_ble_lifecycle_event(
                hisi_rf_ws63::BleB2Event::AdvertisingState {
                    adv_id: 0,
                    status: 1,
                },
                &mut lifecycles,
            )
            .unwrap();
            drop(producer.try_publish(event).unwrap_err());
            assert!(run_ble_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.advertising_stops, 1);
        }

        #[test]
        fn scan_guard_stop_completes_after_synchronous_backend_stop() {
            let state = Box::leak(Box::new(BleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(BleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = BleController {
                sender,
                events: consumer,
            };
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = ble_lifecycles();
            controller.try_start_scanning(scanning()).unwrap();
            assert!(run_ble_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let _ = controller.try_take_completion().unwrap().unwrap();
            producer
                .try_publish(
                    map_ble_lifecycle_event(
                        hisi_rf_ws63::BleB2Event::ScanParameters { status: 0 },
                        &mut lifecycles,
                    )
                    .unwrap(),
                )
                .unwrap();
            let BleEvent::ScanReady { scanner } = controller.try_next_event().unwrap() else {
                panic!("expected scan lifecycle guard");
            };
            let mut stop = Box::pin(scanner.stop());
            let waker = Waker::noop();
            let mut context = Context::from_waker(waker);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(run_ble_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.scanning_stops, 1);
            assert!(matches!(
                stop.as_mut().poll(&mut context),
                Poll::Ready(Ok(()))
            ));
        }
    }
}

/// WS63 SLE U4 composition preview.
///
/// This profile establishes facade-owned storage, initialization, typed
/// announce/seek command submission, and runner ownership. The returned
/// completion means the WS63 host synchronously accepted or rejected the
/// request. Static typed SSAP registration and bounded asynchronous lifecycle
/// events, generation-tagged cancellation, and active guards are supported.
#[cfg(all(feature = "chip-ws63", feature = "profile-sle-ssap"))]
pub mod ws63 {
    pub use crate::declare_radio_storage;

    #[doc(hidden)]
    pub enum SleCommand {
        StartAnnounce(crate::sle::AnnounceConfig),
        StartSeek(crate::sle::SeekConfig),
        RegisterSsapServer(crate::sle::SsapServerDefinition),
    }

    type SleControlState =
        hisi_rf_core::control::ControlState<SleCommand, Result<SleOperation, SleOperationError>>;
    type SleEventState = hisi_rf_core::control::EventState<SleEvent, { crate::EVENT_CAPACITY }>;
    type SleLifecycleState = hisi_rf_core::control::LifecycleState<SleOperationError>;

    /// Caller-owned bytes shared by the pinned SLE controller and host tasks.
    pub const RADIO_ARENA_BYTES: usize = hisi_rf_ws63::SLE_S1_ARENA_BYTES;
    /// Smallest stack admitted by the pinned SLE task inventory.
    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    pub const MINIMUM_TASK_STACK_BYTES: usize = hisi_rf_ws63::SLE_S1_MINIMUM_TASK_STACK_BYTES;

    /// Announce payload used only by the paired U2 facade HIL fixture.
    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    pub const U2_HIL_PEER_PAYLOAD: &[u8] = &[1, 1, 1, 3, 2, 0x0b, 0x06];

    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    pub use hisi_rf_ws63::set_log_sink;

    /// Facade-owned lifecycle observations for the temporary U2 HIL gate.
    #[cfg(feature = "u2-hil-diagnostics")]
    #[doc(hidden)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum U2HilEvent {
        AnnounceStarted,
        SeekReady,
        PeerObserved,
        BackendError { stage: u8, status: u32 },
    }

    #[doc(hidden)]
    pub mod __private {
        pub use hisi_rf_ws63::{SleS1ArenaStorage as ArenaStorage, SleS1ControlStorage};
        pub type ProtocolControlStorage = super::SleControlState;
        pub type ProtocolEventStorage = super::SleEventState;
        pub type ProtocolLifecycleStorage = super::SleLifecycleState;
    }

    /// Caller-owned SLE composition storage.
    pub struct RadioStorage {
        inner: hisi_rf_ws63::SleS1Storage<RADIO_ARENA_BYTES>,
        control: &'static SleControlState,
        events: &'static SleEventState,
        announce: &'static SleLifecycleState,
        seek: &'static SleLifecycleState,
    }

    impl RadioStorage {
        /// Join the statically allocated control state and arena.
        #[doc(hidden)]
        pub const fn __from_parts(
            control: &'static __private::SleS1ControlStorage,
            arena: &'static __private::ArenaStorage<RADIO_ARENA_BYTES>,
            protocol: &'static __private::ProtocolControlStorage,
            events: &'static __private::ProtocolEventStorage,
            announce: &'static __private::ProtocolLifecycleStorage,
            seek: &'static __private::ProtocolLifecycleStorage,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::SleS1Storage::from_parts(control, arena),
                control: protocol,
                events,
                announce,
                seek,
            }
        }

        /// Claim and install this process-lifetime storage once.
        pub fn install(&'static self) -> Result<InstalledRadioStorage, InitError> {
            self.inner
                .install()
                .map(|inner| InstalledRadioStorage {
                    inner,
                    control: self.control,
                    events: self.events,
                    announce: self.announce,
                    seek: self.seek,
                })
                .map_err(|_| InitError::new())
        }
    }

    /// Installed SLE storage capability.
    pub struct InstalledRadioStorage {
        inner: hisi_rf_ws63::InstalledSleS1Storage,
        control: &'static SleControlState,
        events: &'static SleEventState,
        announce: &'static SleLifecycleState,
        seek: &'static SleLifecycleState,
    }

    impl InstalledRadioStorage {
        /// Allocate one zeroed runtime object from the installed arena.
        ///
        /// # Safety
        ///
        /// The pointer must be returned only through [`Self::deallocate`].
        pub unsafe fn allocate(size: usize) -> *mut u8 {
            // SAFETY: this facade preserves the backend allocator contract.
            unsafe { hisi_rf_ws63::InstalledSleS1Storage::allocate(size) }
        }

        /// Release an allocation from this composition.
        ///
        /// # Safety
        ///
        /// `pointer` must be null or a live allocation returned by
        /// [`Self::allocate`] that has not already been released.
        pub unsafe fn deallocate(pointer: *mut u8) {
            // SAFETY: the caller upholds the backend deallocation contract.
            unsafe { hisi_rf_ws63::InstalledSleS1Storage::deallocate(pointer) };
        }
    }

    /// Uniquely owned HAL capabilities required by this SLE profile.
    pub struct Resources {
        inner: hisi_rf_ws63::SleS1Resources,
    }

    impl Resources {
        /// Bind the WS63 eFuse and crypto peripherals to the radio lifecycle.
        pub const fn new(
            efuse: hisi_hal::peripherals::Efuse<'static>,
            km: hisi_hal::peripherals::Km<'static>,
            spacc: hisi_hal::peripherals::Spacc<'static>,
            trng: hisi_hal::peripherals::Trng<'static>,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::SleS1Resources::new(efuse, km, spacc, trng),
            }
        }
    }

    /// Opaque U1 initialization failure pending the U6 error taxonomy.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct InitError {
        _private: (),
    }

    impl InitError {
        const fn new() -> Self {
            Self { _private: () }
        }
    }

    /// Synchronous WS63 acceptance stage for one SLE command.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum SleOperationErrorKind {
        /// The asynchronous SLE host enable operation failed.
        Enable,
        /// The controller rejected announce timing or channel parameters.
        SetAnnounceParameters,
        /// The controller rejected announce or seek-response payload data.
        SetAnnounceData,
        /// The controller rejected the start-announce request.
        StartAnnounce,
        /// The controller rejected seek timing or filtering parameters.
        SetSeekParameters,
        /// The controller rejected the start-seek request.
        StartSeek,
        /// The controller rejected the announce stop request.
        StopAnnounce,
        /// The controller rejected the seek stop request.
        StopSeek,
        /// Another generation still owns this lifecycle.
        LifecycleBusy,
        /// A late callback or handle no longer names the active generation.
        StaleLifecycle,
        /// The selected WS63 profile cannot represent this valid generic config.
        UnsupportedConfiguration,
        /// This operation is unavailable in a host-only build.
        UnsupportedTarget,
        /// A legacy stage outside the U2 announce/seek surface rejected a request.
        LegacyStage,
        /// The static SSAP database could not be registered.
        SsapDatabase,
    }

    /// Fail-closed SLE command rejection with an optional vendor status.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct SleOperationError {
        kind: SleOperationErrorKind,
        vendor_status: Option<u32>,
    }

    impl SleOperationError {
        /// Return the operation stage that rejected the request.
        pub const fn kind(&self) -> SleOperationErrorKind {
            self.kind
        }

        /// Return the raw vendor status when rejection came from the WS63 host.
        pub const fn vendor_status(&self) -> Option<u32> {
            self.vendor_status
        }
    }

    /// Command accepted or rejected synchronously by the WS63 SLE host.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum SleOperation {
        /// The WS63 host accepted the start-announce request.
        AnnounceRequested,
        /// The WS63 host accepted the start-seek request.
        SeekRequested,
        /// The static SSAP database was registered and started.
        SsapServerRegistered(SsapServerHandle),
    }

    /// Active announce ownership returned by the SLE event plane.
    #[must_use = "call stop().await or retain the announcer; dropping it requests cleanup"]
    pub struct Announcer {
        operation: crate::ProtocolCommandId,
        inner: hisi_rf_core::control::LifecycleGuard<SleOperationError>,
    }

    impl Announcer {
        /// Command that created this announce generation.
        pub const fn operation(&self) -> crate::ProtocolCommandId {
            self.operation
        }

        /// Stop announcing and wait for the matching backend callback.
        pub async fn stop(self) -> Result<(), SleOperationError> {
            map_sle_stop_result(self.inner.stop().await)
        }
    }

    impl core::fmt::Debug for Announcer {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter
                .debug_struct("Announcer")
                .field("operation", &self.operation)
                .finish_non_exhaustive()
        }
    }

    /// Active seek ownership returned by the SLE event plane.
    #[must_use = "call stop().await or retain the seeker; dropping it requests cleanup"]
    pub struct Seeker {
        operation: crate::ProtocolCommandId,
        inner: hisi_rf_core::control::LifecycleGuard<SleOperationError>,
    }

    impl Seeker {
        /// Command that created this seek generation.
        pub const fn operation(&self) -> crate::ProtocolCommandId {
            self.operation
        }

        /// Stop seeking and wait for the matching backend callback.
        pub async fn stop(self) -> Result<(), SleOperationError> {
            map_sle_stop_result(self.inner.stop().await)
        }
    }

    impl core::fmt::Debug for Seeker {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter
                .debug_struct("Seeker")
                .field("operation", &self.operation)
                .finish_non_exhaustive()
        }
    }

    fn map_sle_stop_result(
        result: Result<(), hisi_rf_core::control::LifecycleStopError<SleOperationError>>,
    ) -> Result<(), SleOperationError> {
        result.map_err(|error| match error {
            hisi_rf_core::control::LifecycleStopError::Stale => SleOperationError {
                kind: SleOperationErrorKind::StaleLifecycle,
                vendor_status: None,
            },
            hisi_rf_core::control::LifecycleStopError::Backend(error) => error,
        })
    }

    /// Facade-owned SLE lifecycle event copied out of vendor callback context.
    #[derive(Debug)]
    pub enum SleEvent {
        /// Announcing entered the active state for this command generation.
        AnnounceStarted {
            /// Unique active announce capability.
            announcer: Announcer,
        },
        /// Seeking entered the active state for this command generation.
        SeekReady {
            /// Unique active seek capability.
            seeker: Seeker,
        },
        /// An asynchronous callback reported a backend failure.
        BackendError {
            /// Correlated command when the runner still owns its lifecycle.
            operation: Option<crate::ProtocolCommandId>,
            /// Stable facade stage identifier.
            stage: u8,
            /// Lossless vendor status.
            status: u32,
        },
    }

    struct SleLifecycle {
        operation: Option<crate::ProtocolCommandId>,
        generation: Option<hisi_rf_core::control::LifecycleId>,
        stopping: Option<hisi_rf_core::control::LifecycleId>,
        runner: hisi_rf_core::control::LifecycleRunner<SleOperationError>,
    }

    struct SleLifecycles {
        announce: SleLifecycle,
        seek: SleLifecycle,
    }

    impl SleLifecycle {
        fn new(runner: hisi_rf_core::control::LifecycleRunner<SleOperationError>) -> Self {
            Self {
                operation: None,
                generation: None,
                stopping: None,
                runner,
            }
        }

        fn clear(&mut self) {
            self.operation = None;
            self.generation = None;
            self.stopping = None;
        }
    }

    impl SleLifecycles {
        fn new(
            announce: hisi_rf_core::control::LifecycleRunner<SleOperationError>,
            seek: hisi_rf_core::control::LifecycleRunner<SleOperationError>,
        ) -> Self {
            Self {
                announce: SleLifecycle::new(announce),
                seek: SleLifecycle::new(seek),
            }
        }
    }

    /// Unforgeable facade handle for one registered static SSAP server.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct SsapServerHandle {
        server_id: u8,
        service_handle: u16,
        property_handle: u16,
    }

    trait SleBackend {
        fn command_ready(&self) -> Result<bool, SleOperationError>;
        fn start_announce(
            &mut self,
            config: crate::sle::AnnounceConfig,
        ) -> Result<(), SleOperationError>;
        fn start_seek(&mut self, config: crate::sle::SeekConfig) -> Result<(), SleOperationError>;
        fn stop_announce(&mut self) -> Result<(), SleOperationError>;
        fn stop_seek(&mut self) -> Result<(), SleOperationError>;
        fn register_ssap_server(
            &mut self,
            definition: crate::sle::SsapServerDefinition,
        ) -> Result<SsapServerHandle, SleOperationError>;
    }

    impl SleBackend for hisi_rf_ws63::SleS1Controller {
        fn command_ready(&self) -> Result<bool, SleOperationError> {
            match self.enable_status() {
                None => Ok(false),
                Some(0) => Ok(true),
                Some(status) => Err(SleOperationError {
                    kind: SleOperationErrorKind::Enable,
                    vendor_status: Some(status),
                }),
            }
        }

        fn start_announce(
            &mut self,
            config: crate::sle::AnnounceConfig,
        ) -> Result<(), SleOperationError> {
            self.start_announce_config(config).map_err(map_sle_error)
        }

        fn start_seek(&mut self, config: crate::sle::SeekConfig) -> Result<(), SleOperationError> {
            self.start_seek_config(config).map_err(map_sle_error)
        }

        fn stop_announce(&mut self) -> Result<(), SleOperationError> {
            self.stop_announce().map_err(map_sle_error)
        }

        fn stop_seek(&mut self) -> Result<(), SleOperationError> {
            self.stop_seek().map_err(map_sle_error)
        }

        fn register_ssap_server(
            &mut self,
            definition: crate::sle::SsapServerDefinition,
        ) -> Result<SsapServerHandle, SleOperationError> {
            self.configure_ssap_server_definition(definition)
                .map(|handles| SsapServerHandle {
                    server_id: handles.server_id,
                    service_handle: handles.service_handle,
                    property_handle: handles.property_handle,
                })
                .map_err(map_sle_error)
        }
    }

    fn map_sle_error(error: hisi_rf_ws63::SleS1OperationError) -> SleOperationError {
        use hisi_rf_ws63::SleS1OperationError as E;
        let (kind, vendor_status) = match error {
            E::AnnounceDataTooLong { .. } | E::SeekResponseDataTooLong { .. } => {
                (SleOperationErrorKind::UnsupportedConfiguration, None)
            }
            E::SetAnnounceParameters(status) => {
                (SleOperationErrorKind::SetAnnounceParameters, Some(status))
            }
            E::SetAnnounceData(status) => (SleOperationErrorKind::SetAnnounceData, Some(status)),
            E::StartAnnounce(status) => (SleOperationErrorKind::StartAnnounce, Some(status)),
            E::SetSeekParameters(status) => {
                (SleOperationErrorKind::SetSeekParameters, Some(status))
            }
            E::StartSeek(status) => (SleOperationErrorKind::StartSeek, Some(status)),
            E::StopAnnounce(status) => (SleOperationErrorKind::StopAnnounce, Some(status)),
            E::StopSeek(status) => (SleOperationErrorKind::StopSeek, Some(status)),
            E::UnsupportedTarget => (SleOperationErrorKind::UnsupportedTarget, None),
            E::SetLocalAddress(status)
            | E::SetConnectionParameters(status)
            | E::Connect(status)
            | E::Disconnect(status)
            | E::Pair(status)
            | E::RegisterSsapServer(status)
            | E::AddSsapService(status)
            | E::AddSsapProperty(status)
            | E::AddSsapDescriptor(status)
            | E::SetSsapInfo(status)
            | E::StartSsapService(status)
            | E::NotifySsap(status)
            | E::ExchangeSsapInfo(status)
            | E::DiscoverSsapServices(status)
            | E::ReadSsap(status)
            | E::WriteSsap(status) => (SleOperationErrorKind::LegacyStage, Some(status)),
            E::UnsupportedDatabase | E::SsapValueTooLong { .. } => {
                (SleOperationErrorKind::UnsupportedConfiguration, None)
            }
        };
        SleOperationError {
            kind,
            vendor_status,
        }
    }

    fn run_sle_once(
        receiver: &mut hisi_rf_core::control::ControlReceiver<
            SleCommand,
            Result<SleOperation, SleOperationError>,
        >,
        backend: &mut impl SleBackend,
        lifecycles: &mut SleLifecycles,
    ) -> Result<bool, crate::ProtocolError> {
        let readiness = backend.command_ready();
        if matches!(readiness, Ok(false)) {
            return Ok(false);
        }
        let Some(command) = receiver.try_take_command() else {
            return Ok(false);
        };
        let id = command.id();
        let command = command.into_inner();
        let result = match readiness {
            Ok(true) => match command {
                SleCommand::StartAnnounce(config) => match lifecycles.announce.runner.begin() {
                    Ok(generation) => match backend.start_announce(config) {
                        Ok(()) => {
                            lifecycles.announce.operation = Some(crate::ProtocolCommandId(id));
                            lifecycles.announce.generation = Some(generation);
                            Ok(SleOperation::AnnounceRequested)
                        }
                        Err(error) => {
                            let _ = lifecycles.announce.runner.abort_start(generation);
                            Err(error)
                        }
                    },
                    Err(_) => Err(SleOperationError {
                        kind: SleOperationErrorKind::LifecycleBusy,
                        vendor_status: None,
                    }),
                },
                SleCommand::StartSeek(config) => match lifecycles.seek.runner.begin() {
                    Ok(generation) => match backend.start_seek(config) {
                        Ok(()) => {
                            lifecycles.seek.operation = Some(crate::ProtocolCommandId(id));
                            lifecycles.seek.generation = Some(generation);
                            Ok(SleOperation::SeekRequested)
                        }
                        Err(error) => {
                            let _ = lifecycles.seek.runner.abort_start(generation);
                            Err(error)
                        }
                    },
                    Err(_) => Err(SleOperationError {
                        kind: SleOperationErrorKind::LifecycleBusy,
                        vendor_status: None,
                    }),
                },
                SleCommand::RegisterSsapServer(definition) => backend
                    .register_ssap_server(definition)
                    .map(SleOperation::SsapServerRegistered),
            },
            Err(error) => Err(error),
            Ok(false) => unreachable!(),
        };
        receiver
            .complete(id, result)
            .map_err(|_| crate::ProtocolError::CompletionOwnership)?;
        Ok(true)
    }

    fn map_sle_lifecycle_event(
        event: hisi_rf_ws63::SleS1Event,
        lifecycles: &mut SleLifecycles,
    ) -> Option<SleEvent> {
        match event {
            hisi_rf_ws63::SleS1Event::AnnounceEnabled { status: 0, .. } => {
                let operation = lifecycles.announce.operation?;
                let generation = lifecycles.announce.generation?;
                lifecycles
                    .announce
                    .runner
                    .activate(generation)
                    .ok()
                    .map(|inner| SleEvent::AnnounceStarted {
                        announcer: Announcer { operation, inner },
                    })
            }
            hisi_rf_ws63::SleS1Event::AnnounceEnabled { status, .. } => {
                if let Some(generation) = lifecycles.announce.generation {
                    let _ = lifecycles.announce.runner.abort_start(generation);
                }
                let operation = lifecycles.announce.operation;
                lifecycles.announce.clear();
                Some(SleEvent::BackendError {
                    operation,
                    stage: 1,
                    status,
                })
            }
            hisi_rf_ws63::SleS1Event::AnnounceDisabled { status, .. } => {
                let generation = lifecycles.announce.stopping.take()?;
                let result = if status == 0 {
                    Ok(())
                } else {
                    Err(SleOperationError {
                        kind: SleOperationErrorKind::StopAnnounce,
                        vendor_status: Some(status),
                    })
                };
                let _ = lifecycles.announce.runner.finish(generation, result);
                lifecycles.announce.clear();
                None
            }
            hisi_rf_ws63::SleS1Event::SeekEnabled { status: 0 } => {
                let operation = lifecycles.seek.operation?;
                let generation = lifecycles.seek.generation?;
                lifecycles
                    .seek
                    .runner
                    .activate(generation)
                    .ok()
                    .map(|inner| SleEvent::SeekReady {
                        seeker: Seeker { operation, inner },
                    })
            }
            hisi_rf_ws63::SleS1Event::SeekEnabled { status } => {
                if let Some(generation) = lifecycles.seek.generation {
                    let _ = lifecycles.seek.runner.abort_start(generation);
                }
                let operation = lifecycles.seek.operation;
                lifecycles.seek.clear();
                Some(SleEvent::BackendError {
                    operation,
                    stage: 2,
                    status,
                })
            }
            hisi_rf_ws63::SleS1Event::SeekDisabled { status } => {
                let generation = lifecycles.seek.stopping.take()?;
                let result = if status == 0 {
                    Ok(())
                } else {
                    Err(SleOperationError {
                        kind: SleOperationErrorKind::StopSeek,
                        vendor_status: Some(status),
                    })
                };
                let _ = lifecycles.seek.runner.finish(generation, result);
                lifecycles.seek.clear();
                None
            }
            hisi_rf_ws63::SleS1Event::Enabled { status } if status != 0 => {
                Some(SleEvent::BackendError {
                    operation: None,
                    stage: 0,
                    status,
                })
            }
            _ => None,
        }
    }

    fn run_sle_cancellation_once(
        backend: &mut impl SleBackend,
        lifecycles: &mut SleLifecycles,
    ) -> bool {
        if let Some(generation) = lifecycles.announce.runner.try_take_cancel() {
            match backend.stop_announce() {
                Ok(()) => lifecycles.announce.stopping = Some(generation),
                Err(error) => {
                    let _ = lifecycles.announce.runner.finish(generation, Err(error));
                    lifecycles.announce.clear();
                }
            }
            return true;
        }
        if let Some(generation) = lifecycles.seek.runner.try_take_cancel() {
            match backend.stop_seek() {
                Ok(()) => lifecycles.seek.stopping = Some(generation),
                Err(error) => {
                    let _ = lifecycles.seek.runner.finish(generation, Err(error));
                    lifecycles.seek.clear();
                }
            }
            return true;
        }
        false
    }

    /// Exclusive SLE composition before task ownership is split.
    pub struct RadioController {
        inner: hisi_rf_ws63::SleS1Controller,
        sender: hisi_rf_core::control::ControlSender<
            SleCommand,
            Result<SleOperation, SleOperationError>,
        >,
        receiver: hisi_rf_core::control::ControlReceiver<
            SleCommand,
            Result<SleOperation, SleOperationError>,
        >,
        event_producer: hisi_rf_core::control::EventProducer<SleEvent, { crate::EVENT_CAPACITY }>,
        event_consumer: hisi_rf_core::control::EventConsumer<SleEvent, { crate::EVENT_CAPACITY }>,
        announce: hisi_rf_core::control::LifecycleRunner<SleOperationError>,
        seek: hisi_rf_core::control::LifecycleRunner<SleOperationError>,
    }

    impl RadioController {
        /// Split the facade into the SLE handle and mandatory runner owner.
        pub fn split(self) -> RadioParts {
            RadioParts {
                sle: SleController {
                    sender: self.sender,
                    events: self.event_consumer,
                },
                runner: RadioRunner {
                    inner: self.inner,
                    receiver: self.receiver,
                    events: self.event_producer,
                    lifecycles: SleLifecycles::new(self.announce, self.seek),
                },
            }
        }
    }

    /// SLE protocol handle reserved for the U2-U4 typed control plane.
    pub struct SleController {
        sender: hisi_rf_core::control::ControlSender<
            SleCommand,
            Result<SleOperation, SleOperationError>,
        >,
        events: hisi_rf_core::control::EventConsumer<SleEvent, { crate::EVENT_CAPACITY }>,
    }

    impl SleController {
        // Returning the fixed-capacity request is the allocation-free
        // backpressure contract; boxing would violate the no-heap profile.
        #[allow(clippy::result_large_err)]
        /// Queue a typed announce request without blocking the caller.
        ///
        /// A busy mailbox returns ownership of `config`. Success only means the
        /// runner accepted the command; call [`Self::try_take_completion`] for
        /// the synchronous WS63 host result.
        pub fn try_start_announce(
            &mut self,
            config: crate::sle::AnnounceConfig,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::sle::AnnounceConfig>>
        {
            self.sender
                .try_submit(SleCommand::StartAnnounce(config))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    SleCommand::StartAnnounce(config) => crate::ProtocolBusy { request: config },
                    SleCommand::StartSeek(_) | SleCommand::RegisterSsapServer(_) => unreachable!(),
                })
        }

        /// Queue a typed seek request without blocking the caller.
        ///
        /// A busy mailbox returns ownership of `config`. Success only means the
        /// runner accepted the command; call [`Self::try_take_completion`] for
        /// the synchronous WS63 host result.
        pub fn try_start_seek(
            &mut self,
            config: crate::sle::SeekConfig,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::sle::SeekConfig>> {
            self.sender
                .try_submit(SleCommand::StartSeek(config))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    SleCommand::StartSeek(config) => crate::ProtocolBusy { request: config },
                    SleCommand::StartAnnounce(_) | SleCommand::RegisterSsapServer(_) => {
                        unreachable!()
                    }
                })
        }

        /// Queue registration of one validated static SSAP database.
        pub fn try_register_ssap_server(
            &mut self,
            definition: crate::sle::SsapServerDefinition,
        ) -> Result<crate::ProtocolCommandId, crate::ProtocolBusy<crate::sle::SsapServerDefinition>>
        {
            self.sender
                .try_submit(SleCommand::RegisterSsapServer(definition))
                .map(crate::ProtocolCommandId)
                .map_err(|error| match error.into_inner() {
                    SleCommand::RegisterSsapServer(definition) => crate::ProtocolBusy {
                        request: definition,
                    },
                    SleCommand::StartAnnounce(_) | SleCommand::StartSeek(_) => unreachable!(),
                })
        }

        /// Take the terminal result for the current command, if available.
        ///
        /// The operation variants report synchronous request acceptance, not
        /// an on-air announce or seek lifecycle transition.
        pub fn try_take_completion(
            &mut self,
        ) -> Result<
            Option<crate::ProtocolCompletion<SleOperation, SleOperationError>>,
            crate::ProtocolError,
        > {
            self.sender
                .try_take_completion()
                .map(|completion| {
                    completion.map(|completion| crate::ProtocolCompletion {
                        id: crate::ProtocolCommandId(completion.id()),
                        result: completion.into_inner(),
                    })
                })
                .map_err(|_| crate::ProtocolError::StaleCompletion)
        }

        /// Take the oldest unsolicited lifecycle event without waiting.
        pub fn try_next_event(&mut self) -> Option<SleEvent> {
            self.events.try_next_event()
        }

        /// Wait for the oldest unsolicited lifecycle event.
        pub async fn next_event(&mut self) -> SleEvent {
            self.events.next_event().await
        }

        /// Snapshot public event conservation counters.
        pub fn event_diagnostics(&self) -> crate::ProtocolEventDiagnostics {
            crate::protocol_event_diagnostics(self.events.diagnostics())
        }
    }

    /// Parts enabled by the SLE-only compile-time profile.
    pub struct RadioParts {
        /// SLE control-plane capability.
        pub sle: SleController,
        /// Mandatory owner of the WS63 controller/host runtime.
        pub runner: RadioRunner,
    }

    /// Unique process-lifetime owner of the internal SLE stage controller.
    pub struct RadioRunner {
        inner: hisi_rf_ws63::SleS1Controller,
        receiver: hisi_rf_core::control::ControlReceiver<
            SleCommand,
            Result<SleOperation, SleOperationError>,
        >,
        events: hisi_rf_core::control::EventProducer<SleEvent, { crate::EVENT_CAPACITY }>,
        lifecycles: SleLifecycles,
    }

    impl RadioRunner {
        /// Number of copied vendor events rejected by the bounded backend queue.
        pub fn dropped_events(&self) -> u32 {
            self.inner.dropped_events()
        }

        /// Execute at most one queued SLE command and publish its completion.
        ///
        /// Returns `Ok(false)` when no command is pending or the asynchronous
        /// SLE enable callback has not arrived yet. The queued command retains
        /// ownership in both cases. Applications should call this from their
        /// single long-lived radio runner task.
        pub fn run_once(&mut self) -> Result<bool, crate::ProtocolError> {
            if run_sle_cancellation_once(&mut self.inner, &mut self.lifecycles) {
                return Ok(true);
            }
            run_sle_once(&mut self.receiver, &mut self.inner, &mut self.lifecycles)
        }

        /// Copy at most one backend lifecycle event into the public queue.
        pub fn run_event_once(&mut self) -> bool {
            let Some(event) = self.inner.next_event() else {
                return false;
            };
            let event = map_sle_lifecycle_event(event, &mut self.lifecycles);
            if let Some(event) = event {
                let _ = self.events.try_publish(event);
            }
            true
        }

        /// Consume one backend lifecycle event for the temporary U2 HIL gate.
        #[cfg(feature = "u2-hil-diagnostics")]
        #[doc(hidden)]
        pub fn next_hil_event(&mut self) -> Option<U2HilEvent> {
            while let Some(event) = self.inner.next_event() {
                use hisi_rf_ws63::SleS1Event as E;
                let mapped = match event {
                    E::AnnounceEnabled { status: 0, .. } => Some(U2HilEvent::AnnounceStarted),
                    E::AnnounceEnabled { status, .. } => {
                        Some(U2HilEvent::BackendError { stage: 1, status })
                    }
                    E::SeekEnabled { status: 0 } => Some(U2HilEvent::SeekReady),
                    E::SeekEnabled { status } => {
                        Some(U2HilEvent::BackendError { stage: 2, status })
                    }
                    E::SeekResult { data_len, data, .. }
                        if data[..usize::from(data_len).min(data.len())]
                            == *U2_HIL_PEER_PAYLOAD =>
                    {
                        Some(U2HilEvent::PeerObserved)
                    }
                    E::Enabled { status } if status != 0 => {
                        Some(U2HilEvent::BackendError { stage: 0, status })
                    }
                    _ => None,
                };
                if mapped.is_some() {
                    return mapped;
                }
            }
            None
        }
    }

    /// Initialize the SLE-only WS63 composition.
    pub fn init(
        resources: Resources,
        storage: InstalledRadioStorage,
    ) -> Result<RadioController, InitError> {
        let (sender, receiver) = storage.control.claim().ok_or_else(InitError::new)?;
        let (event_producer, event_consumer) = storage.events.claim().ok_or_else(InitError::new)?;
        let announce = storage.announce.claim().ok_or_else(InitError::new)?;
        let seek = storage.seek.claim().ok_or_else(InitError::new)?;
        hisi_rf_ws63::init_sle_s1(resources.inner, storage.inner)
            .map(|inner| RadioController {
                inner,
                sender,
                receiver,
                event_producer,
                event_consumer,
                announce,
                seek,
            })
            .map_err(|_| InitError::new())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::future::Future;
        use std::boxed::Box;
        use std::task::{Context, Poll, Waker};

        #[derive(Default)]
        struct FakeBackend {
            announce: usize,
            seek: usize,
            announce_stops: usize,
            seek_stops: usize,
            ssap_servers: usize,
            ready: bool,
            enable_error: Option<u32>,
            reject_seek: bool,
        }

        impl SleBackend for FakeBackend {
            fn command_ready(&self) -> Result<bool, SleOperationError> {
                match self.enable_error {
                    Some(status) => Err(SleOperationError {
                        kind: SleOperationErrorKind::Enable,
                        vendor_status: Some(status),
                    }),
                    None => Ok(self.ready),
                }
            }

            fn start_announce(
                &mut self,
                _: crate::sle::AnnounceConfig,
            ) -> Result<(), SleOperationError> {
                self.announce += 1;
                Ok(())
            }

            fn start_seek(&mut self, _: crate::sle::SeekConfig) -> Result<(), SleOperationError> {
                self.seek += 1;
                if self.reject_seek {
                    Err(SleOperationError {
                        kind: SleOperationErrorKind::StartSeek,
                        vendor_status: Some(0x5678),
                    })
                } else {
                    Ok(())
                }
            }

            fn stop_announce(&mut self) -> Result<(), SleOperationError> {
                self.announce_stops += 1;
                Ok(())
            }

            fn stop_seek(&mut self) -> Result<(), SleOperationError> {
                self.seek_stops += 1;
                Ok(())
            }

            fn register_ssap_server(
                &mut self,
                _: crate::sle::SsapServerDefinition,
            ) -> Result<SsapServerHandle, SleOperationError> {
                self.ssap_servers += 1;
                Ok(SsapServerHandle {
                    server_id: 1,
                    service_handle: 2,
                    property_handle: 3,
                })
            }
        }

        fn sle_lifecycles() -> SleLifecycles {
            let announce = Box::leak(Box::new(SleLifecycleState::new()))
                .claim()
                .unwrap();
            let seek = Box::leak(Box::new(SleLifecycleState::new()))
                .claim()
                .unwrap();
            SleLifecycles::new(announce, seek)
        }

        fn announce() -> crate::sle::AnnounceConfig {
            let interval = crate::sle::AnnounceInterval::try_from_units(0x20).unwrap();
            crate::sle::AnnounceConfig::new(
                crate::sle::AnnounceTiming::try_new(interval, interval).unwrap(),
                crate::sle::AnnounceChannels::ALL,
                crate::sle::AnnouncePayload::try_from_slice(b"announce").unwrap(),
                crate::sle::AnnouncePayload::try_from_slice(b"response").unwrap(),
            )
        }

        fn seek() -> crate::sle::SeekConfig {
            let interval = crate::sle::SeekInterval::try_from_units(0x20).unwrap();
            crate::sle::SeekConfig::new(
                crate::sle::SeekTiming::try_new(interval, interval).unwrap(),
                true,
            )
        }

        fn ssap_database() -> crate::sle::SsapServerDefinition {
            const DESCRIPTOR: crate::sle::SsapDescriptorDefinition =
                crate::sle::SsapDescriptorDefinition::try_new(
                    crate::sle::SsapUuid::Uuid16(0x600d),
                    crate::sle::SsapPermissions::READ,
                    b"U3 descriptor",
                    32,
                )
                .unwrap();
            const PROPERTY: crate::sle::SsapPropertyDefinition =
                crate::sle::SsapPropertyDefinition::try_new(
                    crate::sle::SsapUuid::Uuid16(0x600c),
                    crate::sle::SsapPermissions::READ.union(crate::sle::SsapPermissions::WRITE),
                    crate::sle::SsapOperations::READ.union(crate::sle::SsapOperations::NOTIFY),
                    b"U3",
                    32,
                    &[DESCRIPTOR],
                )
                .unwrap();
            const SERVICE: crate::sle::SsapServiceDefinition =
                crate::sle::SsapServiceDefinition::try_new(
                    crate::sle::SsapUuid::Uuid16(0x600b),
                    &[PROPERTY],
                )
                .unwrap();
            crate::sle::SsapServerDefinition::try_new(
                crate::sle::SsapUuid::Uuid16(0x600a),
                &[SERVICE],
            )
            .unwrap()
        }

        #[test]
        fn bounded_controller_and_runner_preserve_sle_command_ownership() {
            let state = Box::leak(Box::new(SleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(SleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = SleController {
                sender,
                events: consumer,
            };
            let mut lifecycles = sle_lifecycles();
            let id = controller.try_start_announce(announce()).unwrap();
            let rejected = controller.try_start_seek(seek()).unwrap_err();
            assert_eq!(rejected.into_inner(), seek());

            let mut backend = FakeBackend::default();
            assert!(!run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert!(controller.try_take_completion().unwrap().is_none());
            backend.ready = true;
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(backend.announce, 1);
            assert_eq!(backend.seek, 0);
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), id);
            assert_eq!(
                completion.into_result(),
                Ok(SleOperation::AnnounceRequested)
            );
            let event = map_sle_lifecycle_event(
                hisi_rf_ws63::SleS1Event::AnnounceEnabled {
                    announce_id: 0,
                    status: 0,
                },
                &mut lifecycles,
            )
            .unwrap();
            producer.try_publish(event).unwrap();
            let SleEvent::AnnounceStarted { announcer } = controller.try_next_event().unwrap()
            else {
                panic!("expected announce lifecycle guard");
            };
            assert_eq!(announcer.operation(), id);
            drop(announcer);
            assert!(run_sle_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.announce_stops, 1);
            assert!(
                map_sle_lifecycle_event(
                    hisi_rf_ws63::SleS1Event::AnnounceDisabled {
                        announce_id: 1,
                        status: 0,
                    },
                    &mut lifecycles,
                )
                .is_none()
            );
            assert_eq!(
                controller.event_diagnostics(),
                crate::ProtocolEventDiagnostics {
                    accepted: 1,
                    consumed: 1,
                    dropped: 0,
                    pending: 0,
                    high_water: 1,
                }
            );

            backend.reject_seek = true;
            let seek_id = controller.try_start_seek(seek()).unwrap();
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), seek_id);
            let error = completion.into_result().unwrap_err();
            assert_eq!(error.kind(), SleOperationErrorKind::StartSeek);
            assert_eq!(error.vendor_status(), Some(0x5678));
            assert!(!run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());

            backend.enable_error = Some(0x8765);
            let enable_id = controller.try_start_announce(announce()).unwrap();
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(backend.announce, 1);
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), enable_id);
            let error = completion.into_result().unwrap_err();
            assert_eq!(error.kind(), SleOperationErrorKind::Enable);
            assert_eq!(error.vendor_status(), Some(0x8765));
        }

        #[test]
        fn full_sle_event_queue_does_not_consume_command_completion() {
            let state = Box::leak(Box::new(SleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(SleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = SleController {
                sender,
                events: consumer,
            };
            for _ in 0..crate::EVENT_CAPACITY {
                producer
                    .try_publish(SleEvent::BackendError {
                        operation: None,
                        stage: 0,
                        status: 1,
                    })
                    .unwrap();
            }
            assert!(
                producer
                    .try_publish(SleEvent::BackendError {
                        operation: None,
                        stage: 0,
                        status: 1,
                    })
                    .is_err()
            );

            let id = controller.try_start_announce(announce()).unwrap();
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = sle_lifecycles();
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), id);
            assert_eq!(
                completion.into_result(),
                Ok(SleOperation::AnnounceRequested)
            );
            assert_eq!(
                controller.event_diagnostics().pending,
                crate::EVENT_CAPACITY
            );
            assert_eq!(controller.event_diagnostics().dropped, 1);
        }

        #[test]
        fn typed_ssap_database_crosses_only_the_facade_command_boundary() {
            let state = Box::leak(Box::new(SleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(SleEventState::new()));
            let (_producer, consumer) = events.claim().unwrap();
            let mut controller = SleController {
                sender,
                events: consumer,
            };
            let mut lifecycles = sle_lifecycles();
            let id = controller
                .try_register_ssap_server(ssap_database())
                .unwrap();
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(backend.ssap_servers, 1);
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), id);
            assert_eq!(
                completion.into_result().unwrap(),
                SleOperation::SsapServerRegistered(SsapServerHandle {
                    server_id: 1,
                    service_handle: 2,
                    property_handle: 3,
                })
            );
        }

        #[test]
        fn announce_guard_rejects_duplicate_start_and_waits_for_stop_callback() {
            let state = Box::leak(Box::new(SleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(SleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = SleController {
                sender,
                events: consumer,
            };
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = sle_lifecycles();

            let first = controller.try_start_announce(announce()).unwrap();
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            assert_eq!(
                controller
                    .try_take_completion()
                    .unwrap()
                    .unwrap()
                    .into_result(),
                Ok(SleOperation::AnnounceRequested)
            );

            let duplicate = controller.try_start_announce(announce()).unwrap();
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let completion = controller.try_take_completion().unwrap().unwrap();
            assert_eq!(completion.id(), duplicate);
            assert_eq!(
                completion.into_result().unwrap_err().kind(),
                SleOperationErrorKind::LifecycleBusy
            );
            assert_eq!(backend.announce, 1);

            let event = map_sle_lifecycle_event(
                hisi_rf_ws63::SleS1Event::AnnounceEnabled {
                    announce_id: 1,
                    status: 0,
                },
                &mut lifecycles,
            )
            .unwrap();
            producer.try_publish(event).unwrap();
            let SleEvent::AnnounceStarted { announcer } = controller.try_next_event().unwrap()
            else {
                panic!("expected announce lifecycle guard");
            };
            assert_eq!(announcer.operation(), first);

            let mut stop = Box::pin(announcer.stop());
            let waker = Waker::noop();
            let mut context = Context::from_waker(waker);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(run_sle_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.announce_stops, 1);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(
                map_sle_lifecycle_event(
                    hisi_rf_ws63::SleS1Event::AnnounceDisabled {
                        announce_id: 1,
                        status: 0,
                    },
                    &mut lifecycles,
                )
                .is_none()
            );
            assert!(matches!(
                stop.as_mut().poll(&mut context),
                Poll::Ready(Ok(()))
            ));
        }

        #[test]
        fn seek_guard_stop_waits_for_matching_disabled_callback() {
            let state = Box::leak(Box::new(SleControlState::new()));
            let (sender, mut receiver) = state.claim().unwrap();
            let events = Box::leak(Box::new(SleEventState::new()));
            let (mut producer, consumer) = events.claim().unwrap();
            let mut controller = SleController {
                sender,
                events: consumer,
            };
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = sle_lifecycles();
            controller.try_start_seek(seek()).unwrap();
            assert!(run_sle_once(&mut receiver, &mut backend, &mut lifecycles).unwrap());
            let _ = controller.try_take_completion().unwrap().unwrap();
            producer
                .try_publish(
                    map_sle_lifecycle_event(
                        hisi_rf_ws63::SleS1Event::SeekEnabled { status: 0 },
                        &mut lifecycles,
                    )
                    .unwrap(),
                )
                .unwrap();
            let SleEvent::SeekReady { seeker } = controller.try_next_event().unwrap() else {
                panic!("expected seek lifecycle guard");
            };
            let mut stop = Box::pin(seeker.stop());
            let waker = Waker::noop();
            let mut context = Context::from_waker(waker);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(run_sle_cancellation_once(&mut backend, &mut lifecycles));
            assert_eq!(backend.seek_stops, 1);
            assert!(matches!(stop.as_mut().poll(&mut context), Poll::Pending));
            assert!(
                map_sle_lifecycle_event(
                    hisi_rf_ws63::SleS1Event::SeekDisabled { status: 0 },
                    &mut lifecycles,
                )
                .is_none()
            );
            assert!(matches!(
                stop.as_mut().poll(&mut context),
                Poll::Ready(Ok(()))
            ));
        }
    }
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
        osal_queue::{
            EventDiagnostic as OsalEventDiagnostic, event_diagnostics as osal_event_diagnostics,
        },
        osal_wait::{
            WaitDiagnostic as OsalWaitDiagnostic, wait_diagnostics as osal_wait_diagnostics,
        },
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
        feature = "incremental-backend-experiment",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// WS63 controller for the selected public profile.
    #[cfg_attr(
        not(feature = "incremental-embassy-wait"),
        allow(dead_code, reason = "split is enabled by incremental-embassy-wait")
    )]
    pub struct RadioController(
        hisi_rf_ws63::IncrementalRadioController<SelectedProfile, EVENT_CAPACITY>,
    );

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    impl RadioController {
        /// Split into fixed-profile Wi-Fi parts and a bounded runner.
        pub fn split(self, budget: crate::WorkBudget) -> RadioParts {
            let parts = self.0.split(budget);
            RadioParts {
                wifi: WifiParts {
                    controller: parts.wifi.controller,
                    device: parts.wifi.device,
                },
                runner: RadioRunner(parts.runner),
            }
        }
    }

    #[cfg(all(
        feature = "incremental-backend-experiment",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    /// Initialize the selected bounded WS63 profile.
    pub fn init(
        config: crate::RadioConfig,
        resources: Resources<SelectedProfile>,
        storage: &'static Storage,
    ) -> Result<RadioController, InitError> {
        hisi_rf_ws63::init_incremental(config, resources, &storage.inner).map(RadioController)
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
    /// WS63 parts for the selected public profile.
    pub struct RadioParts {
        /// Async Wi-Fi control and data plane.
        pub wifi: WifiParts,
        /// Bounded incremental runner.
        pub runner: RadioRunner,
    }

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    impl RadioParts {
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
    /// Bounded WS63 runner for the selected public profile.
    pub struct RadioRunner(hisi_rf_ws63::IncrementalRadioRunner<EVENT_CAPACITY>);

    #[cfg(all(
        feature = "incremental-embassy-wait",
        feature = "smoltcp",
        any(feature = "wpa2-personal", feature = "wpa3-personal")
    ))]
    impl RadioRunner {
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

/// Declare caller-owned storage for the BLE U1 migration profile.
#[cfg(all(feature = "chip-ws63", feature = "profile-ble-dual-role"))]
#[macro_export]
macro_rules! declare_radio_storage {
    ($(#[$meta:meta])* $vis:vis static $name:ident) => {
        $(#[$meta])*
        $vis static $name: $crate::ws63::RadioStorage = {
            static CONTROL: $crate::ws63::__private::BleB1ControlStorage =
                $crate::ws63::__private::BleB1ControlStorage::new();
            static PROTOCOL: $crate::ws63::__private::ProtocolControlStorage =
                $crate::ws63::__private::ProtocolControlStorage::new();
            static EVENTS: $crate::ws63::__private::ProtocolEventStorage =
                $crate::ws63::__private::ProtocolEventStorage::new();
            static ADVERTISING: $crate::ws63::__private::ProtocolLifecycleStorage =
                $crate::ws63::__private::ProtocolLifecycleStorage::new();
            static SCANNING: $crate::ws63::__private::ProtocolLifecycleStorage =
                $crate::ws63::__private::ProtocolLifecycleStorage::new();
            #[cfg_attr(target_arch = "riscv32", unsafe(link_section = ".hisi.shared-arena"))]
            static ARENA: $crate::ws63::__private::ArenaStorage<
                { $crate::ws63::RADIO_ARENA_BYTES },
            > = $crate::ws63::__private::ArenaStorage::new();
            $crate::ws63::RadioStorage::__from_parts(
                &CONTROL,
                &ARENA,
                &PROTOCOL,
                &EVENTS,
                &ADVERTISING,
                &SCANNING,
            )
        };
    };
}

/// Declare caller-owned storage for the SLE U1 migration profile.
#[cfg(all(feature = "chip-ws63", feature = "profile-sle-ssap"))]
#[macro_export]
macro_rules! declare_radio_storage {
    ($(#[$meta:meta])* $vis:vis static $name:ident) => {
        $(#[$meta])*
        $vis static $name: $crate::ws63::RadioStorage = {
            static CONTROL: $crate::ws63::__private::SleS1ControlStorage =
                $crate::ws63::__private::SleS1ControlStorage::new();
            static PROTOCOL: $crate::ws63::__private::ProtocolControlStorage =
                $crate::ws63::__private::ProtocolControlStorage::new();
            static EVENTS: $crate::ws63::__private::ProtocolEventStorage =
                $crate::ws63::__private::ProtocolEventStorage::new();
            static ANNOUNCE: $crate::ws63::__private::ProtocolLifecycleStorage =
                $crate::ws63::__private::ProtocolLifecycleStorage::new();
            static SEEK: $crate::ws63::__private::ProtocolLifecycleStorage =
                $crate::ws63::__private::ProtocolLifecycleStorage::new();
            #[cfg_attr(target_arch = "riscv32", unsafe(link_section = ".hisi.shared-arena"))]
            static ARENA: $crate::ws63::__private::ArenaStorage<
                { $crate::ws63::RADIO_ARENA_BYTES },
            > = $crate::ws63::__private::ArenaStorage::new();
            $crate::ws63::RadioStorage::__from_parts(
                &CONTROL,
                &ARENA,
                &PROTOCOL,
                &EVENTS,
                &ANNOUNCE,
                &SEEK,
            )
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
    type TestController = super::ws63::RadioController;
    type TestInit = fn(
        super::RadioConfig,
        super::ws63::Resources<TestProfile>,
        &'static TestStorage,
    ) -> Result<TestController, super::ws63::InitError>;

    #[test]
    fn facade_exposes_the_bounded_ws63_lifecycle() {
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
