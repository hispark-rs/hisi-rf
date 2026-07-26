use std::{env, path::Path};

fn main() {
    const ROM_FALLBACKS: &str = "DEP_HISI_RF_WS63_ROM_FALLBACKS";

    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-env-changed={ROM_FALLBACKS}");

    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("riscv32") {
        return;
    }

    let script = env::var(ROM_FALLBACKS)
        .unwrap_or_else(|_| panic!("hisi-rf-ws63 did not export its ROM fallback link contract"));
    assert!(
        Path::new(&script).is_file(),
        "hisi-rf-ws63 ROM fallback linker script does not exist: {script}"
    );
    println!("cargo:rustc-link-arg=-T{script}");
}
