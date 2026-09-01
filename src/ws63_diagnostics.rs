//! Versioned, allocation-free diagnostics for the WS63 facade.

use core::fmt;

#[cfg(feature = "incremental-backend-experiment")]
use crate::IncrementalRunnerDiagnostics;
use crate::{BlockingRunnerDiagnostics, EventDiagnostics};
#[cfg(feature = "incremental-embassy-wait")]
use hisi_rf_ws63::Ws63IncrementalWaitDiagnostics;

/// Versioned, allocation-free resource contract for the selected WS63 profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceReport {
    /// Report schema consumed by CI and tooling.
    pub schema: &'static str,
    /// Selected chip backend.
    pub chip: &'static str,
    /// Selected named profile.
    pub profile: &'static str,
    /// Profile metadata revision.
    pub profile_revision: &'static str,
    /// Security backend selected by the profile.
    pub security: &'static str,
    /// Network adapter selected by the profile.
    pub network: &'static str,
    /// Radio integration backend.
    pub radio_backend: &'static str,
    /// Supplicant implementation.
    pub supplicant_backend: &'static str,
    /// Cryptographic backend.
    pub crypto_backend: &'static str,
    /// Minimum runtime contract required before startup.
    pub runtime_contract: &'static str,
    /// Task admission mechanism.
    pub task_admission: &'static str,
    /// Number of bounded public events.
    pub event_capacity: usize,
    /// Total caller-owned bytes.
    pub caller_owned_bytes: usize,
    /// Bytes held by bounded control and crypto state.
    pub control_storage_bytes: usize,
    /// Writable bytes used by the immutable composition handle.
    pub composition_handle_bytes: usize,
    /// Bytes used by chip-neutral radio state.
    pub radio_state_bytes: usize,
    /// Bytes used by SPACC DMA scratch.
    pub crypto_dma_bytes: usize,
    /// Bytes reserved by the shared arena backing object.
    pub arena_storage_bytes: usize,
    /// Linker-owned Wi-Fi packet RAM bytes.
    pub linker_packet_ram_bytes: usize,
    /// HIL-verified main-stack envelope.
    pub main_stack_bytes_required: usize,
    /// Dynamic tasks required by the selected profile.
    pub dynamic_tasks_required: usize,
    /// Vendor task slots from the pinned archive inventory.
    pub vendor_task_slots: usize,
    /// Stack bytes reserved for each vendor task.
    pub vendor_stack_bytes_per_task: usize,
    /// Incremental-worker slots, when present.
    pub worker_task_slots: Option<usize>,
    /// Stack bytes reserved for each incremental worker.
    pub worker_stack_bytes_per_task: Option<usize>,
    /// Dynamic task slots owned by a coexisting stack.
    pub coexistence_task_slots: usize,
    /// Stack bytes owned by a coexisting stack.
    pub coexistence_stack_bytes: usize,
    /// Runtime-internal task count, when modeled.
    pub runtime_internal_tasks: Option<usize>,
    /// Total task-stack bytes, when profile-owned.
    pub task_stack_bytes: Option<usize>,
    /// Smallest admitted task stack.
    pub minimum_task_stack_bytes: Option<usize>,
    /// RTOS synchronization-object headroom.
    pub runtime_object_headroom_bytes: Option<usize>,
    /// Scheduler arena bytes.
    pub runtime_arena_bytes: Option<usize>,
    /// Supplicant arena bytes, when independently owned.
    pub supplicant_arena_bytes: Option<usize>,
    /// Shared RF arena bytes.
    pub shared_rf_arena_bytes: Option<usize>,
    /// Final linked flash bytes, when supplied by image tooling.
    pub flash_bytes: Option<usize>,
    /// Whether task, stack, and arena totals completed HIL calibration.
    pub runtime_resources_calibrated: bool,
}

impl ResourceReport {
    pub(crate) const fn from_backend(value: hisi_rf_ws63::ResourceReport) -> Self {
        Self {
            schema: value.schema,
            chip: value.chip,
            profile: value.profile,
            profile_revision: value.profile_revision,
            security: value.security,
            network: value.network,
            radio_backend: value.radio_backend,
            supplicant_backend: value.supplicant_backend,
            crypto_backend: value.crypto_backend,
            runtime_contract: value.runtime_contract,
            task_admission: value.task_admission,
            event_capacity: value.event_capacity,
            caller_owned_bytes: value.caller_owned_bytes,
            control_storage_bytes: value.control_storage_bytes,
            composition_handle_bytes: value.composition_handle_bytes,
            radio_state_bytes: value.radio_state_bytes,
            crypto_dma_bytes: value.crypto_dma_bytes,
            arena_storage_bytes: value.arena_storage_bytes,
            linker_packet_ram_bytes: value.linker_packet_ram_bytes,
            main_stack_bytes_required: value.main_stack_bytes_required,
            dynamic_tasks_required: value.dynamic_tasks_required,
            vendor_task_slots: value.vendor_task_slots,
            vendor_stack_bytes_per_task: value.vendor_stack_bytes_per_task,
            worker_task_slots: value.worker_task_slots,
            worker_stack_bytes_per_task: value.worker_stack_bytes_per_task,
            coexistence_task_slots: value.coexistence_task_slots,
            coexistence_stack_bytes: value.coexistence_stack_bytes,
            runtime_internal_tasks: value.runtime_internal_tasks,
            task_stack_bytes: value.task_stack_bytes,
            minimum_task_stack_bytes: value.minimum_task_stack_bytes,
            runtime_object_headroom_bytes: value.runtime_object_headroom_bytes,
            runtime_arena_bytes: value.runtime_arena_bytes,
            supplicant_arena_bytes: value.supplicant_arena_bytes,
            shared_rf_arena_bytes: value.shared_rf_arena_bytes,
            flash_bytes: value.flash_bytes,
            runtime_resources_calibrated: value.runtime_resources_calibrated,
        }
    }

    /// Write deterministic JSON without allocation.
    pub fn write_json(self, output: &mut impl fmt::Write) -> fmt::Result {
        write!(
            output,
            concat!(
                "{{\"schema\":\"{}\",\"chip\":\"{}\",\"profile\":\"{}\",",
                "\"profile_revision\":\"{}\",\"security\":\"{}\",",
                "\"network\":\"{}\",\"radio_backend\":\"{}\",",
                "\"supplicant_backend\":\"{}\",\"crypto_backend\":\"{}\",",
                "\"runtime_contract\":\"{}\",\"task_admission\":\"{}\",",
                "\"event_capacity\":{},\"caller_owned_bytes\":{},",
                "\"control_storage_bytes\":{},\"composition_handle_bytes\":{},",
                "\"radio_state_bytes\":{},\"crypto_dma_bytes\":{},",
                "\"arena_storage_bytes\":{},\"linker_packet_ram_bytes\":{},",
                "\"main_stack_bytes_required\":{},\"dynamic_tasks_required\":{},",
                "\"vendor_task_slots\":{},\"vendor_stack_bytes_per_task\":{},",
                "\"worker_task_slots\":{},\"worker_stack_bytes_per_task\":{},",
                "\"coexistence_task_slots\":{},\"coexistence_stack_bytes\":{},",
                "\"runtime_internal_tasks\":{},\"task_stack_bytes\":{},",
                "\"minimum_task_stack_bytes\":{},\"runtime_object_headroom_bytes\":{},",
                "\"runtime_arena_bytes\":{},\"supplicant_arena_bytes\":null,",
                "\"shared_rf_arena_bytes\":{},\"flash_bytes\":null,",
                "\"runtime_resources_calibrated\":{}}}"
            ),
            self.schema,
            self.chip,
            self.profile,
            self.profile_revision,
            self.security,
            self.network,
            self.radio_backend,
            self.supplicant_backend,
            self.crypto_backend,
            self.runtime_contract,
            self.task_admission,
            self.event_capacity,
            self.caller_owned_bytes,
            self.control_storage_bytes,
            self.composition_handle_bytes,
            self.radio_state_bytes,
            self.crypto_dma_bytes,
            self.arena_storage_bytes,
            self.linker_packet_ram_bytes,
            self.main_stack_bytes_required,
            self.dynamic_tasks_required,
            self.vendor_task_slots,
            self.vendor_stack_bytes_per_task,
            self.worker_task_slots.unwrap_or(0),
            self.worker_stack_bytes_per_task.unwrap_or(0),
            self.coexistence_task_slots,
            self.coexistence_stack_bytes,
            self.runtime_internal_tasks.unwrap_or(0),
            self.task_stack_bytes.unwrap_or(0),
            self.minimum_task_stack_bytes.unwrap_or(0),
            self.runtime_object_headroom_bytes.unwrap_or(0),
            self.runtime_arena_bytes.unwrap_or(0),
            self.shared_rf_arena_bytes.unwrap_or(0),
            self.runtime_resources_calibrated,
        )
    }
}

/// Versioned schema for a complete public WS63 radio diagnostic snapshot.
pub const RADIO_DIAGNOSTICS_SCHEMA: &str = "hisi-rf-radio-diagnostics/v8";

/// Runner counters selected by the active facade profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunnerDiagnosticsSnapshot {
    /// Counters from the compatibility blocking runner.
    Blocking(BlockingRunnerDiagnostics),
    /// Counters from the bounded incremental runner.
    #[cfg(feature = "incremental-backend-experiment")]
    Incremental(IncrementalRunnerDiagnostics),
}

/// Secret-free view of the WS63 wait bridge.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaitDiagnosticsSnapshot {
    /// Whether this profile owns the explicit incremental wait bridge.
    pub active: bool,
    /// Native supplicant or vendor callback signal calls.
    pub backend_signals: u32,
    /// L2 receive signal calls.
    pub l2_rx_signals: u32,
    /// Calls made to the executor waker after recording a signal.
    pub waker_notifications: u32,
    /// Polls of the platform wait contract.
    pub poll_calls: u32,
    /// Polls that returned `Pending`.
    pub pending_polls: u32,
    /// Polls that returned at least one ready source.
    pub ready_polls: u32,
    /// Ready polls containing the monotonic timer source.
    pub timer_ready_polls: u32,
}

#[cfg(feature = "incremental-embassy-wait")]
impl WaitDiagnosticsSnapshot {
    pub(crate) fn from_backend(value: Ws63IncrementalWaitDiagnostics) -> Self {
        Self {
            active: true,
            backend_signals: value.backend_signals,
            l2_rx_signals: value.l2_rx_signals,
            waker_notifications: value.waker_notifications,
            poll_calls: value.poll_calls,
            pending_polls: value.pending_polls,
            ready_polls: value.ready_polls,
            timer_ready_polls: value.timer_ready_polls,
        }
    }
}

/// Ordered stages in the one-shot WS63 Wi-Fi bootstrap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BootstrapStage {
    /// Consume the uniquely owned HAL resources.
    ResourceClaim = 0,
    /// Install the selected cryptographic backend.
    CryptoInstall = 1,
    /// Run enabled cryptographic startup self-tests.
    CryptoSelfTest = 2,
    /// Prepare vendor RAM and linker-owned state.
    VendorMemoryPrepare = 3,
    /// Initialize the ROM monotonic timebase.
    RomTimebaseInitialize = 4,
    /// Start the vendor Wi-Fi runtime.
    VendorWifiInitialize = 5,
    /// Create the station network device.
    StationDeviceCreate = 6,
    /// Register bounded event delivery.
    EventRegistration = 7,
    /// Open the station data path.
    StationDeviceOpen = 8,
    /// Install the upstream supplicant port.
    SupplicantPortPrepare = 9,
    /// Create the upstream supplicant context.
    NativeSupplicantCreate = 10,
}

impl BootstrapStage {
    /// Stages in execution order.
    pub const ALL: [Self; 11] = [
        Self::ResourceClaim,
        Self::CryptoInstall,
        Self::CryptoSelfTest,
        Self::VendorMemoryPrepare,
        Self::RomTimebaseInitialize,
        Self::VendorWifiInitialize,
        Self::StationDeviceCreate,
        Self::EventRegistration,
        Self::StationDeviceOpen,
        Self::SupplicantPortPrepare,
        Self::NativeSupplicantCreate,
    ];

    /// Stable machine-readable stage name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceClaim => "resource_claim",
            Self::CryptoInstall => "crypto_install",
            Self::CryptoSelfTest => "crypto_self_test",
            Self::VendorMemoryPrepare => "vendor_memory_prepare",
            Self::RomTimebaseInitialize => "rom_timebase_initialize",
            Self::VendorWifiInitialize => "vendor_wifi_initialize",
            Self::StationDeviceCreate => "station_device_create",
            Self::EventRegistration => "event_registration",
            Self::StationDeviceOpen => "station_device_open",
            Self::SupplicantPortPrepare => "supplicant_port_prepare",
            Self::NativeSupplicantCreate => "native_supplicant_create",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }

    const fn backend(self) -> hisi_rf_ws63::BootstrapStage {
        match self {
            Self::ResourceClaim => hisi_rf_ws63::BootstrapStage::ResourceClaim,
            Self::CryptoInstall => hisi_rf_ws63::BootstrapStage::CryptoInstall,
            Self::CryptoSelfTest => hisi_rf_ws63::BootstrapStage::CryptoSelfTest,
            Self::VendorMemoryPrepare => hisi_rf_ws63::BootstrapStage::VendorMemoryPrepare,
            Self::RomTimebaseInitialize => hisi_rf_ws63::BootstrapStage::RomTimebaseInitialize,
            Self::VendorWifiInitialize => hisi_rf_ws63::BootstrapStage::VendorWifiInitialize,
            Self::StationDeviceCreate => hisi_rf_ws63::BootstrapStage::StationDeviceCreate,
            Self::EventRegistration => hisi_rf_ws63::BootstrapStage::EventRegistration,
            Self::StationDeviceOpen => hisi_rf_ws63::BootstrapStage::StationDeviceOpen,
            Self::SupplicantPortPrepare => hisi_rf_ws63::BootstrapStage::SupplicantPortPrepare,
            Self::NativeSupplicantCreate => hisi_rf_ws63::BootstrapStage::NativeSupplicantCreate,
        }
    }
}

/// Per-stage bootstrap counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BootstrapStageMetrics {
    pub calls: u32,
    pub completed_calls: u32,
    pub failed_calls: u32,
    pub timed_calls: u32,
    pub max_elapsed_ms: u32,
}

impl BootstrapStageMetrics {
    const fn from_backend(value: hisi_rf_ws63::BootstrapStageMetrics) -> Self {
        Self {
            calls: value.calls,
            completed_calls: value.completed_calls,
            failed_calls: value.failed_calls,
            timed_calls: value.timed_calls,
            max_elapsed_ms: value.max_elapsed_ms,
        }
    }
}

/// Stage-by-stage bootstrap counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlockingBootstrapMetrics {
    stages: [BootstrapStageMetrics; 11],
}

impl BlockingBootstrapMetrics {
    /// Return counters for one bootstrap stage.
    pub fn stage(&self, stage: BootstrapStage) -> BootstrapStageMetrics {
        self.stages[stage.index()]
    }

    fn from_backend(value: hisi_rf_ws63::BlockingBootstrapMetrics) -> Self {
        let mut stages = [BootstrapStageMetrics::default(); 11];
        for stage in BootstrapStage::ALL {
            stages[stage.index()] =
                BootstrapStageMetrics::from_backend(value.stage(stage.backend()));
        }
        Self { stages }
    }
}

/// Per-operation blocking-call counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlockingOperationMetrics {
    pub calls: u32,
    pub timed_calls: u32,
    pub max_elapsed_ms: u32,
}

impl BlockingOperationMetrics {
    const fn from_backend(value: hisi_rf_ws63::BlockingOperationMetrics) -> Self {
        Self {
            calls: value.calls,
            timed_calls: value.timed_calls,
            max_elapsed_ms: value.max_elapsed_ms,
        }
    }
}

/// Counter-only timings for synchronous host-to-HMAC messages.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FrwSyncPostMetrics {
    pub calls: u32,
    pub last_msg_id: u32,
    pub last_timeout_ms: u32,
    pub last_elapsed_ms: u32,
    pub last_result: u32,
    pub max_msg_id: u32,
    pub max_elapsed_ms: u32,
    pub last_wait_blocks: u32,
    pub last_wait_wakeups: u32,
    pub last_wait_ready_checks: u32,
}

impl FrwSyncPostMetrics {
    const fn from_backend(value: hisi_rf_ws63::FrwSyncPostMetrics) -> Self {
        Self {
            calls: value.calls,
            last_msg_id: value.last_msg_id,
            last_timeout_ms: value.last_timeout_ms,
            last_elapsed_ms: value.last_elapsed_ms,
            last_result: value.last_result,
            max_msg_id: value.max_msg_id,
            max_elapsed_ms: value.max_elapsed_ms,
            last_wait_blocks: value.last_wait_blocks,
            last_wait_wakeups: value.last_wait_wakeups,
            last_wait_ready_checks: value.last_wait_ready_checks,
        }
    }
}

/// Snapshot of the selected WS63 blocking workload.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlockingBackendMetrics {
    pub bootstrap: BlockingBootstrapMetrics,
    pub initialize: BlockingOperationMetrics,
    pub scan: BlockingOperationMetrics,
    pub connect: BlockingOperationMetrics,
    pub disconnect: BlockingOperationMetrics,
    pub poll: BlockingOperationMetrics,
    pub internal_sleep_calls: u32,
    pub supplicant_poll_calls: u32,
    pub frw_sync_post: FrwSyncPostMetrics,
}

impl BlockingBackendMetrics {
    pub(crate) fn from_backend(value: hisi_rf_ws63::BlockingBackendMetrics) -> Self {
        Self {
            bootstrap: BlockingBootstrapMetrics::from_backend(value.bootstrap),
            initialize: BlockingOperationMetrics::from_backend(value.initialize),
            scan: BlockingOperationMetrics::from_backend(value.scan),
            connect: BlockingOperationMetrics::from_backend(value.connect),
            disconnect: BlockingOperationMetrics::from_backend(value.disconnect),
            poll: BlockingOperationMetrics::from_backend(value.poll),
            internal_sleep_calls: value.internal_sleep_calls,
            supplicant_poll_calls: value.supplicant_poll_calls,
            frw_sync_post: FrwSyncPostMetrics::from_backend(value.frw_sync_post),
        }
    }
}

/// Secret-free scan/callback state captured at an operation boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScanDiagnostics {
    pub native_starts: u32,
    pub native_results: u32,
    pub native_done: u32,
    pub native_active: bool,
    pub queue_pending: bool,
    pub queue_dropped: u32,
    pub native_start_ms: u32,
    pub native_done_ms: u32,
    pub driver_active: bool,
    pub driver_done: bool,
    pub driver_results: u32,
    pub driver_status: u32,
    pub driver_done_ms: u32,
}

impl ScanDiagnostics {
    pub(crate) const fn from_backend(value: hisi_rf_ws63::ScanDiagnostics) -> Self {
        Self {
            native_starts: value.native_starts,
            native_results: value.native_results,
            native_done: value.native_done,
            native_active: value.native_active,
            queue_pending: value.queue_pending,
            queue_dropped: value.queue_dropped,
            native_start_ms: value.native_start_ms,
            native_done_ms: value.native_done_ms,
            driver_active: value.driver_active,
            driver_done: value.driver_done,
            driver_results: value.driver_results,
            driver_status: value.driver_status,
            driver_done_ms: value.driver_done_ms,
        }
    }
}

/// Bounded L2 receive-queue counters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RxQueueDiagnostics {
    pub depth: usize,
    pub pending: usize,
    pub high_watermark: usize,
    pub dropped: u32,
    pub icmp_echo_replies: u32,
    pub icmp_sequence_mask: u32,
}

impl RxQueueDiagnostics {
    pub(crate) const fn from_backend(value: hisi_rf_ws63::RxQueueDiagnostics) -> Self {
        Self {
            depth: value.depth,
            pending: value.pending,
            high_watermark: value.high_watermark,
            dropped: value.dropped,
            icmp_echo_replies: value.icmp_echo_replies,
            icmp_sequence_mask: value.icmp_sequence_mask,
        }
    }
}

/// DHCP packet counts at the Rust-visible L2 seam.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DhcpDiagnostics {
    pub client_packets: u32,
    pub server_packets: u32,
}

impl DhcpDiagnostics {
    pub(crate) const fn from_backend(value: hisi_rf_ws63::DhcpDiagnostics) -> Self {
        Self {
            client_packets: value.client_packets,
            server_packets: value.server_packets,
        }
    }
}

/// Ethernet protocol counters at the Rust-visible L2 seam.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct L2ProtocolDiagnostics {
    pub rx_arp_requests: u32,
    pub rx_arp_replies: u32,
    pub rx_ipv4: u32,
    pub rx_other: u32,
    pub tx_arp_requests: u32,
    pub tx_arp_replies: u32,
    pub tx_ipv4: u32,
    pub tx_other: u32,
}

impl L2ProtocolDiagnostics {
    pub(crate) const fn from_backend(value: hisi_rf_ws63::L2ProtocolDiagnostics) -> Self {
        Self {
            rx_arp_requests: value.rx_arp_requests,
            rx_arp_replies: value.rx_arp_replies,
            rx_ipv4: value.rx_ipv4,
            rx_other: value.rx_other,
            tx_arp_requests: value.tx_arp_requests,
            tx_arp_replies: value.tx_arp_replies,
            tx_ipv4: value.tx_ipv4,
            tx_other: value.tx_other,
        }
    }
}

/// Bounded frame-submission and completion timeline.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TxTimelineDiagnostics {
    pub submission_total: u32,
    pub completion_total: u32,
    pub callback_total: u32,
    pub submissions: [u32; 18],
    pub submission_time_ms: [u32; 18],
    pub completions: [u32; 18],
    pub packet_number_lsb: [u32; 18],
    pub completion_time_ms: [u32; 18],
    pub completion_echo: [u32; 18],
}

impl TxTimelineDiagnostics {
    const fn from_backend(value: hisi_rf_ws63::TxTimelineDiagnostics) -> Self {
        Self {
            submission_total: value.submission_total,
            completion_total: value.completion_total,
            callback_total: value.callback_total,
            submissions: value.submissions,
            submission_time_ms: value.submission_time_ms,
            completions: value.completions,
            packet_number_lsb: value.packet_number_lsb,
            completion_time_ms: value.completion_time_ms,
            completion_echo: value.completion_echo,
        }
    }
}

/// Counter-only view spanning the Rust L2 bridge and vendor IRQ boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DataPathDiagnostics {
    pub instrumented_capabilities: u32,
    pub tx_frames: u32,
    pub tx_failed: u32,
    pub vendor_tx_frames: u32,
    pub tx_reference_diagnostics: [u32; 3],
    pub tx_completions: u32,
    pub tx_completion_status: [u32; 16],
    pub tx_timeline: TxTimelineDiagnostics,
    pub dmac_rx_prepares: u32,
    pub dmac_rx_prepare_zero: u32,
    pub dmac_rx_prepare_nonzero: u32,
    pub dmac_rx_prepare_last_result: u32,
    pub hmac_rx_data_event_adapt_calls: u32,
    pub hmac_rx_process_data_msg_calls: u32,
    pub hmac_rx_data_calls: u32,
    pub vendor_rx_frames: u32,
    pub rx_frames: u32,
    pub mac_rx_successful_mpdu: u32,
    pub mac_rx_failed_mpdu: u32,
    pub mac_rx_filtered_mpdu: u32,
    pub mac_ccmp_replay_failures: u32,
    pub mac_tkip_replay_failures: u32,
    pub mac_ccmp_mic_failures: u32,
    pub mac_tkip_mic_failures: u32,
    pub mac_key_search_failures: u32,
    pub mac_rx_filter_control: u32,
    pub mac_station_address_matches_device: bool,
    pub mac_bssid_programmed: bool,
    pub coex_wlan_irqs: u32,
    pub wlphy_irqs: u32,
    pub wlmac_irqs: u32,
    pub wlmac_irq_lifecycle: [u32; 6],
}

impl DataPathDiagnostics {
    pub(crate) const fn from_backend(value: hisi_rf_ws63::DataPathDiagnostics) -> Self {
        Self {
            instrumented_capabilities: value.instrumented_capabilities,
            tx_frames: value.tx_frames,
            tx_failed: value.tx_failed,
            vendor_tx_frames: value.vendor_tx_frames,
            tx_reference_diagnostics: value.tx_reference_diagnostics,
            tx_completions: value.tx_completions,
            tx_completion_status: value.tx_completion_status,
            tx_timeline: TxTimelineDiagnostics::from_backend(value.tx_timeline),
            dmac_rx_prepares: value.dmac_rx_prepares,
            dmac_rx_prepare_zero: value.dmac_rx_prepare_zero,
            dmac_rx_prepare_nonzero: value.dmac_rx_prepare_nonzero,
            dmac_rx_prepare_last_result: value.dmac_rx_prepare_last_result,
            hmac_rx_data_event_adapt_calls: value.hmac_rx_data_event_adapt_calls,
            hmac_rx_process_data_msg_calls: value.hmac_rx_process_data_msg_calls,
            hmac_rx_data_calls: value.hmac_rx_data_calls,
            vendor_rx_frames: value.vendor_rx_frames,
            rx_frames: value.rx_frames,
            mac_rx_successful_mpdu: value.mac_rx_successful_mpdu,
            mac_rx_failed_mpdu: value.mac_rx_failed_mpdu,
            mac_rx_filtered_mpdu: value.mac_rx_filtered_mpdu,
            mac_ccmp_replay_failures: value.mac_ccmp_replay_failures,
            mac_tkip_replay_failures: value.mac_tkip_replay_failures,
            mac_ccmp_mic_failures: value.mac_ccmp_mic_failures,
            mac_tkip_mic_failures: value.mac_tkip_mic_failures,
            mac_key_search_failures: value.mac_key_search_failures,
            mac_rx_filter_control: value.mac_rx_filter_control,
            mac_station_address_matches_device: value.mac_station_address_matches_device,
            mac_bssid_programmed: value.mac_bssid_programmed,
            coex_wlan_irqs: value.coex_wlan_irqs,
            wlphy_irqs: value.wlphy_irqs,
            wlmac_irqs: value.wlmac_irqs,
            wlmac_irq_lifecycle: value.wlmac_irq_lifecycle,
        }
    }
}

/// Facade-owned, allocation-free snapshot of one WS63 radio composition.
///
/// The snapshot contains only counters and static profile metadata. It never
/// contains credentials, station/BSSID identity, frame contents, keys, or
/// unbounded backend traces. The nested error and resource schemas remain
/// authoritative in `DIAGNOSTIC_SCHEMA` and [`ResourceReport::schema`].
///
/// Counters are observational and may advance while this function samples
/// them. One call nevertheless captures every public diagnostic source through
/// the owning radio parts, so applications do not assemble state through
/// process-global mutable cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RadioDiagnosticsSnapshot {
    /// Schema for this aggregate view.
    pub schema: &'static str,
    /// Existing typed-error schema referenced by this view.
    pub error_schema: &'static str,
    /// Selected profile's resource report.
    pub resources: ResourceReport,
    /// Control/event runner counters.
    pub runner: RunnerDiagnosticsSnapshot,
    /// Shared command-channel and blocking-runner migration counters.
    pub control: BlockingRunnerDiagnostics,
    /// Explicit wait-bridge counters, inactive for the blocking profile.
    pub wait: WaitDiagnosticsSnapshot,
    /// Public event queue occupancy and loss counters.
    pub events: EventDiagnostics,
    /// Compatibility backend call counts and timing bounds.
    pub blocking_calls: BlockingBackendMetrics,
    /// Native and vendor scan completion/callback counters.
    pub scan: ScanDiagnostics,
    /// Rust-visible L2 receive queue counters.
    pub rx_queue: RxQueueDiagnostics,
    /// DHCP packets observed at the Rust-visible L2 seam.
    pub dhcp: DhcpDiagnostics,
    /// Ethernet protocol classes observed at the Rust-visible L2 seam.
    pub l2_protocol: L2ProtocolDiagnostics,
    /// Aggregate frame progress and radio interrupt dispatch counters.
    pub data_path: DataPathDiagnostics,
}

impl RadioDiagnosticsSnapshot {
    #[cfg(not(feature = "incremental-backend-experiment"))]
    pub(crate) fn blocking(
        controller: &crate::WifiController,
        device: &crate::ws63_wifi::WifiDevice,
        resources: ResourceReport,
    ) -> Self {
        Self {
            schema: RADIO_DIAGNOSTICS_SCHEMA,
            error_schema: crate::DIAGNOSTIC_SCHEMA,
            resources,
            runner: RunnerDiagnosticsSnapshot::Blocking(controller.blocking_runner_diagnostics()),
            control: controller.blocking_runner_diagnostics(),
            wait: WaitDiagnosticsSnapshot::default(),
            events: controller.event_diagnostics(),
            blocking_calls: BlockingBackendMetrics::from_backend(
                hisi_rf_ws63::blocking_backend_metrics(),
            ),
            scan: ScanDiagnostics::from_backend(
                hisi_rf_ws63::upstream_supplicant_scan_diagnostics(),
            ),
            rx_queue: device.rx_queue_diagnostics(),
            dhcp: device.dhcp_diagnostics(),
            l2_protocol: device.l2_protocol_diagnostics(),
            data_path: device.data_path_diagnostics(),
        }
    }

    #[cfg(all(
        feature = "incremental-backend-experiment",
        feature = "incremental-embassy-wait"
    ))]
    pub(crate) fn incremental(
        controller: &crate::WifiController,
        device: &crate::ws63_wifi::WifiDevice,
        resources: ResourceReport,
    ) -> Self {
        Self {
            schema: RADIO_DIAGNOSTICS_SCHEMA,
            error_schema: crate::DIAGNOSTIC_SCHEMA,
            resources,
            runner: RunnerDiagnosticsSnapshot::Incremental(
                controller.incremental_runner_diagnostics(),
            ),
            control: controller.blocking_runner_diagnostics(),
            wait: WaitDiagnosticsSnapshot::from_backend(
                hisi_rf_ws63::incremental_wait_diagnostics(),
            ),
            events: controller.event_diagnostics(),
            blocking_calls: BlockingBackendMetrics::from_backend(
                hisi_rf_ws63::blocking_backend_metrics(),
            ),
            scan: ScanDiagnostics::from_backend(
                hisi_rf_ws63::upstream_supplicant_scan_diagnostics(),
            ),
            rx_queue: device.rx_queue_diagnostics(),
            dhcp: device.dhcp_diagnostics(),
            l2_protocol: device.l2_protocol_diagnostics(),
            data_path: device.data_path_diagnostics(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::String;

    #[test]
    fn schema_references_existing_error_and_resource_truth() {
        assert_eq!(RADIO_DIAGNOSTICS_SCHEMA, "hisi-rf-radio-diagnostics/v8");
        assert_eq!(crate::DIAGNOSTIC_SCHEMA, "hisi-rf-error/v3");
        let report = ResourceReport::from_backend(hisi_rf_ws63::resource_report::<
            hisi_rf_ws63::SelectedProfile,
            { crate::EVENT_CAPACITY },
        >());
        assert_eq!(report.schema, "hisi-rf-resource-report/v13");
        #[cfg(all(feature = "wpa2-personal", not(feature = "incremental-embassy-wait")))]
        assert!(report.runtime_resources_calibrated);
        #[cfg(any(feature = "wpa3-personal", feature = "incremental-embassy-wait"))]
        assert!(!report.runtime_resources_calibrated);
    }

    #[test]
    fn resource_report_projection_preserves_the_machine_contract() {
        let backend = hisi_rf_ws63::resource_report::<
            hisi_rf_ws63::SelectedProfile,
            { crate::EVENT_CAPACITY },
        >();
        let facade = ResourceReport::from_backend(backend);
        let mut backend_json = String::new();
        let mut facade_json = String::new();
        backend.write_json(&mut backend_json).unwrap();
        facade.write_json(&mut facade_json).unwrap();
        assert_eq!(facade_json, backend_json);
    }

    #[test]
    fn l2_diagnostic_projections_preserve_backend_values() {
        let rx = hisi_rf_ws63::RxQueueDiagnostics {
            depth: 16,
            pending: 7,
            high_watermark: 12,
            dropped: 3,
            icmp_echo_replies: 9,
            icmp_sequence_mask: 0x155,
        };
        let facade_rx = RxQueueDiagnostics::from_backend(rx);
        assert_eq!(facade_rx.depth, rx.depth);
        assert_eq!(facade_rx.pending, rx.pending);
        assert_eq!(facade_rx.high_watermark, rx.high_watermark);
        assert_eq!(facade_rx.dropped, rx.dropped);
        assert_eq!(facade_rx.icmp_echo_replies, rx.icmp_echo_replies);
        assert_eq!(facade_rx.icmp_sequence_mask, rx.icmp_sequence_mask);

        let dhcp = hisi_rf_ws63::DhcpDiagnostics {
            client_packets: 5,
            server_packets: 4,
        };
        let facade_dhcp = DhcpDiagnostics::from_backend(dhcp);
        assert_eq!(facade_dhcp.client_packets, dhcp.client_packets);
        assert_eq!(facade_dhcp.server_packets, dhcp.server_packets);

        let l2 = hisi_rf_ws63::L2ProtocolDiagnostics {
            rx_arp_requests: 1,
            rx_arp_replies: 2,
            rx_ipv4: 3,
            rx_other: 4,
            tx_arp_requests: 5,
            tx_arp_replies: 6,
            tx_ipv4: 7,
            tx_other: 8,
        };
        let facade_l2 = L2ProtocolDiagnostics::from_backend(l2);
        assert_eq!(facade_l2.rx_arp_requests, l2.rx_arp_requests);
        assert_eq!(facade_l2.rx_arp_replies, l2.rx_arp_replies);
        assert_eq!(facade_l2.rx_ipv4, l2.rx_ipv4);
        assert_eq!(facade_l2.rx_other, l2.rx_other);
        assert_eq!(facade_l2.tx_arp_requests, l2.tx_arp_requests);
        assert_eq!(facade_l2.tx_arp_replies, l2.tx_arp_replies);
        assert_eq!(facade_l2.tx_ipv4, l2.tx_ipv4);
        assert_eq!(facade_l2.tx_other, l2.tx_other);
    }

    #[test]
    fn data_path_projection_preserves_nested_and_edge_fields() {
        let backend = hisi_rf_ws63::DataPathDiagnostics {
            instrumented_capabilities: 0x3f,
            tx_frames: 11,
            tx_failed: 12,
            tx_reference_diagnostics: [13, 14, 15],
            tx_completion_status: [16; 16],
            tx_timeline: hisi_rf_ws63::TxTimelineDiagnostics {
                submission_total: 17,
                completion_total: 18,
                callback_total: 19,
                submissions: [20; 18],
                submission_time_ms: [21; 18],
                completions: [22; 18],
                packet_number_lsb: [23; 18],
                completion_time_ms: [24; 18],
                completion_echo: [25; 18],
            },
            mac_station_address_matches_device: true,
            mac_bssid_programmed: true,
            wlmac_irq_lifecycle: [26; 6],
            ..Default::default()
        };
        let facade = DataPathDiagnostics::from_backend(backend);
        assert_eq!(facade.instrumented_capabilities, 0x3f);
        assert_eq!(facade.tx_frames, 11);
        assert_eq!(facade.tx_failed, 12);
        assert_eq!(facade.tx_reference_diagnostics, [13, 14, 15]);
        assert_eq!(facade.tx_completion_status, [16; 16]);
        assert_eq!(facade.tx_timeline.submission_total, 17);
        assert_eq!(facade.tx_timeline.completion_total, 18);
        assert_eq!(facade.tx_timeline.callback_total, 19);
        assert_eq!(facade.tx_timeline.submissions, [20; 18]);
        assert_eq!(facade.tx_timeline.submission_time_ms, [21; 18]);
        assert_eq!(facade.tx_timeline.completions, [22; 18]);
        assert_eq!(facade.tx_timeline.packet_number_lsb, [23; 18]);
        assert_eq!(facade.tx_timeline.completion_time_ms, [24; 18]);
        assert_eq!(facade.tx_timeline.completion_echo, [25; 18]);
        assert!(facade.mac_station_address_matches_device);
        assert!(facade.mac_bssid_programmed);
        assert_eq!(facade.wlmac_irq_lifecycle, [26; 6]);
    }

    #[cfg(feature = "incremental-embassy-wait")]
    #[test]
    fn wait_snapshot_is_a_lossless_secret_free_projection() {
        let raw = Ws63IncrementalWaitDiagnostics {
            backend_signals: 1,
            l2_rx_signals: 2,
            waker_notifications: 3,
            poll_calls: 4,
            pending_polls: 5,
            ready_polls: 6,
            timer_ready_polls: 7,
        };
        let snapshot = WaitDiagnosticsSnapshot::from_backend(raw);
        assert!(snapshot.active);
        assert_eq!(snapshot.backend_signals, 1);
        assert_eq!(snapshot.l2_rx_signals, 2);
        assert_eq!(snapshot.waker_notifications, 3);
        assert_eq!(snapshot.poll_calls, 4);
        assert_eq!(snapshot.pending_polls, 5);
        assert_eq!(snapshot.ready_polls, 6);
        assert_eq!(snapshot.timer_ready_polls, 7);
    }
}
