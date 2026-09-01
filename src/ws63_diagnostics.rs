//! Versioned, allocation-free diagnostics for the WS63 facade.

use core::fmt;

#[cfg(feature = "incremental-backend-experiment")]
use crate::IncrementalRunnerDiagnostics;
use crate::{BlockingRunnerDiagnostics, EventDiagnostics};
#[cfg(feature = "incremental-embassy-wait")]
use hisi_rf_ws63::Ws63IncrementalWaitDiagnostics;
use hisi_rf_ws63::{
    BlockingBackendMetrics, DataPathDiagnostics, DhcpDiagnostics, L2ProtocolDiagnostics,
    RxQueueDiagnostics, ScanDiagnostics,
};

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
        device: &hisi_rf_ws63::WifiDevice,
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
            blocking_calls: hisi_rf_ws63::blocking_backend_metrics(),
            scan: hisi_rf_ws63::upstream_supplicant_scan_diagnostics(),
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
        device: &hisi_rf_ws63::WifiDevice,
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
            blocking_calls: hisi_rf_ws63::blocking_backend_metrics(),
            scan: hisi_rf_ws63::upstream_supplicant_scan_diagnostics(),
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
