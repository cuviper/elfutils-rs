use crate::ffi;

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use libdw::Dwarf;

use super::Result;

const STANDARD_CALLBACKS: &'static ffi::Dwfl_Callbacks = &ffi::Dwfl_Callbacks {
    find_elf: Some(ffi::dwfl_build_id_find_elf),
    find_debuginfo: Some(ffi::dwfl_standard_find_debuginfo),
    section_address: Some(ffi::dwfl_offline_section_address),
    debuginfo_path: 0 as *mut _, //ptr::null_mut(),
};

pub struct Dwfl {
    inner: *mut ffi::Dwfl,
}

impl Dwfl {
    #[inline]
    fn new(dwfl: *mut ffi::Dwfl) -> Self {
        Dwfl { inner: dwfl }
    }

    /// Open a `Dwfl` from a path.
    ///
    /// # Examples
    ///
    /// ```
    /// let exe = std::env::current_exe().unwrap();
    /// let dw = libdwfl::Dwfl::open(exe).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Dwfl> {
        let name = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();

        let dwfl = Dwfl::new(ffi!(dwfl_begin(STANDARD_CALLBACKS))?);
        ffi!(dwfl_report_offline(
            dwfl.as_ptr(),
            name.as_ptr(),
            name.as_ptr(),
            -1
        ))?;
        ffi!(dwfl_report_end(dwfl.as_ptr(), None, ptr::null_mut()))?;
        Ok(dwfl)
    }

    #[inline]
    pub fn dwarfs(&self) -> Dwarfs<'_> {
        Dwarfs {
            dwfl: self,
            offset: 0,
        }
    }

    unsafe fn getdwarf<'dwfl, F>(&'dwfl self, offset: isize, mut f: F) -> Result<isize>
    where
        F: FnMut(Dwarf<'dwfl>) -> libc::c_uint,
    {
        let argp = &mut f as *mut F as *mut libc::c_void;
        return ffi!(dwfl_getdwarf(
            self.as_ptr(),
            Some(callback::<'dwfl, F>),
            argp,
            offset
        ));

        unsafe extern "C" fn callback<'dwfl, F>(
            _module: *mut ffi::Dwfl_Module,
            _userdata: *mut *mut libc::c_void,
            _name: *const libc::c_char,
            _start: u64,
            dwarf: *mut ffi::Dwarf,
            _bias: u64,
            argp: *mut libc::c_void,
        ) -> libc::c_int
        where
            F: FnMut(Dwarf<'dwfl>) -> libc::c_uint,
        {
            let f = &mut *(argp as *mut F);
            f(Dwarf::from_raw(dwarf)) as libc::c_int
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwfl {
        self.inner
    }
}

impl Drop for Dwfl {
    #[inline]
    fn drop(&mut self) {
        raw_ffi!(dwfl_end(self.as_ptr()));
    }
}

pub struct Dwarfs<'dwfl> {
    dwfl: &'dwfl Dwfl,
    offset: isize,
}

impl<'dwfl> Iterator for Dwarfs<'dwfl> {
    type Item = Result<Dwarf<'dwfl>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut dwarf = None;
        let rc = unsafe {
            self.dwfl.getdwarf(self.offset, |dw| {
                dwarf = Some(dw);
                libdw::raw::DWARF_CB_ABORT
            })
        };

        match rc {
            Ok(offset) => {
                self.offset = offset;
                dwarf.map(Ok)
            }
            Err(e) => Some(Err(e)),
        }
    }
}
