use ffi;
use libelf;
use std::ptr;

use std::fmt;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use super::Result;
use super::{CompileUnits, TypeUnits};


pub struct Dwarf<'dw> {
    inner: *mut ffi::Dwarf,
    owned: bool,
    phantom: PhantomData<&'dw mut ffi::Dwarf>,
}

impl<'dw> fmt::Debug for Dwarf<'dw> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Dwarf")
            .field("inner", &self.inner)
            .field("owned", &self.owned)
            .finish()
    }
}

impl<'dw> Dwarf<'dw> {
    #[inline]
    fn new(dwarf: *mut ffi::Dwarf) -> Self {
        Dwarf {
            inner: dwarf,
            owned: true,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from_fd<FD: AsRawFd>(fd: &'dw FD) -> Result<Dwarf<'dw>> {
        let fd = fd.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(fd, ffi::Dwarf_Cmd::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf))
    }

    #[inline]
    pub fn from_elf(elf: &'dw libelf::Elf) -> Result<Dwarf<'dw>> {
        let elf = elf.as_ptr() as *mut _; // FIXME distinct bindgen Elf types
        let dwarf = ffi!(dwarf_begin_elf(elf, ffi::Dwarf_Cmd::DWARF_C_READ, ptr::null_mut()))?;
        Ok(Dwarf::new(dwarf))
    }

    #[inline]
    pub unsafe fn from_raw(dwarf: *mut ffi::Dwarf) -> Dwarf<'dw> {
        Dwarf {
            inner: dwarf,
            owned: false,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get_elf(&'dw self) -> libelf::Elf<'dw> {
        let elf = raw_ffi!(dwarf_getelf(self.as_ptr()));
        let elf = elf as *mut _; // FIXME distinct bindgen Elf types
        unsafe { libelf::Elf::from_raw(elf) }
    }

    #[inline]
    pub fn compile_units(&'dw self) -> CompileUnits<'dw> {
        CompileUnits::new(self)
    }

    #[inline]
    pub fn type_units(&'dw self) -> TypeUnits<'dw> {
        TypeUnits::new(self)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf {
        self.inner
    }
}

impl<'dw> Drop for Dwarf<'dw> {
    #[inline]
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                ffi::dwarf_end(self.inner);
            }
        }
    }
}
