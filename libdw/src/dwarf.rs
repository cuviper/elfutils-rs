use ffi;
use libelf;
use std::ptr;

use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use super::Result;
use super::{CompileUnits, TypeUnits};
use super::Die;


#[derive(Debug)]
pub struct Dwarf<'a> {
    inner: *mut ffi::Dwarf,
    phantom: PhantomData<&'a mut ffi::Dwarf>,
}

impl<'a> Dwarf<'a> {
    #[inline]
    fn new(dwarf: *mut ffi::Dwarf) -> Dwarf<'a> {
        Dwarf {
            inner: dwarf,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from_fd<FD: AsRawFd>(fd: &FD) -> Result<Dwarf> {
        let fd = fd.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(fd, ffi::Dwarf_Cmd::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf))
    }

    #[inline]
    pub fn from_elf<'b>(elf: &'b libelf::Elf) -> Result<Dwarf<'b>> {
        let elf = elf.as_ptr() as *mut _; // FIXME distinct bindgen Elf types
        let dwarf = ffi!(dwarf_begin_elf(elf, ffi::Dwarf_Cmd::DWARF_C_READ, ptr::null_mut()))?;
        Ok(Dwarf::new(dwarf))
    }

    #[inline]
    pub fn compile_units(&self) -> CompileUnits {
        ::units::compile_units(self)
    }

    #[inline]
    pub fn type_units(&self) -> TypeUnits {
        ::units::type_units(self)
    }

    #[inline]
    pub fn offdie(&self, offset: ffi::Dwarf_Off) -> Result<Die> {
        let die = Die::default();
        ffi!(dwarf_offdie(self.as_ptr(), offset, die.as_ptr()))?;
        Ok(die)
    }

    #[inline]
    pub fn offdie_types(&self, offset: ffi::Dwarf_Off) -> Result<Die> {
        let die = Die::default();
        ffi!(dwarf_offdie_types(self.as_ptr(), offset, die.as_ptr()))?;
        Ok(die)
    }

    #[inline]
    pub fn addrdie(&self, address: ffi::Dwarf_Addr) -> Result<Die> {
        let die = Die::default();
        ffi!(dwarf_addrdie(self.as_ptr(), address, die.as_ptr()))?;
        Ok(die)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf {
        self.inner
    }
}

impl<'a> Drop for Dwarf<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ffi::dwarf_end(self.inner);
        }
    }
}
