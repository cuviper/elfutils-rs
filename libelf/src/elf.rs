use ffi;

use std::fmt;
use std::os::raw;
use std::ptr;

use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use super::Result;


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

    #[inline]
    pub fn from_fd<FD: AsRawFd>(fd: &'a FD) -> Result<Elf<'a>> {
        let fd = fd.as_raw_fd();
        unsafe { ffi::elf_version(ffi::EV_CURRENT); }
        ffi!(elf_begin(fd, ffi::Elf_Cmd::ELF_C_READ_MMAP, ptr::null_mut()))
            .map(Elf::new)
    }

    #[inline]
    pub fn from_mem(mem: &'a [u8]) -> Result<Elf<'a>> {
        // NB: `Elf` must not expose write interfaces!
        let ptr = mem.as_ptr() as *mut raw::c_char;
        ffi!(elf_memory(ptr, mem.len()))
            .map(Elf::new)
    }

    #[inline]
    pub unsafe fn from_raw(elf: *mut ffi::Elf) -> Self {
        Elf {
            inner: elf,
            owned: false,
            phantom: PhantomData,
        }
    }

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


#[cfg(test)]
mod tests {
    use super::Elf;

    #[test]
    fn self_file() {
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        Elf::from_fd(&f).unwrap();
    }

    #[test]
    fn empty_mem() {
        // elfutils doesn't mind an empty ELF!
        Elf::from_mem(&[]).unwrap();
    }
}
