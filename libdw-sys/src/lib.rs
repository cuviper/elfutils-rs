extern crate libc;
extern crate libelf_sys;

#[cfg_attr(target_pointer_width = "32", path = "lib32.rs")]
#[cfg_attr(target_pointer_width = "64", path = "lib64.rs")]
mod bindgen;
pub use bindgen::*;
