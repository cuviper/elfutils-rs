use ffi;

use std::fmt;
use std::os::raw;
use std::ptr;

use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use super::Result;


/// A handle to an ELF file.
pub struct Elf<'a> {
    inner: *mut ffi::Elf,
    owned: bool,
    phantom: PhantomData<&'a mut ffi::Elf>,
}

impl<'a> fmt::Debug for Elf<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Elf")
            .field("inner", &self.inner)
            .field("owned", &self.owned)
            .finish()
    }
}

impl<'a> Elf<'a> {
    #[inline]
    fn new(elf: *mut ffi::Elf) -> Self {
        Elf {
            inner: elf,
            owned: true,
            phantom: PhantomData,
        }
    }

    /// Create an `Elf` from an open file.
    ///
    /// # Examples
    ///
    /// ```
    /// let exe = std::env::current_exe().unwrap();
    /// let f = std::fs::File::open(exe).unwrap();
    /// let elf = libelf::Elf::from_fd(&f).unwrap();
    /// ```
    #[inline]
    pub fn from_fd<FD: AsRawFd>(fd: &'a FD) -> Result<Elf<'a>> {
        let fd = fd.as_raw_fd();
        unsafe { ffi::elf_version(ffi::EV_CURRENT); }
        ffi!(elf_begin(fd, ffi::Elf_Cmd::ELF_C_READ_MMAP, ptr::null_mut()))
            .map(Elf::new)
    }

    /// Create an `Elf` from a memory image.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Read;
    /// let exe = std::env::current_exe().unwrap();
    /// let mut buf = vec![];
    /// std::fs::File::open(exe).unwrap()
    ///     .read_to_end(&mut buf).unwrap();
    ///
    /// let elf = libelf::Elf::from_mem(&buf).unwrap();
    /// ```
    ///
    /// ```
    /// // elfutils doesn't mind an empty ELF!
    /// let empty = libelf::Elf::from_mem(&[]).unwrap();
    ///
    /// ```
    #[inline]
    pub fn from_mem(mem: &'a [u8]) -> Result<Elf<'a>> {
        // NB: `Elf` must not expose write interfaces!
        let ptr = mem.as_ptr() as *mut raw::c_char;
        ffi!(elf_memory(ptr, mem.len()))
            .map(Elf::new)
    }

    /// Create an `Elf` from a raw FFI pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because there is no guarantee that the given
    /// pointer is a valid `libelf` handle, nor whether the lifetime inferred
    /// is appropriate.  This does not take ownership of the underlying object,
    /// so the caller must ensure it outlives the returned `Elf` wrapper.
    #[inline]
    pub unsafe fn from_raw(elf: *mut ffi::Elf) -> Elf<'a> {
        Elf {
            inner: elf,
            owned: false,
            phantom: PhantomData,
        }
    }

    /// Get a raw FFI pointer
    ///
    /// # Examples
    ///
    /// ```
    /// # let exe = std::env::current_exe().unwrap();
    /// # let f = std::fs::File::open(exe).unwrap();
    /// # let elf = libelf::Elf::from_fd(&f).unwrap();
    /// let ptr = elf.as_ptr();
    /// assert!(!ptr.is_null());
    /// let base = unsafe { libelf::raw::elf_getbase(ptr) };
    /// assert!(base >= 0);
    /// ```
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Elf {
        self.inner
    }
}

impl<'a> Drop for Elf<'a> {
    #[inline]
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                ffi::elf_end(self.inner);
            }
        }
    }
}
