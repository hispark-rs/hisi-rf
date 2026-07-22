#![no_std]
#![no_main]

use hisi_riscv_rt::entry;

struct DiagnosticSink;

impl core::fmt::Write for DiagnosticSink {
    fn write_str(&mut self, _: &str) -> core::fmt::Result {
        Ok(())
    }
}

static RADIO_STORAGE: hisi_rf::ws63::Storage<hisi_rf::ws63::SelectedProfile, 4> =
    hisi_rf::ws63::Storage::new();

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
#[allow(dead_code)]
fn check_incremental_facade<B, D, const EVENTS: usize>(
    radio: hisi_rf::RadioController<B, D, EVENTS>,
) where
    B: hisi_rf::IncrementalWifiBackend,
{
    let budget = hisi_rf::WorkBudget::try_new(4, 100).expect("non-zero work budget");
    let _parts = radio.split_incremental(budget);
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
    let resources = hisi_rf::ws63::Resources::new(
        peripherals.EFUSE,
        peripherals.KM,
        peripherals.SPACC,
        peripherals.PKE,
        peripherals.TRNG,
    );
    let _radio = hisi_rf::ws63::init(
        hisi_rf::RadioConfig::default(),
        resources,
        &RADIO_STORAGE,
    )
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
