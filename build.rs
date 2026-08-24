fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("riscv32") {
        println!("cargo:rustc-link-arg=-Thisi-riscv-link.x");

        // Linker wrapping must be requested by the final binary package. A
        // dependency build script cannot propagate `rustc-link-arg` to facade
        // examples that own the final link.
        if std::env::var_os("CARGO_FEATURE_BLE").is_some() {
            println!("cargo:rustc-link-arg=--wrap=smp_ecdh_public_key_reserv");
            println!("cargo:rustc-link-arg=--wrap=smp_ecdh_dh_key_reserv");
        }
    }
}
