use ffi;
use libc;
use std::error;
use std::fmt;
use std::result;
use std::ffi::CStr;


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug, Clone, Copy)]
pub struct Error {
    errno: libc::c_int,
}

impl Error {
    fn to_cstr(&self) -> &'static CStr {
        // Normalize 0 to -1, which behaves the same except it always returns a legal string
        let errno = match self.errno { 0 => -1, e => e };
        unsafe { CStr::from_ptr(ffi::dwarf_errmsg(errno)) }
    }
}

impl<'a> From<&'a Error> for Error {
    fn from(other: &'a Error) -> Error {
        *other
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.to_cstr().to_str()
            .unwrap_or("invalid UTF-8 from dwarf_errmsg")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.to_cstr().to_string_lossy(), f)
    }
}


pub fn last() -> Error {
    Error {
        errno: unsafe { ffi::dwarf_errno() },
    }
}

macro_rules! itry {
    ($expr:expr) => ({
        let i = $expr;
        if i < 0 { return Err(::error::last()) }
        i
    })
}

macro_rules! ptry {
    ($expr:expr) => ({
        let p = $expr;
        if p.is_null() { return Err(::error::last()) }
        p
    })
}
