extern crate pkg_config;

fn main() {
    let config = pkg_config::Config::new();
    assert!(config.target_supported());
    if config.probe("libdw").is_err() {
        // Guess!  This probably won't work in general, but it helps to have a
        // shim for cases like docs.rs that don't really need to build.
        println!("cargo:rustc-link-lib=dw");
        println!("cargo:rustc-link-lib=elf");
    }
}
