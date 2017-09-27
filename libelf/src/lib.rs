extern crate libelf_sys as ffi;

pub mod raw {
    pub use ffi::*;
}

#[macro_use]
mod error;
pub use error::{Error, Result};

mod elf;
pub use elf::Elf;
