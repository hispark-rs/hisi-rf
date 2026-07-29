#![no_std]
#![no_main]

use hisi_riscv_rt::entry;

struct DiagnosticSink;

impl core::fmt::Write for DiagnosticSink {
    fn write_str(&mut self, _: &str) -> core::fmt::Result {
        Ok(())
    }
}

#[allow(dead_code)]
fn check_blocking_runner_diagnostics(controller: &hisi_rf::WifiController) {
    let runner: hisi_rf::BlockingRunnerDiagnostics = controller.blocking_runner_diagnostics();
    let events: hisi_rf::EventDiagnostics = controller.event_diagnostics();
    let _migration_baseline = (
        runner.command_queue_pending,
        runner.command_queue_high_water,
        runner.run_once_calls,
        runner.commands_processed,
        runner.backend_poll_calls,
        runner.backend_poll_work_batches,
        runner.backend_poll_errors,
        runner.immediate_repoll_hints,
        events.high_water,
    );
}

#[allow(dead_code)]
fn check_ws63_blocking_backend_metrics() {
    let metrics: hisi_rf::ws63::BlockingBackendMetrics = hisi_rf::ws63::blocking_backend_metrics();
    let connect: hisi_rf::ws63::BlockingOperationMetrics = metrics.connect;
    let _migration_baseline = (
        connect.calls,
        connect.timed_calls,
        connect.max_elapsed_ms,
        metrics.internal_sleep_calls,
        metrics.supplicant_poll_calls,
    );
}

#[allow(dead_code)]
fn check_opaque_ws63_composition(controller: hisi_rf::ws63::RadioController) {
    let result: Result<hisi_rf::ws63::WifiParts, hisi_rf::ws63::InitError> =
        controller.start_runner();
    if let Err(error) = result {
        let _opaque_error_contract = (error.kind(), error.diagnostic());
    }
}

#[cfg(not(feature = "incremental-contract"))]
#[allow(dead_code)]
fn check_unified_blocking_diagnostics(parts: &hisi_rf::ws63::WifiParts) {
    let snapshot: hisi_rf::ws63::RadioDiagnosticsSnapshot = parts.diagnostics();
    let runner = match snapshot.runner {
        hisi_rf::ws63::RunnerDiagnosticsSnapshot::Blocking(runner) => runner,
    };
    let _secret_free_contract = (
        snapshot.schema,
        snapshot.error_schema,
        snapshot.resources.schema,
        snapshot.resources.profile_revision,
        runner.run_once_calls,
        snapshot.wait.active,
        snapshot.events.dropped,
        snapshot.blocking_calls.connect.calls,
        snapshot.rx_queue.dropped,
        snapshot.dhcp.client_packets,
    );
}

hisi_rf::ws63::declare_radio_storage!(static RADIO_STORAGE);

#[cfg(feature = "incremental-contract")]
#[allow(dead_code)]
fn check_incremental_contract<B: hisi_rf::IncrementalWifiBackend>(backend: B) {
    let budget = hisi_rf::WorkBudget::try_new(4, 100).expect("non-zero work budget");
    let _driver = hisi_rf::IncrementalBackendDriver::new(backend, budget);
    let sequence = hisi_rf::CommandSequence::try_from_raw(1).expect("non-zero sequence");
    let mut arbiter = hisi_rf::CommandArbiter::new();
    arbiter
        .submit(hisi_rf::PendingCommand::new(
            sequence,
            hisi_rf::IncrementalRequest::Initialize(hisi_rf::WifiConfig::default()),
        ))
        .expect("empty bounded arbiter");
}

#[cfg(feature = "incremental-contract")]
struct ExternalWaitPlatform;

#[cfg(feature = "incremental-contract")]
impl hisi_rf::IncrementalWaitPlatform for ExternalWaitPlatform {
    type Error = core::convert::Infallible;

    fn poll_ready(
        &mut self,
        _: &mut core::task::Context<'_>,
        sources: hisi_rf::WaitSet,
        deadline_us: Option<u64>,
    ) -> core::task::Poll<Result<hisi_rf::WaitSet, Self::Error>> {
        let _registered_contract = (sources, deadline_us);
        core::task::Poll::Pending
    }
}

#[cfg(feature = "incremental-contract")]
#[allow(dead_code)]
fn check_incremental_facade<B, D>(radio: hisi_rf::RadioController<B, D>)
where
    B: hisi_rf::IncrementalWifiBackend,
{
    let budget = hisi_rf::WorkBudget::try_new(4, 100).expect("non-zero work budget");
    let parts = radio.split_incremental(budget);
    let intent: hisi_rf::IncrementalWaitIntent = parts.runner.wait_intent();
    let _platform_wait_contract = (
        intent.sources(),
        intent.deadline_us(),
        intent.run_immediately(),
    );
    let mut platform = ExternalWaitPlatform;
    let _wait = parts.runner.wait_ready(&mut platform);
    let diagnostics: hisi_rf::IncrementalRunnerDiagnostics = parts.runner.diagnostics();
    let _observability_contract = (
        diagnostics.run_once_calls,
        diagnostics.wait_ready_calls,
        diagnostics.operations_completed,
    );
}

#[cfg(feature = "incremental-contract")]
#[allow(dead_code)]
fn check_ws63_incremental_facade(radio: hisi_rf::ws63::IncrementalRadioController) {
    let budget = hisi_rf::WorkBudget::try_new(4, 100).expect("non-zero work budget");
    let mut parts = radio.split(budget);
    {
        let _wait = parts.runner.wait_ready();
    }
    let runner_diagnostics: hisi_rf::IncrementalRunnerDiagnostics = parts.runner.diagnostics();
    let wait_diagnostics: hisi_rf::ws63::Ws63IncrementalWaitDiagnostics =
        parts.runner.wait_diagnostics();
    let snapshot: hisi_rf::ws63::RadioDiagnosticsSnapshot = parts.diagnostics();
    let snapshot_runner = match snapshot.runner {
        hisi_rf::ws63::RunnerDiagnosticsSnapshot::Incremental(runner) => runner,
        hisi_rf::ws63::RunnerDiagnosticsSnapshot::Blocking(_) => unreachable!(),
    };
    let _observability_contract = (
        runner_diagnostics.wait_ready_completions,
        wait_diagnostics.backend_signals,
        wait_diagnostics.ready_polls,
        snapshot_runner.operations_completed,
        snapshot.wait.backend_signals,
        snapshot.events.dropped,
        snapshot.blocking_calls.connect.calls,
        snapshot.resources.caller_owned_bytes,
    );
}

#[entry]
fn main() -> ! {
    let mut diagnostic_sink = DiagnosticSink;
    let _diagnostic_contract = (
        hisi_rf::DIAGNOSTIC_SCHEMA,
        hisi_rf::DIAGNOSTIC_TRACE_CAPACITY,
    );
    hisi_rf::Error::AlreadyInitialized
        .diagnostic()
        .write_json(&mut diagnostic_sink)
        .expect("diagnostic sink is infallible");

    let peripherals = unsafe { hisi_hal::peripherals::Peripherals::steal() };
    let (control, arena) = RADIO_STORAGE
        .install()
        .expect("install caller-owned radio storage")
        .into_init_parts();
    let resources = hisi_rf::ws63::Resources::<hisi_rf::ws63::SelectedProfile>::builder(
        peripherals.EFUSE,
        arena,
    )
    .crypto(peripherals.KM, peripherals.SPACC, peripherals.TRNG);
    #[cfg(feature = "wpa2-personal")]
    let resources = resources.build();
    #[cfg(feature = "wpa3-personal")]
    let resources = resources.pke(peripherals.PKE).build();
    let _radio = hisi_rf::ws63::init(hisi_rf::RadioConfig::default(), resources, control)
        .expect("fresh static radio storage");

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
