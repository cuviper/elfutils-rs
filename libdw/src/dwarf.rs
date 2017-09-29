use ffi;
use libelf;
use std::ptr;

use std::fmt;
use std::fs;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use super::Result;
use super::{CompileUnits, TypeUnits};


pub struct Dwarf<'dw> {
    inner: *mut ffi::Dwarf,
    owned: bool,
    file: Option<fs::File>,
    phantom: PhantomData<&'dw mut ffi::Dwarf>,
}

impl<'dw> fmt::Debug for Dwarf<'dw> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Dwarf")
            .field("inner", &self.inner)
            .field("owned", &self.owned)
            .field("file", &self.file)
            .finish()
    }
}

impl<'dw> Dwarf<'dw> {
    #[inline]
    fn new(dwarf: *mut ffi::Dwarf, owned: bool, file: Option<fs::File>) -> Self {
        Dwarf {
            inner: dwarf,
            owned: owned,
            file: file,
            phantom: PhantomData,
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
        let fd = file.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(fd, ffi::Dwarf_Cmd::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf, true, Some(file)))
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
        let fd = fd.as_raw_fd();
        let dwarf = ffi!(dwarf_begin(fd, ffi::Dwarf_Cmd::DWARF_C_READ))?;
        Ok(Dwarf::new(dwarf, true, None))
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
        let elf = elf.as_ptr() as *mut _; // FIXME distinct bindgen Elf types
        let dwarf = ffi!(dwarf_begin_elf(elf, ffi::Dwarf_Cmd::DWARF_C_READ, ptr::null_mut()))?;
        Ok(Dwarf::new(dwarf, true, None))
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
        Dwarf::new(dwarf, false, None)
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
