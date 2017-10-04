use ffi;

use libc;
use std::fmt;
use std::fs;
use std::path::Path;
use std::ptr;

use std::os::unix::io::AsRawFd;

use super::Result;


/// A handle to an ELF file.
#[derive(Debug)]
pub struct Elf<'elf> {
    inner: *mut ffi::Elf,
    kind: ElfKind<'elf>,
}

enum ElfKind<'elf> {
    Raw,
    File(fs::File),
    Fd(&'elf AsRawFd),
    Bytes(&'elf [u8]),
}

impl<'elf> fmt::Debug for ElfKind<'elf> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ElfKind::Raw => fmt.debug_tuple("Raw").finish(),
            ElfKind::File(ref f) => fmt.debug_tuple("File").field(f).finish(),
            ElfKind::Fd(f) => fmt.debug_tuple("Fd").field(&f.as_raw_fd()).finish(),
            ElfKind::Bytes(b) => fmt.debug_tuple("Bytes").field(&b.as_ptr()).field(&b.len()).finish(),
        }
    }
}

impl<'elf> Elf<'elf> {
    #[inline]
    fn new(elf: *mut ffi::Elf, kind: ElfKind<'elf>) -> Elf<'elf> {
        Elf {
            inner: elf,
            kind: kind,
        }
    }

    /// Open an `Elf` from a path.
    ///
    /// # Examples
    ///
    /// ```
    /// let exe = std::env::current_exe().unwrap();
    /// let elf = libelf::Elf::open(exe).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Elf<'static>> {
        let file = fs::File::open(path)?;
        let fd = file.as_raw_fd();
        raw_ffi!(elf_version(ffi::EV_CURRENT));
        let elf = ffi!(elf_begin(fd, ffi::Elf_Cmd::ELF_C_READ_MMAP, ptr::null_mut()))?;
        Ok(Elf::new(elf, ElfKind::File(file)))
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
    pub fn from_fd<FD: AsRawFd>(fd: &'elf FD) -> Result<Elf<'elf>> {
        let raw_fd = fd.as_raw_fd();
        raw_ffi!(elf_version(ffi::EV_CURRENT));
        let elf = ffi!(elf_begin(raw_fd, ffi::Elf_Cmd::ELF_C_READ_MMAP, ptr::null_mut()))?;
        Ok(Elf::new(elf, ElfKind::Fd(fd)))
    }

    /// Create an `Elf` from a byte slice.
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
    /// let elf = libelf::Elf::from_bytes(&buf).unwrap();
    /// ```
    ///
    /// ```
    /// // elfutils doesn't mind an empty ELF!
    /// let empty = libelf::Elf::from_bytes(&[]).unwrap();
    /// ```
    #[inline]
    pub fn from_bytes(bytes: &'elf [u8]) -> Result<Elf<'elf>> {
        // NB: `Elf` must not expose write interfaces!
        let ptr = bytes.as_ptr() as *mut libc::c_char;
        let elf = ffi!(elf_memory(ptr, bytes.len()))?;
        Ok(Elf::new(elf, ElfKind::Bytes(bytes)))
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
    pub unsafe fn from_raw(elf: *mut ffi::Elf) -> Elf<'elf> {
        Elf::new(elf, ElfKind::Raw)
    }

    /// Get a raw FFI pointer
    ///
    /// # Examples
    ///
    /// ```
    /// # let exe = std::env::current_exe().unwrap();
    /// # let elf = libelf::Elf::open(exe).unwrap();
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

impl<'elf> Drop for Elf<'elf> {
    #[inline]
    fn drop(&mut self) {
        match self.kind {
            ElfKind::Raw => (),
            _ => { raw_ffi!(elf_end(self.as_ptr())); },
        }
    }
}
