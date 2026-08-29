#![no_std]
#![no_main]

use core::num::{NonZeroU32, NonZeroUsize};

use hisi_hal::Peripherals;
use hisi_hal::delay::Delay;
use hisi_hal::interrupt;
use hisi_hal::rf_power::RfPower;
use hisi_hal::software_interrupt::SoftwareInterrupt0;
use hisi_hal::time::Instant;
use hisi_hal::timer::TimerAlarm0;
use hisi_hal::wdt::Watchdog;
use hisi_riscv_rt::entry;

hisi_rf::declare_radio_storage!(static RADIO_STORAGE);

fn monotonic_ms() -> u64 {
    Instant::now().raw() / (u64::from(hisi_hal::soc::chip::TCXO_HZ) / 1_000)
}

fn contract_violation(_: hisi_rtos::ContractViolation) -> ! {
    panic!("hisi-rtos scheduler contract violation")
}

unsafe fn rtos_allocate(size: usize) -> *mut u8 {
    // SAFETY: the process-lifetime radio storage is the RTOS allocator owner.
    unsafe { hisi_rf::ws63::InstalledRadioStorage::allocate(size) }
}

unsafe fn rtos_deallocate(pointer: *mut u8) {
    // SAFETY: `pointer` came from `rtos_allocate` in this runtime instance.
    unsafe { hisi_rf::ws63::InstalledRadioStorage::deallocate(pointer) };
}

#[unsafe(no_mangle)]
extern "C" fn TIMER_INT0() {
    TimerAlarm0::clear_interrupt();
    hisi_rtos::interrupt_enter();
    hisi_rtos::on_timer_interrupt();
    hisi_rtos::interrupt_exit();
}

#[unsafe(no_mangle)]
extern "C" fn SOFT_INT0() {
    SoftwareInterrupt0::clear_interrupt();
    hisi_rtos::interrupt_enter();
    hisi_rtos::on_software_interrupt();
    hisi_rtos::interrupt_exit();
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().expect("peripherals already taken");
    Watchdog::new(p.WDT).disable();

    let report = RADIO_STORAGE.report();
    let storage = RADIO_STORAGE.install().expect("install radio storage");
    let mut delay = Delay::new();
    let rf_ready = RfPower::new(p.CMU, p.CLDO_CRG).enable(p.EFUSE, &mut delay);
    let (_cldo_crg, efuse) = rf_ready.into_parts();

    let _timer = TimerAlarm0::new(p.TIMER);
    let _software_interrupt = SoftwareInterrupt0::new(p.SYS_CTL1);
    let runtime = hisi_rtos::start_with_port(
        hisi_rtos::PortedConfig {
            minimum_stack_size: NonZeroUsize::new(report.minimum_task_stack_bytes).unwrap(),
            radio_task_policy: hisi_rtos::RunPolicy::Cooperative,
            max_scheduler_lock_duration: NonZeroU32::new(5_000).unwrap(),
        },
        hisi_rtos::Resources {
            allocate: rtos_allocate,
            deallocate: rtos_deallocate,
            monotonic_ms,
        },
        hisi_rtos::SchedulerPort {
            max_timer_delay: NonZeroU32::new(TimerAlarm0::MAX_DELAY_MS).unwrap(),
            arm_timer: TimerAlarm0::arm_millis,
            disarm_timer: TimerAlarm0::disarm,
            pend_reschedule: SoftwareInterrupt0::pend_interrupt,
            contract_violation,
        },
    )
    .expect("start hisi-rtos");
    let main_task = runtime.current_task().expect("adopt main task");
    runtime
        .set_task_run_policy(
            main_task,
            hisi_rtos::RunPolicy::Preemptive {
                time_slice: NonZeroU32::new(5).unwrap(),
            },
        )
        .expect("set main task policy");

    // SAFETY: both RTOS interrupt handlers and the scheduler port are installed.
    unsafe { interrupt::enable_global() };
    hisi_rtos::request_reschedule();

    #[cfg(feature = "ble-peripheral")]
    let resources = hisi_rf::ws63::Resources::new(efuse, p.KM, p.SPACC, p.PKE, p.TRNG);
    #[cfg(feature = "sle-announce")]
    let resources = hisi_rf::ws63::Resources::new(efuse, p.KM, p.SPACC, p.TRNG);
    let controller = hisi_rf::ws63::init(resources, storage).expect("initialize radio facade");
    let mut parts = controller.split();

    loop {
        parts
            .runner
            .run_once()
            .expect("progress radio command plane");
        while parts.runner.run_event_once() {}
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
