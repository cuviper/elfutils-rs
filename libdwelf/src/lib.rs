extern crate libdw_sys as ffi;
extern crate libdw;
extern crate libelf;

use std::ffi::CStr;
use std::ptr;
use std::slice;

#[inline]
pub fn gnu_debuglink<'a>(elf: &libelf::Elf<'a>) -> libelf::Result<Option<(&'a CStr, u32)>> {
    let mut crc = 0;
    let namep = unsafe {
        let elf = elf.as_ptr() as *mut _; // FIXME distinct bindgen Elf types
        ffi::dwelf_elf_gnu_debuglink(elf, &mut crc)
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
pub fn gnu_debugaltlink<'a>(
    dwarf: &libdw::Dwarf<'a>,
) -> libdw::Result<Option<(&'a CStr, &'a [u8])>> {
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
    #[test]
    fn it_works() {}
}
