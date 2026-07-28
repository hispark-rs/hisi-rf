#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

#[cfg(target_arch = "riscv32")]
use hisi_riscv_rt::entry;

#[cfg(target_arch = "riscv32")]
hisi_rf::ws63::declare_radio_storage!(static RADIO_STORAGE);

#[cfg(target_arch = "riscv32")]
#[entry]
fn main() -> ! {
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
