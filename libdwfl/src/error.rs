use ffi;
use libc;
use std::error;
use std::fmt;
use std::io;
use std::result;
use std::ffi::CStr;


macro_rules! raw_ffi {
    ($func:ident ($($arg:expr),*)) => ({
        #[allow(unused_unsafe)]
        unsafe { ffi::$func($($arg),*) }
    })
}

macro_rules! ffi {
    ($func:ident ($($arg:expr),*)) => ({
        let result = raw_ffi!($func($($arg),*));
        ::error::IntoResult::into_result(result)
    })
}


pub type Result<T> = result::Result<T, Error>;

pub(crate) trait IntoResult: Sized {
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

impl IntoResult for isize {
    #[inline]
    fn into_result(self) -> Result<Self> {
        if self < 0 {
            Err(Error::last())
        } else {
            Ok(self)
        }
    }
}

impl<T> IntoResult for *const T {
    #[inline]
    fn into_result(self) -> Result<Self> {
        if self.is_null() {
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


#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Dwfl(libc::c_int),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error {
            kind: ErrorKind::Io(error),
        }
    }
}

impl Error {
    #[inline]
    pub fn last() -> Error {
        let errno = raw_ffi!(dwfl_errno());
        Error {
            kind: ErrorKind::Dwfl(errno),
        }
    }

    #[inline]
    pub fn check() -> Option<Error> {
        let error = Error::last();
        if let ErrorKind::Dwfl(0) = error.kind {
            None
        } else {
            Some(error)
        }
    }
}

#[inline]
fn errmsg(errno: libc::c_int) -> &'static CStr {
    // Normalize 0 to -1, which behaves the same except it always returns a legal string
    let errno = match errno { 0 => -1, e => e };
    let msg = raw_ffi!(dwfl_errmsg(errno));
    unsafe { CStr::from_ptr(msg) }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Dwfl(errno) => errmsg(errno).to_str().unwrap_or("libdwfl error"),
            ErrorKind::Io(ref error) => error.description(),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Dwfl(errno) => {
                let msg = errmsg(errno);
                match msg.to_str() {
                    Ok(s) => fmt::Display::fmt(s, f),
                    Err(_) => fmt::Debug::fmt(msg, f),
                }
            },
            ErrorKind::Io(ref error) => fmt::Display::fmt(&error, f),
        }
    }
}
