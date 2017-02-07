use ffi;
use libelf;
use std::ptr;

use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use super::Result;
use super::{CompileUnits, TypeUnits};


#[derive(Debug)]
pub struct Dwarf<'a> {
    inner: *mut ffi::Dwarf,
    phantom: PhantomData<&'a mut ffi::Dwarf>,
}

impl<'a> Dwarf<'a> {
    fn new(dwarf: *mut ffi::Dwarf) -> Dwarf<'a> {
        Dwarf {
            inner: dwarf,
            phantom: PhantomData,
        }
    }

    pub fn from_fd<FD: AsRawFd>(fd: &FD) -> Result<Dwarf> {
        let fd = fd.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(fd, ffi::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf))
    }

    pub fn from_elf<'b>(elf: &'b libelf::Elf) -> Result<Dwarf<'b>> {
        let dwarf = ffi!(dwarf_begin_elf(elf.as_ptr(), ffi::DWARF_C_READ, ptr::null_mut()))?;
        Ok(Dwarf::new(dwarf))
    }

    pub fn compile_units(&self) -> CompileUnits {
        ::units::compile_units(self)
    }

    pub fn type_units(&self) -> TypeUnits {
        ::units::type_units(self)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf {
        self.inner
    }
}

impl<'a> Drop for Dwarf<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::dwarf_end(self.inner);
        }
    }
}
