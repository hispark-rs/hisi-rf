#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

#[cfg(target_arch = "riscv32")]
use hisi_riscv_rt::entry;

#[cfg(target_arch = "riscv32")]
static RADIO_STATE: hisi_rf::RadioState<4> = hisi_rf::RadioState::new();

#[cfg(target_arch = "riscv32")]
#[entry]
fn main() -> ! {
    let peripherals = unsafe { hisi_hal::peripherals::Peripherals::steal() };
    let resources = hisi_rf::ws63::Resources::new(
        peripherals.EFUSE,
        peripherals.KM,
        peripherals.SPACC,
        peripherals.PKE,
        peripherals.TRNG,
    );
    let _radio = hisi_rf::ws63::init(hisi_rf::RadioConfig::default(), resources, &RADIO_STATE)
        .expect("fresh static radio state");

    loop {
        core::hint::spin_loop();
    }
}

#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(not(target_arch = "riscv32"))]
fn main() {}
