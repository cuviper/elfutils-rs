use ffi;

use ffi::Dwarf_Off;

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::os::raw;
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

#[inline]
pub fn offdie<'a>(dwarf: &'a Dwarf<'a>, offset: Dwarf_Off) -> Result<Die<'a>> {
    let die = Die::default();
    ffi!(dwarf_offdie(dwarf.as_ptr(), offset, die.as_ptr()))?;
    Ok(die)
}

#[inline]
pub fn offdie_types<'a>(dwarf: &'a Dwarf<'a>, offset: Dwarf_Off) -> Result<Die<'a>> {
    let die = Die::default();
    ffi!(dwarf_offdie_types(dwarf.as_ptr(), offset, die.as_ptr()))?;
    Ok(die)
}

impl<'a> Die<'a> {
    #[inline]
    fn get_abbrev(&self) -> Result<*mut ffi::Dwarf_Abbrev> {
        unsafe {
            if (*self.as_ptr()).abbrev.is_null() {
                self.has_children()?;
            }
            Ok((*self.as_ptr()).abbrev)
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
        self.for_each_attr(|a| { v.push(*a); Ok(true) })?;
        Ok(v)
    }

    pub fn for_each_attr<F>(&self, f: F) -> Result<()>
        where F: FnMut(&mut ffi::Dwarf_Attribute) -> Result<bool>
    {
        type Arg<F> = (Result<()>, F);

        unsafe extern "C" fn callback<F>(attr: *mut ffi::Dwarf_Attribute,
                                         argp: *mut raw::c_void)
                                         -> raw::c_int
            where F: FnMut(&mut ffi::Dwarf_Attribute) -> Result<bool>
        {
            let (ref mut res, ref mut f) = *(argp as *mut Arg<F>);
            let res = match f(&mut *attr) {
                Ok(true) => ffi::DWARF_CB_OK,
                Ok(false) => ffi::DWARF_CB_ABORT,
                Err(e) => { *res = Err(e); ffi::DWARF_CB_ABORT },
            };
            res as raw::c_int
        }

        let mut arg = (Ok(()), f);
        let argp = &mut arg as *mut Arg<F> as *mut _;
        ffi!(dwarf_getattrs(self.as_ptr(), Some(callback::<F>), argp, 0))?;
        arg.0
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf_Die {
        self.inner.get()
    }
}

impl<'a> Clone for Die<'a> {
    #[inline]
    fn clone(&self) -> Self {
        Die {
            inner: UnsafeCell::new(unsafe { *self.as_ptr() }),
            phantom: PhantomData,
        }
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
