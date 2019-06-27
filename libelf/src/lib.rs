use libelf_sys as ffi;

pub mod raw {
    pub use crate::ffi::*;
}

#[macro_use]
mod error;
pub use crate::error::{Error, Result};

mod elf;
pub use crate::elf::Elf;
