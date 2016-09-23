use ffi;
use libc;
use std::mem;

use ffi::Dwarf_Off;

use std::cell::RefCell;
use std::marker::PhantomData;

use super::Result;
use super::Dwarf;


#[derive(Debug)]
pub struct Die<'a> {
    inner: RefCell<ffi::Dwarf_Die>,
    phantom: PhantomData<&'a Dwarf<'a>>,
}

pub fn offdie<'a>(dwarf: &'a Dwarf<'a>, offset: Dwarf_Off) -> Result<Die<'a>> {
    unsafe {
        let mut die = mem::uninitialized();
        if ffi::dwarf_offdie(dwarf.as_ptr(), offset, &mut die).is_null() {
            Err(::error::last())
        } else {
            Ok(Die::new(die))
        }
    }
}

pub fn offdie_types<'a>(dwarf: &'a Dwarf<'a>, offset: Dwarf_Off) -> Result<Die<'a>> {
    unsafe {
        let mut die = mem::uninitialized();
        if ffi::dwarf_offdie_types(dwarf.as_ptr(), offset, &mut die).is_null() {
            Err(::error::last())
        } else {
            Ok(Die::new(die))
        }
    }
}

impl<'a> Die<'a> {
    fn new(die: ffi::Dwarf_Die) -> Die<'a> {
        Die {
            inner: RefCell::new(die),
            phantom: PhantomData,
        }
    }

    pub fn children(&'a self) -> DieChildren<'a> {
        DieChildren {
            first: true,
            finished: false,
            die: *self.inner.borrow(),
            phantom: PhantomData,
        }
    }

    pub fn with_children<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Die<'a>) -> bool
    {
        unsafe {
            let mut child = Die {
                inner: RefCell::new(mem::uninitialized()),
                phantom: PhantomData,
            };

            let mut rc = {
                let parent = &mut *self.inner.borrow_mut();
                let child = child.inner.get_mut();
                ffi::dwarf_child(parent, child)
            };

            loop {
                if rc < 0 {
                    return Err(::error::last());
                } else if rc > 0 || !f(&child) {
                    return Ok(());
                }

                let child = child.inner.get_mut();
                rc = ffi::dwarf_siblingof(child, child);
            }
        }
    }

    pub fn with_attrs<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&mut ffi::Dwarf_Attribute) -> bool
    {
        unsafe extern "C" fn callback<F>(attr: *mut ffi::Dwarf_Attribute,
                                         fn_ptr: *mut libc::c_void)
                                         -> libc::c_int
            where F: FnMut(&mut ffi::Dwarf_Attribute) -> bool
        {
            let f = fn_ptr as *mut F;
            if (*f)(&mut *attr) {
                ffi::DWARF_CB_OK as libc::c_int
            } else {
                ffi::DWARF_CB_ABORT as libc::c_int
            }
        }

        let rc = unsafe {
            ffi::dwarf_getattrs(&mut *self.inner.borrow_mut(),
                                Some(callback::<F>),
                                &mut f as *mut F as *mut _, 0)
        };

        if rc < 0 {
            Err(::error::last())
        } else {
            Ok(())
        }
    }
}


#[derive(Debug)]
pub struct DieChildren<'a> {
    first: bool,
    finished: bool,
    die: ffi::Dwarf_Die,
    phantom: PhantomData<&'a Dwarf<'a>>,
}

impl<'a> Iterator for DieChildren<'a> {
    type Item = Result<Die<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None }

        let rc = unsafe {
            let die: *mut _ = &mut self.die;
            if self.first {
                self.first = false;
                ffi::dwarf_child(die, die)
            } else {
                ffi::dwarf_siblingof(die, die)
            }
        };

        if rc == 0 {
            Some(Ok(Die::new(self.die)))
        } else if rc < 0 {
            self.finished = true;
            Some(Err(::error::last()))
        } else {
            self.finished = true;
            None
        }
    }
}
