use ffi;

use std::any::Any;
use std::cell::UnsafeCell;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw;
use std::panic;
use std::ptr;

use super::Result;
use super::Dwarf;
use super::Attribute;


pub struct Die<'a> {
    inner: UnsafeCell<ffi::Dwarf_Die>,
    phantom: PhantomData<&'a Dwarf<'a>>,
}

impl<'a> Default for Die<'a> {
    #[inline]
    fn default() -> Self {
        Die {
            inner: ffi::Dwarf_Die {
                addr: ptr::null_mut(),
                cu: ptr::null_mut(),
                abbrev: ptr::null_mut(),
                padding__: 0,
            }.into(),
            phantom: PhantomData,
        }
    }
}

impl<'a> fmt::Debug for Die<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("Die")
            .field(unsafe { &*self.as_ptr() })
            .finish()
    }
}

impl<'a> Die<'a> {
    #[inline]
    unsafe fn from_raw(die: *mut ffi::Dwarf_Die) -> Self {
        Die {
            inner: UnsafeCell::new(*die),
            phantom: PhantomData,
        }
    }

    #[inline]
    fn get_abbrev(&self) -> Result<*mut ffi::Dwarf_Abbrev> {
        let die = self.as_ptr();
        unsafe {
            if (*die).abbrev.is_null() {
                self.has_children()?;
                debug_assert!(!(*die).abbrev.is_null());
            }
            Ok((*die).abbrev)
        }
    }

    #[inline]
    pub fn unit(&self) -> Result<Self> {
        let die = Die::default();
        ffi!(dwarf_diecu(self.as_ptr(), die.as_ptr(), ptr::null_mut(), ptr::null_mut()))?;
        Ok(die)
    }

    #[inline]
    pub fn offset(&self) -> ffi::Dwarf_Off {
        raw_ffi!(dwarf_dieoffset(self.as_ptr()))
    }

    #[inline]
    pub fn unit_offset(&self) -> ffi::Dwarf_Off {
        raw_ffi!(dwarf_cuoffset(self.as_ptr()))
    }

    #[inline]
    pub fn child(&self) -> Result<Option<Self>> {
        let die = Die::default();
        let rc = ffi!(dwarf_child(self.as_ptr(), die.as_ptr()))?;
        if rc == 0 {
            Ok(Some(die))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn siblingof(&self) -> Result<Option<Self>> {
        let die = Die::default();
        let rc = ffi!(dwarf_siblingof(self.as_ptr(), die.as_ptr()))?;
        if rc == 0 {
            Ok(Some(die))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn has_children(&self) -> Result<bool> {
        let rc = ffi!(dwarf_haschildren(self.as_ptr()))?;
        Ok(rc > 0)
    }

    #[inline]
    pub fn iter_children(&self) -> DieChildren<'a> {
        DieChildren {
            first: true,
            finished: false,
            die: self.clone(),
        }
    }

    pub fn for_each_child<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Self) -> Result<bool>
    {
        let child = Die::default();

        let mut rc = ffi!(dwarf_child(self.as_ptr(), child.as_ptr()))?;

        while rc == 0 && f(&child)? {
            let ptr = child.as_ptr();
            rc = ffi!(dwarf_siblingof(ptr, ptr))?;
        }
        Ok(())
    }

    #[inline]
    pub fn has_attr(&self, at: u32) -> Result<bool> {
        let b = ffi!(dwarf_hasattr(self.as_ptr(), at))?;
        Ok(b != 0)
    }

    #[inline]
    pub fn has_attr_integrate(&self, at: u32) -> Result<bool> {
        let b = ffi!(dwarf_hasattr_integrate(self.as_ptr(), at))?;
        Ok(b != 0)
    }

    #[inline]
    pub fn attr(&self, at: u32) -> Result<Attribute<'a>> {
        let attr = Attribute::default();
        ffi!(dwarf_attr(self.as_ptr(), at, attr.as_ptr()))?;
        Ok(attr)
    }

    #[inline]
    pub fn attr_integrate(&self, at: u32) -> Result<Attribute<'a>> {
        let attr = Attribute::default();
        ffi!(dwarf_attr_integrate(self.as_ptr(), at, attr.as_ptr()))?;
        Ok(attr)
    }

    #[inline]
    pub fn attr_count(&self) -> Result<usize> {
        let mut count = 0;
        let abbrev = self.get_abbrev()?;
        ffi!(dwarf_getattrcnt(abbrev, &mut count))?;
        Ok(count)
    }

    #[inline]
    pub fn attrs(&self) -> Result<Vec<Attribute<'a>>> {
        let mut v = Vec::with_capacity(self.attr_count()?);
        unsafe {
            self.getattrs(|a| { v.push(a.clone()); ffi::DWARF_CB_OK })?;
        }
        Ok(v)
    }

    pub fn for_each_attr<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Attribute<'a>) -> Result<bool>
    {
        let mut guard = CallbackGuard::new();
        let mut result = Ok(());

        unsafe {
            self.getattrs(|attr| {
                guard.call(|| dwarf_cb_map(f(attr), &mut result))
            })?;
        }

        result
    }

    pub unsafe fn for_each_attr_unchecked<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Attribute<'a>) -> Result<bool>
    {
        let mut result = Ok(());

        self.getattrs(|attr| dwarf_cb_map(f(attr), &mut result))?;

        result
    }

    unsafe fn getattrs<F>(&self, mut f: F) -> Result<isize>
        where F: FnMut(&Attribute<'a>) -> raw::c_uint
    {
        let argp = &mut f as *mut F as *mut raw::c_void;
        return ffi!(dwarf_getattrs(self.as_ptr(), Some(callback::<'a, F>), argp, 0));

        unsafe extern "C" fn callback<'a, F>(attr: *mut ffi::Dwarf_Attribute,
                                             argp: *mut raw::c_void)
                                         -> raw::c_int
            where F: FnMut(&Attribute<'a>) -> raw::c_uint
        {
            let f = &mut *(argp as *mut F);
            let attr = &*(attr as *const Attribute<'a>);
            f(attr) as raw::c_int
        }
    }

    pub fn for_each_func<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Self) -> Result<bool>
    {
        let mut guard = CallbackGuard::new();
        let mut result = Ok(());

        unsafe {
            self.getfuncs(|func| {
                guard.call(|| dwarf_cb_map(f(func), &mut result))
            })?;
        }

        result
    }

    pub unsafe fn for_each_func_unchecked<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Self) -> Result<bool>
    {
        let mut result = Ok(());

        self.getfuncs(|func| dwarf_cb_map(f(func), &mut result))?;

        result
    }

    unsafe fn getfuncs<F>(&self, mut f: F) -> Result<isize>
        where F: FnMut(&Self) -> raw::c_uint
    {
        let argp = &mut f as *mut F as *mut raw::c_void;
        return ffi!(dwarf_getfuncs(self.as_ptr(), Some(callback::<'a, F>), argp, 0));

        unsafe extern "C" fn callback<'a, F>(func: *mut ffi::Dwarf_Die,
                                             argp: *mut raw::c_void)
                                             -> raw::c_int
            where F: FnMut(&Die<'a>) -> raw::c_uint
        {
            let f = &mut *(argp as *mut F);
            let func = &*(func as *const Die<'a>);
            f(func) as raw::c_int
        }
    }

    #[inline]
    pub fn tag(&self) -> Result<u32> {
        let invalid = ffi::DW_TAG_invalid as raw::c_int;
        let tag = ffi_check!(dwarf_tag(self.as_ptr()) != invalid)?;
        Ok(tag as u32)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf_Die {
        self.inner.get()
    }
}

impl<'a> Clone for Die<'a> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Die::from_raw(self.as_ptr()) }
    }
}


#[derive(Debug)]
pub struct DieChildren<'a> {
    first: bool,
    finished: bool,
    die: Die<'a>,
}

impl<'a> Iterator for DieChildren<'a> {
    type Item = Result<Die<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None }

        let die = self.die.as_ptr();
        let rc = if self.first {
            self.first = false;
            ffi!(dwarf_child(die, die))
        } else {
            ffi!(dwarf_siblingof(die, die))
        };

        match rc {
            Ok(0) => {
                // prime the die->abbrev before we Clone
                self.die.get_abbrev().ok();
                Some(Ok(self.die.clone()))
            },
            Ok(_) => { self.finished = true; None },
            Err(e) => { self.finished = true; Some(Err(e)) },
        }
    }
}


#[inline]
fn dwarf_cb_map<T>(cont: Result<bool>, result: &mut Result<T>) -> raw::c_uint {
    match cont {
        Ok(true) => ffi::DWARF_CB_OK,
        Ok(false) => ffi::DWARF_CB_ABORT,
        Err(e) => {
            *result = Err(e);
            ffi::DWARF_CB_ABORT
        },
    }
}


struct CallbackGuard {
    payload: Option<Box<Any + Send>>
}

impl CallbackGuard {
    #[inline]
    fn new() -> CallbackGuard {
        CallbackGuard {
            payload: None
        }
    }

    fn call<F>(&mut self, f: F) -> raw::c_uint
        where F: FnMut() -> raw::c_uint
    {
        if self.payload.is_some() {
            // We already panicked!
            return ffi::DWARF_CB_ABORT;
        }

        // Asserted safe because we'll rethrow after the ffi returns,
        // so no one can see any possibly inconsistent state.
        let call = panic::AssertUnwindSafe(f);

        match panic::catch_unwind(call) {
            Ok(rc) => rc,
            Err(e) => {
                self.payload = Some(e);
                ffi::DWARF_CB_ABORT
            }
        }
    }
}

impl Drop for CallbackGuard {
    #[inline]
    fn drop(&mut self) {
        if let Some(payload) = self.payload.take() {
            panic::resume_unwind(payload);
        }
    }
}
