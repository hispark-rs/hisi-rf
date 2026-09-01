//! Versioned, allocation-free diagnostics for the WS63 facade.

#[cfg(feature = "incremental-backend-experiment")]
use crate::IncrementalRunnerDiagnostics;
use crate::{BlockingRunnerDiagnostics, EventDiagnostics};
#[cfg(feature = "incremental-embassy-wait")]
use hisi_rf_ws63::Ws63IncrementalWaitDiagnostics;
use hisi_rf_ws63::{
    BlockingBackendMetrics, DataPathDiagnostics, DhcpDiagnostics, L2ProtocolDiagnostics,
    ResourceReport, RxQueueDiagnostics, ScanDiagnostics,
};

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

    #[test]
    fn schema_references_existing_error_and_resource_truth() {
        assert_eq!(RADIO_DIAGNOSTICS_SCHEMA, "hisi-rf-radio-diagnostics/v8");
        assert_eq!(crate::DIAGNOSTIC_SCHEMA, "hisi-rf-error/v3");
        let report = hisi_rf_ws63::resource_report::<
            hisi_rf_ws63::SelectedProfile,
            { crate::EVENT_CAPACITY },
        >();
        assert_eq!(report.schema, "hisi-rf-resource-report/v13");
        #[cfg(all(feature = "wpa2-personal", not(feature = "incremental-embassy-wait")))]
        assert!(report.runtime_resources_calibrated);
        #[cfg(any(feature = "wpa3-personal", feature = "incremental-embassy-wait"))]
        assert!(!report.runtime_resources_calibrated);
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
