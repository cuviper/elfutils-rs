use ffi;
use std::error;
use std::fmt;
use std::os::raw;
use std::result;
use std::ffi::CStr;


pub type Result<T> = result::Result<T, Error>;

pub trait IntoResult: Sized {
    fn into_result(self) -> Result<Self>;
}

impl IntoResult for raw::c_int {
    fn into_result(self) -> Result<Self> {
        if self < 0 {
            Err(Error::last())
        } else {
            Ok(self)
        }
    }
}

impl<T> IntoResult for *mut T {
    fn into_result(self) -> Result<Self> {
        if self.is_null() {
            Err(Error::last())
        } else {
            Ok(self)
        }
    }
}


#[derive(Debug)]
pub struct Error {
    errno: raw::c_int,
}

impl Error {
    fn last() -> Error {
        Error {
            errno: unsafe { ffi::elf_errno() },
        }
    }

    fn to_cstr(&self) -> &'static CStr {
        // Normalize 0 to -1, which behaves the same except it always returns a legal string
        let errno = match self.errno { 0 => -1, e => e };
        unsafe { CStr::from_ptr(ffi::elf_errmsg(errno)) }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.to_cstr().to_str()
            .unwrap_or("invalid UTF-8 from elf_errmsg")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.to_cstr().to_string_lossy(), f)
    }
}


macro_rules! ffi {
    ($func:ident ($($arg:expr),*)) => ({
        let result = unsafe { ffi::$func($($arg),*) };
        ::error::IntoResult::into_result(result)
    })
}
