extern crate libc;
extern crate libelf_sys as ffi;


#[macro_use]
mod error;
pub use error::{Error, Result};

mod elf;
pub use elf::Elf;
