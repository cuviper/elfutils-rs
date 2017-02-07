use ffi;
use libc;
use std::error;
use std::fmt;
use std::result;
use std::ffi::CStr;


pub type Result<T> = result::Result<T, Error>;

pub trait IntoResult: Sized {
    fn into_result(self) -> Result<Self>;
}

impl IntoResult for libc::c_int {
    #[inline]
    fn into_result(self) -> Result<Self> {
        if self < 0 {
            Err(Error::last())
        } else {
            Ok(self)
        }
    }
}

impl IntoResult for libc::ptrdiff_t {
    #[inline]
    fn into_result(self) -> Result<Self> {
        if self < 0 {
            Err(Error::last())
        } else {
            Ok(self)
        }
    }
}

impl<T> IntoResult for *mut T {
    #[inline]
    fn into_result(self) -> Result<Self> {
        if self.is_null() {
            Err(Error::last())
        } else {
            Ok(self)
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Error {
    errno: libc::c_int,
}

impl Error {
    fn last() -> Error {
        Error {
            errno: unsafe { ffi::dwarf_errno() },
        }
    }

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


macro_rules! ffi {
    ($func:ident ($($arg:expr),*)) => ({
        let result = unsafe { ffi::$func($($arg),*) };
        ::error::IntoResult::into_result(result)
    })
}
