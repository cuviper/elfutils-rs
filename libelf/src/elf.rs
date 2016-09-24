use ffi;
use libc;
use std::ptr;

use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use super::Result;


#[derive(Debug)]
pub struct Elf<'a> {
    inner: *mut ffi::Elf,
    phantom: PhantomData<&'a mut ffi::Elf>,
}

impl<'a> Elf<'a> {
    fn new(elf: *mut ffi::Elf) -> Elf<'a> {
        Elf {
            inner: elf,
            phantom: PhantomData,
        }
    }

    pub fn from_fd<FD: AsRawFd>(fd: &FD) -> Result<Elf> {
        let fd = fd.as_raw_fd();
        let elf = ptry!(unsafe {
            ffi::elf_version(ffi::EV_CURRENT);
            ffi::elf_begin(fd, ffi::ELF_C_READ, ptr::null_mut())
        });
        Ok(Elf::new(elf))
    }

    pub fn from_mem(mem: &mut [u8]) -> Result<Elf> {
        let ptr = mem.as_mut_ptr() as *mut libc::c_char;
        let elf = ptry!(unsafe { ffi::elf_memory(ptr, mem.len()) });
        Ok(Elf::new(elf))
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Elf {
        self.inner
    }
}

impl<'a> Drop for Elf<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::elf_end(self.inner);
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
        Elf::from_mem(&mut []).unwrap();
    }
}
