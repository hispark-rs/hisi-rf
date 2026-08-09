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
/// cancellation and active lifecycle guards remain experimental U4 work.
/// Applications must not bypass this facade to depend on internal stage APIs.
#[cfg(all(feature = "chip-ws63", feature = "profile-ble-dual-role"))]
pub mod ws63 {
    pub use crate::declare_radio_storage;

    #[doc(hidden)]
    pub enum BleCommand {
        StartAdvertising(crate::ble::AdvertisingConfig),
        StartScanning(crate::ble::ScanConfig),
        RegisterGattServer(crate::ble::GattServerDefinition),
    }

    type BleControlState =
        hisi_rf_core::control::ControlState<BleCommand, Result<BleOperation, BleOperationError>>;
    type BleEventState = hisi_rf_core::control::EventState<BleEvent, { crate::EVENT_CAPACITY }>;

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
    }

    /// Caller-owned BLE composition storage.
    pub struct RadioStorage {
        inner: hisi_rf_ws63::BleB1Storage<RADIO_ARENA_BYTES>,
        control: &'static BleControlState,
        events: &'static BleEventState,
    }

    impl RadioStorage {
        /// Join the statically allocated control state and arena.
        #[doc(hidden)]
        pub const fn __from_parts(
            control: &'static __private::BleB1ControlStorage,
            arena: &'static __private::ArenaStorage<RADIO_ARENA_BYTES>,
            protocol: &'static __private::ProtocolControlStorage,
            events: &'static __private::ProtocolEventStorage,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::BleB1Storage::from_parts(control, arena),
                control: protocol,
                events,
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
                })
                .map_err(|_| InitError::new())
        }
    }

    /// Installed BLE storage capability.
    pub struct InstalledRadioStorage {
        inner: hisi_rf_ws63::InstalledBleB1Storage,
        control: &'static BleControlState,
        events: &'static BleEventState,
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
            trng: hisi_hal::peripherals::Trng<'static>,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::BleB1Resources::new(efuse, km, spacc, trng),
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
        /// The static GATT database could not be registered.
        GattDatabase,
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
    }

    /// Facade-owned BLE lifecycle event copied out of vendor callback context.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum BleEvent {
        /// Advertising entered the active state for this command generation.
        AdvertisingStarted {
            /// Command that requested this advertising lifecycle.
            operation: crate::ProtocolCommandId,
        },
        /// Scanning became active for this command generation.
        ScanReady {
            /// Command that requested this scanning lifecycle.
            operation: crate::ProtocolCommandId,
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

    #[derive(Default)]
    struct BleLifecycles {
        advertising: Option<crate::ProtocolCommandId>,
        scanning: Option<crate::ProtocolCommandId>,
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
        fn register_gatt_server(
            &mut self,
            definition: crate::ble::GattServerDefinition,
        ) -> Result<GattServerHandle, BleOperationError>;
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

    fn execute_ble_command(
        backend: &mut impl BleBackend,
        command: BleCommand,
    ) -> Result<BleOperation, BleOperationError> {
        match command {
            BleCommand::StartAdvertising(config) => backend
                .start_advertising(config)
                .map(|()| BleOperation::AdvertisingRequested),
            BleCommand::StartScanning(config) => backend
                .start_scanning(config)
                .map(|()| BleOperation::ScanningRequested),
            BleCommand::RegisterGattServer(definition) => backend
                .register_gatt_server(definition)
                .map(BleOperation::GattServerRegistered),
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
        let lifecycle = match &command {
            BleCommand::StartAdvertising(_) => Some(0),
            BleCommand::StartScanning(_) => Some(1),
            BleCommand::RegisterGattServer(_) => None,
        };
        let result = match readiness {
            Ok(true) => execute_ble_command(backend, command),
            Err(error) => Err(error),
            Ok(false) => unreachable!(),
        };
        if result.is_ok() {
            let id = crate::ProtocolCommandId(id);
            match lifecycle {
                Some(0) => lifecycles.advertising = Some(id),
                Some(1) => lifecycles.scanning = Some(id),
                _ => {}
            }
        }
        receiver
            .complete(id, result)
            .map_err(|_| crate::ProtocolError::CompletionOwnership)?;
        Ok(true)
    }

    fn map_ble_lifecycle_event(
        event: hisi_rf_ws63::BleB2Event,
        lifecycles: &BleLifecycles,
    ) -> Option<BleEvent> {
        match event {
            hisi_rf_ws63::BleB2Event::AdvertisingState { status: 1, .. } => lifecycles
                .advertising
                .map(|operation| BleEvent::AdvertisingStarted { operation }),
            hisi_rf_ws63::BleB2Event::AdvertisingState { status, .. } => {
                Some(BleEvent::BackendError {
                    operation: lifecycles.advertising,
                    stage: 1,
                    status,
                })
            }
            hisi_rf_ws63::BleB2Event::ScanParameters { status: 0 } => lifecycles
                .scanning
                .map(|operation| BleEvent::ScanReady { operation }),
            hisi_rf_ws63::BleB2Event::ScanParameters { status } => Some(BleEvent::BackendError {
                operation: lifecycles.scanning,
                stage: 2,
                status,
            }),
            hisi_rf_ws63::BleB2Event::Enabled { status } if status != 0 => {
                Some(BleEvent::BackendError {
                    operation: None,
                    stage: 0,
                    status,
                })
            }
            _ => None,
        }
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
                    lifecycles: BleLifecycles::default(),
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
                    BleCommand::StartScanning(_) | BleCommand::RegisterGattServer(_) => {
                        unreachable!()
                    }
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
                    BleCommand::StartAdvertising(_) | BleCommand::RegisterGattServer(_) => {
                        unreachable!()
                    }
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
                    BleCommand::StartAdvertising(_) | BleCommand::StartScanning(_) => {
                        unreachable!()
                    }
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
            let event = map_ble_lifecycle_event(event, &self.lifecycles);
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
        hisi_rf_ws63::init_ble_b1(resources.inner, storage.inner)
            .map(|inner| RadioController {
                inner,
                sender,
                receiver,
                event_producer,
                event_consumer,
            })
            .map_err(|_| InitError::new())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::boxed::Box;

        #[derive(Default)]
        struct FakeBackend {
            advertising: usize,
            scanning: usize,
            gatt_servers: usize,
            ready: bool,
            enable_error: Option<u32>,
            reject_scanning: bool,
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
            let mut lifecycles = BleLifecycles::default();
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
                &lifecycles,
            )
            .unwrap();
            producer.try_publish(event).unwrap();
            assert_eq!(
                controller.try_next_event(),
                Some(BleEvent::AdvertisingStarted { operation: id })
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
            let event = BleEvent::BackendError {
                operation: None,
                stage: 0,
                status: 1,
            };
            for _ in 0..crate::EVENT_CAPACITY {
                producer.try_publish(event).unwrap();
            }
            assert!(producer.try_publish(event).is_err());

            let id = controller.try_start_advertising(advertising()).unwrap();
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = BleLifecycles::default();
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
            let mut lifecycles = BleLifecycles::default();
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
    }
}

/// WS63 SLE U4 composition preview.
///
/// This profile establishes facade-owned storage, initialization, typed
/// announce/seek command submission, and runner ownership. The returned
/// completion means the WS63 host synchronously accepted or rejected the
/// request. Static typed SSAP registration and bounded asynchronous lifecycle
/// events are supported; cancellation and active guards remain experimental
/// U4 work.
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
    }

    /// Caller-owned SLE composition storage.
    pub struct RadioStorage {
        inner: hisi_rf_ws63::SleS1Storage<RADIO_ARENA_BYTES>,
        control: &'static SleControlState,
        events: &'static SleEventState,
    }

    impl RadioStorage {
        /// Join the statically allocated control state and arena.
        #[doc(hidden)]
        pub const fn __from_parts(
            control: &'static __private::SleS1ControlStorage,
            arena: &'static __private::ArenaStorage<RADIO_ARENA_BYTES>,
            protocol: &'static __private::ProtocolControlStorage,
            events: &'static __private::ProtocolEventStorage,
        ) -> Self {
            Self {
                inner: hisi_rf_ws63::SleS1Storage::from_parts(control, arena),
                control: protocol,
                events,
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
                })
                .map_err(|_| InitError::new())
        }
    }

    /// Installed SLE storage capability.
    pub struct InstalledRadioStorage {
        inner: hisi_rf_ws63::InstalledSleS1Storage,
        control: &'static SleControlState,
        events: &'static SleEventState,
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

    /// Facade-owned SLE lifecycle event copied out of vendor callback context.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum SleEvent {
        /// Announcing entered the active state for this command generation.
        AnnounceStarted {
            /// Command that requested this announce lifecycle.
            operation: crate::ProtocolCommandId,
        },
        /// Seeking entered the active state for this command generation.
        SeekReady {
            /// Command that requested this seek lifecycle.
            operation: crate::ProtocolCommandId,
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

    #[derive(Default)]
    struct SleLifecycles {
        announce: Option<crate::ProtocolCommandId>,
        seek: Option<crate::ProtocolCommandId>,
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
            E::UnsupportedTarget => (SleOperationErrorKind::UnsupportedTarget, None),
            E::StopSeek(status)
            | E::SetLocalAddress(status)
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

    fn execute_sle_command(
        backend: &mut impl SleBackend,
        command: SleCommand,
    ) -> Result<SleOperation, SleOperationError> {
        match command {
            SleCommand::StartAnnounce(config) => backend
                .start_announce(config)
                .map(|()| SleOperation::AnnounceRequested),
            SleCommand::StartSeek(config) => backend
                .start_seek(config)
                .map(|()| SleOperation::SeekRequested),
            SleCommand::RegisterSsapServer(definition) => backend
                .register_ssap_server(definition)
                .map(SleOperation::SsapServerRegistered),
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
        let lifecycle = match &command {
            SleCommand::StartAnnounce(_) => Some(0),
            SleCommand::StartSeek(_) => Some(1),
            SleCommand::RegisterSsapServer(_) => None,
        };
        let result = match readiness {
            Ok(true) => execute_sle_command(backend, command),
            Err(error) => Err(error),
            Ok(false) => unreachable!(),
        };
        if result.is_ok() {
            let id = crate::ProtocolCommandId(id);
            match lifecycle {
                Some(0) => lifecycles.announce = Some(id),
                Some(1) => lifecycles.seek = Some(id),
                _ => {}
            }
        }
        receiver
            .complete(id, result)
            .map_err(|_| crate::ProtocolError::CompletionOwnership)?;
        Ok(true)
    }

    fn map_sle_lifecycle_event(
        event: hisi_rf_ws63::SleS1Event,
        lifecycles: &SleLifecycles,
    ) -> Option<SleEvent> {
        match event {
            hisi_rf_ws63::SleS1Event::AnnounceEnabled { status: 0, .. } => lifecycles
                .announce
                .map(|operation| SleEvent::AnnounceStarted { operation }),
            hisi_rf_ws63::SleS1Event::AnnounceEnabled { status, .. } => {
                Some(SleEvent::BackendError {
                    operation: lifecycles.announce,
                    stage: 1,
                    status,
                })
            }
            hisi_rf_ws63::SleS1Event::SeekEnabled { status: 0 } => lifecycles
                .seek
                .map(|operation| SleEvent::SeekReady { operation }),
            hisi_rf_ws63::SleS1Event::SeekEnabled { status } => Some(SleEvent::BackendError {
                operation: lifecycles.seek,
                stage: 2,
                status,
            }),
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
                    lifecycles: SleLifecycles::default(),
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
            run_sle_once(&mut self.receiver, &mut self.inner, &mut self.lifecycles)
        }

        /// Copy at most one backend lifecycle event into the public queue.
        pub fn run_event_once(&mut self) -> bool {
            let Some(event) = self.inner.next_event() else {
                return false;
            };
            let event = map_sle_lifecycle_event(event, &self.lifecycles);
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
        hisi_rf_ws63::init_sle_s1(resources.inner, storage.inner)
            .map(|inner| RadioController {
                inner,
                sender,
                receiver,
                event_producer,
                event_consumer,
            })
            .map_err(|_| InitError::new())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::boxed::Box;

        #[derive(Default)]
        struct FakeBackend {
            announce: usize,
            seek: usize,
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
            let mut lifecycles = SleLifecycles::default();
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
                &lifecycles,
            )
            .unwrap();
            producer.try_publish(event).unwrap();
            assert_eq!(
                controller.try_next_event(),
                Some(SleEvent::AnnounceStarted { operation: id })
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
            let event = SleEvent::BackendError {
                operation: None,
                stage: 0,
                status: 1,
            };
            for _ in 0..crate::EVENT_CAPACITY {
                producer.try_publish(event).unwrap();
            }
            assert!(producer.try_publish(event).is_err());

            let id = controller.try_start_announce(announce()).unwrap();
            let mut backend = FakeBackend {
                ready: true,
                ..FakeBackend::default()
            };
            let mut lifecycles = SleLifecycles::default();
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
            let mut lifecycles = SleLifecycles::default();
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
            #[cfg_attr(target_arch = "riscv32", unsafe(link_section = ".hisi.shared-arena"))]
            static ARENA: $crate::ws63::__private::ArenaStorage<
                { $crate::ws63::RADIO_ARENA_BYTES },
            > = $crate::ws63::__private::ArenaStorage::new();
            $crate::ws63::RadioStorage::__from_parts(&CONTROL, &ARENA, &PROTOCOL, &EVENTS)
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
            #[cfg_attr(target_arch = "riscv32", unsafe(link_section = ".hisi.shared-arena"))]
            static ARENA: $crate::ws63::__private::ArenaStorage<
                { $crate::ws63::RADIO_ARENA_BYTES },
            > = $crate::ws63::__private::ArenaStorage::new();
            $crate::ws63::RadioStorage::__from_parts(&CONTROL, &ARENA, &PROTOCOL, &EVENTS)
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
