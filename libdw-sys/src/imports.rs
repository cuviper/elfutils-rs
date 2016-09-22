// imports to `include!` into bindgen's output

extern crate libc;
extern crate libelf_sys;

use libc::{size_t, ptrdiff_t};
use libc::{uint8_t, uint64_t};
use libc::pid_t;
use libc::FILE;

use libelf_sys::*;

mod dwarf;
pub use dwarf::*;
