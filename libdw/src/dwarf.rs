use ffi;
use libelf;
use std::ptr;

use std::fmt;
use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use super::Result;
use super::{CompileUnits, TypeUnits};


#[derive(Debug)]
pub struct Dwarf<'dw> {
    inner: *mut ffi::Dwarf,
    kind: DwarfKind<'dw>,
}

enum DwarfKind<'dw> {
    Raw,
    File(fs::File),
    Fd(&'dw AsRawFd),
    Elf(&'dw libelf::Elf<'dw>),
}

impl<'elf> fmt::Debug for DwarfKind<'elf> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DwarfKind::Raw => fmt.debug_tuple("Raw").finish(),
            DwarfKind::File(ref f) => fmt.debug_tuple("File").field(f).finish(),
            DwarfKind::Fd(f) => fmt.debug_tuple("Fd").field(&f.as_raw_fd()).finish(),
            DwarfKind::Elf(e) => fmt.debug_tuple("Elf").field(&e).finish(),
        }
    }
}

impl<'dw> Dwarf<'dw> {
    #[inline]
    fn new(dwarf: *mut ffi::Dwarf, kind: DwarfKind<'dw>) -> Dwarf<'dw> {
        Dwarf {
            inner: dwarf,
            kind: kind,
        }
    }

    /// Open a `Dwarf` from a path.
    ///
    /// # Examples
    ///
    /// ```
    /// let exe = std::env::current_exe().unwrap();
    /// let dw = libdw::Dwarf::open(exe).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Dwarf<'static>> {
        let file = fs::File::open(path)?;
        let raw_fd = file.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(raw_fd, ffi::Dwarf_Cmd::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf, DwarfKind::File(file)))
    }

    /// Create a `Dwarf` from an open file.
    ///
    /// # Examples
    ///
    /// ```
    /// let exe = std::env::current_exe().unwrap();
    /// let f = std::fs::File::open(exe).unwrap();
    /// let dw = libdw::Dwarf::from_fd(&f).unwrap();
    /// ```
    #[inline]
    pub fn from_fd<FD: AsRawFd>(fd: &'dw FD) -> Result<Dwarf<'dw>> {
        let raw_fd = fd.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(raw_fd, ffi::Dwarf_Cmd::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf, DwarfKind::Fd(fd)))
    }

    /// Create a `Dwarf` from an existing `Elf`.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate libdw;
    /// # extern crate libelf;
    /// let exe = std::env::current_exe().unwrap();
    /// let elf = libelf::Elf::open(exe).unwrap();
    /// let dw = libdw::Dwarf::from_elf(&elf).unwrap();
    /// ```
    #[inline]
    pub fn from_elf(elf: &'dw libelf::Elf) -> Result<Dwarf<'dw>> {
        let ptr = elf.as_ptr();
        let dwarf = ffi!(dwarf_begin_elf(ptr, ffi::Dwarf_Cmd::DWARF_C_READ, ptr::null_mut()))?;
        Ok(Dwarf::new(dwarf, DwarfKind::Elf(elf)))
    }

    /// Create a `Dwarf` from a raw FFI pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because there is no guarantee that the given
    /// pointer is a valid `libdw` handle, nor whether the lifetime inferred
    /// is appropriate.  This does not take ownership of the underlying object,
    /// so the caller must ensure it outlives the returned `Dwarf` wrapper.
    #[inline]
    pub unsafe fn from_raw(dwarf: *mut ffi::Dwarf) -> Dwarf<'dw> {
        Dwarf::new(dwarf, DwarfKind::Raw)
    }

    #[inline]
    pub fn get_elf(&'dw self) -> libelf::Elf<'dw> {
        let elf = raw_ffi!(dwarf_getelf(self.as_ptr()));
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
        match self.kind {
            DwarfKind::Raw => (),
            _ => { raw_ffi!(dwarf_end(self.as_ptr())); },
        }
    }
}
