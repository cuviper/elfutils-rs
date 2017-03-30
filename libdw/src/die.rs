use ffi;

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::os::raw;
use std::panic;
use std::ptr;

use super::Result;
use super::Dwarf;


#[derive(Debug)]
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
    pub fn attr_count(&self) -> Result<usize> {
        let mut count = 0;
        let abbrev = self.get_abbrev()?;
        ffi!(dwarf_getattrcnt(abbrev, &mut count))?;
        Ok(count)
    }

    #[inline]
    pub fn attrs(&self) -> Result<Vec<ffi::Dwarf_Attribute>> {
        let mut v = Vec::with_capacity(self.attr_count()?);
        unsafe {
            self.for_each_attr_unchecked(|a| { v.push(*a); Ok(true) })?;
        }
        Ok(v)
    }

    pub fn for_each_attr<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&mut ffi::Dwarf_Attribute) -> Result<bool>
    {
        let mut panic = None;
        let mut result = Ok(());

        self.getattrs(|attr| {
            if panic.is_some() {
                // We already panicked!
                return ffi::DWARF_CB_ABORT;
            }

            // Asserted safe because we'll rethrow after the ffi returns,
            // so no one can see any possibly inconsistent state.
            let call = panic::AssertUnwindSafe(|| {
                match f(attr) {
                    Ok(true) => ffi::DWARF_CB_OK,
                    Ok(false) => ffi::DWARF_CB_ABORT,
                    Err(e) => {
                        result = Err(e);
                        ffi::DWARF_CB_ABORT
                    },
                }
            });

            match panic::catch_unwind(call) {
                Ok(rc) => rc,
                Err(e) => {
                    panic = Some(e);
                    ffi::DWARF_CB_ABORT
                }
            }
        })?;

        if let Some(payload) = panic {
            panic::resume_unwind(payload);
        }

        result
    }

    pub unsafe fn for_each_attr_unchecked<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&mut ffi::Dwarf_Attribute) -> Result<bool>
    {
        let mut result = Ok(());

        self.getattrs(|attr| {
            match f(attr) {
                Ok(true) => ffi::DWARF_CB_OK,
                Ok(false) => ffi::DWARF_CB_ABORT,
                Err(e) => {
                    result = Err(e);
                    ffi::DWARF_CB_ABORT
                },
            }
        })?;

        result
    }

    fn getattrs<F>(&self, mut f: F) -> Result<isize>
        where F: FnMut(&mut ffi::Dwarf_Attribute) -> raw::c_uint
    {
        let argp = &mut f as *mut F as *mut raw::c_void;
        return ffi!(dwarf_getattrs(self.as_ptr(), Some(callback::<F>), argp, 0));

        unsafe extern "C" fn callback<F>(attr: *mut ffi::Dwarf_Attribute,
                                         argp: *mut raw::c_void)
                                         -> raw::c_int
            where F: FnMut(&mut ffi::Dwarf_Attribute) -> raw::c_uint
        {
            let f = &mut *(argp as *mut F);
            f(&mut *attr) as raw::c_int
        }
    }

    pub fn for_each_func<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Self) -> Result<bool>
    {
        let mut panic = None;
        let mut result = Ok(());

        self.getfuncs(|func| {
            if panic.is_some() {
                // We already panicked!
                return ffi::DWARF_CB_ABORT;
            }

            // Asserted safe because we'll rethrow after the ffi returns,
            // so no one can see any possibly inconsistent state.
            let call = panic::AssertUnwindSafe(|| {
                match f(func) {
                    Ok(true) => ffi::DWARF_CB_OK,
                    Ok(false) => ffi::DWARF_CB_ABORT,
                    Err(e) => {
                        result = Err(e);
                        ffi::DWARF_CB_ABORT
                    },
                }
            });

            match panic::catch_unwind(call) {
                Ok(rc) => rc,
                Err(e) => {
                    panic = Some(e);
                    ffi::DWARF_CB_ABORT
                }
            }
        })?;

        if let Some(payload) = panic {
            panic::resume_unwind(payload);
        }

        result
    }

    pub unsafe fn for_each_func_unchecked<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Self) -> Result<bool>
    {
        let mut result = Ok(());

        self.getfuncs(|func| {
            match f(func) {
                Ok(true) => ffi::DWARF_CB_OK,
                Ok(false) => ffi::DWARF_CB_ABORT,
                Err(e) => {
                    result = Err(e);
                    ffi::DWARF_CB_ABORT
                },
            }
        })?;

        result
    }

    fn getfuncs<F>(&self, mut f: F) -> Result<isize>
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
            f(&Die::from_raw(func)) as raw::c_int
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
