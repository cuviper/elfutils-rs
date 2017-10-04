extern crate libdw_sys as ffi;
extern crate libdw;
extern crate libelf;

use std::ffi::CStr;
use std::ptr;
use std::slice;

#[inline]
pub fn gnu_debuglink<'elf>(elf: &'elf libelf::Elf) -> libelf::Result<Option<(&'elf CStr, u32)>> {
    let mut crc = 0;
    let namep = unsafe {
        ffi::dwelf_elf_gnu_debuglink(elf.as_ptr(), &mut crc)
    };
    if namep.is_null() {
        match libelf::Error::check() {
            Some(error) => Err(error),
            None => Ok(None),
        }
    } else {
        Ok(Some(unsafe { (CStr::from_ptr(namep), crc) }))
    }
}

#[inline]
pub fn gnu_debugaltlink<'dw>(
    dwarf: &'dw libdw::Dwarf,
) -> libdw::Result<Option<(&'dw CStr, &'dw [u8])>> {
    let mut namep = ptr::null();
    let mut build_idp = ptr::null();
    let build_id_len =
        unsafe { ffi::dwelf_dwarf_gnu_debugaltlink(dwarf.as_ptr(), &mut namep, &mut build_idp) };
    if build_id_len < 0 {
        Err(libdw::Error::last())
    } else if build_id_len > 0 {
        Ok(Some(unsafe {
            (
                CStr::from_ptr(namep),
                slice::from_raw_parts(build_idp as *const u8, build_id_len as usize),
            )
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_debuglink() {
        use std::env;
        let exe = env::current_exe().unwrap();
        let elf = libelf::Elf::open(exe).unwrap();
        let link = super::gnu_debuglink(&elf).unwrap();
        assert!(link.is_none());
    }

    #[test]
    fn self_debugaltlink() {
        use std::env;
        let exe = env::current_exe().unwrap();
        let dwarf = libdw::Dwarf::open(exe).unwrap();
        let link = super::gnu_debugaltlink(&dwarf).unwrap();
        assert!(link.is_none());
    }
}
