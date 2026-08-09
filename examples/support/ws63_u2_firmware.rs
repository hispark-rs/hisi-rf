use core::future::Future;
use core::num::{NonZeroU32, NonZeroUsize};
use core::task::{Context, Poll, Waker};

use hisi_hal::Peripherals;
use hisi_hal::delay::Delay;
use hisi_hal::interrupt;
use hisi_hal::rf_power::RfPower;
use hisi_hal::software_interrupt::SoftwareInterrupt0;
use hisi_hal::time::Instant;
use hisi_hal::timer::TimerAlarm0;
use hisi_hal::uart::{Config as UartConfig, Uart, UartClock};
use hisi_hal::wdt::Watchdog;
use hisi_riscv_rt as _;

hisi_rf::ws63::declare_radio_storage!(static RADIO_STORAGE);

pub fn run(role: fn(hisi_rf::ws63::RadioParts) -> !) -> ! {
    let p = Peripherals::take().expect("peripherals already taken");
    let uart = Uart::new_uart0(
        p.UART0,
        UartConfig {
            clock: UartClock::Boot,
            ..UartConfig::default()
        },
    );
    Watchdog::new(p.WDT).disable();
    uart.write(b"\r\nRFDBG_RADIO_U2_BEGIN\r\n");
    hisi_rf::ws63::set_log_sink(log);

    let storage = RADIO_STORAGE.install().expect("install U2 storage");
    log(b"RFDBG_RADIO_U2_STORAGE_OK\r\n");
    let mut delay = Delay::new();
    let rf_ready = RfPower::new(p.CMU, p.CLDO_CRG).enable(p.EFUSE, &mut delay);
    let (_cldo_crg, efuse) = rf_ready.into_parts();
    log(b"RFDBG_RADIO_U2_RF_POWER_OK\r\n");

    let _timer = TimerAlarm0::new(p.TIMER);
    let _software_interrupt = SoftwareInterrupt0::new(p.SYS_CTL1);
    let _runtime = hisi_rtos::start_with_port(
        hisi_rtos::PortedConfig {
            minimum_stack_size: NonZeroUsize::new(hisi_rf::ws63::MINIMUM_TASK_STACK_BYTES).unwrap(),
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
            contract_violation: rtos_contract_violation,
        },
    )
    .expect("start U2 RTOS");
    log(b"RFDBG_RADIO_U2_RTOS_OK\r\n");

    // SAFETY: the RTOS port and both interrupt handlers are installed above.
    unsafe { interrupt::enable_global() };
    hisi_rtos::request_reschedule();

    let resources = hisi_rf::ws63::Resources::new(efuse, p.KM, p.SPACC, p.TRNG);
    log(b"RFDBG_RADIO_U2_INIT_BEGIN\r\n");
    match hisi_rf::ws63::init(resources, storage) {
        Ok(controller) => {
            log(b"RFDBG_RADIO_U2_INIT_OK\r\n");
            role(controller.split())
        }
        Err(_) => fail(b"RFDBG_RADIO_U2_INIT_ERR\r\n"),
    }
}

pub fn drive_scheduler() {
    let _ = hisi_rf_rtos_driver::sleep_ms(NonZeroU32::new(10).unwrap());
}

pub fn wait_with_runner<F, T>(future: F, mut progress: impl FnMut()) -> T
where
    F: Future<Output = T>,
{
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            return result;
        }
        progress();
        drive_scheduler();
    }
}

pub fn log(bytes: &[u8]) {
    const DATA: *mut u32 = 0x4401_0004 as *mut u32;
    const FIFO_STATUS: *const u32 = 0x4401_0044 as *const u32;
    for &byte in bytes {
        // SAFETY: these are the fixed WS63 UART0 status and data registers.
        unsafe {
            while core::ptr::read_volatile(FIFO_STATUS) & 0x01 != 0 {
                core::hint::spin_loop();
            }
            core::ptr::write_volatile(DATA, u32::from(byte));
        }
    }
}

pub fn fail(marker: &[u8]) -> ! {
    log(marker);
    loop {
        core::hint::spin_loop();
    }
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

unsafe fn rtos_allocate(size: usize) -> *mut u8 {
    // SAFETY: the RTOS is the sole allocator client after storage installation.
    unsafe { hisi_rf::ws63::InstalledRadioStorage::allocate(size) }
}

unsafe fn rtos_deallocate(pointer: *mut u8) {
    // SAFETY: `pointer` is returned by `rtos_allocate` to the same RTOS instance.
    unsafe { hisi_rf::ws63::InstalledRadioStorage::deallocate(pointer) };
}

fn monotonic_ms() -> u64 {
    Instant::now().raw() / 24_000
}

fn rtos_contract_violation(_violation: hisi_rtos::ContractViolation) -> ! {
    panic!("hisi-rtos scheduler contract violation")
}
