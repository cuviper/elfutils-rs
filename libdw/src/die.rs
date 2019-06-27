use crate::ffi;

use std::any::Any;
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;
use std::panic;
use std::ptr;

use super::Result;
use super::Dwarf;
use super::Attribute;


pub struct Die<'dw> {
    inner: UnsafeCell<ffi::Dwarf_Die>,
    phantom: PhantomData<&'dw Dwarf<'dw>>,
}

impl<'dw> Default for Die<'dw> {
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

impl<'dw> fmt::Debug for Die<'dw> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("Die")
            .field(unsafe { &*self.as_ptr() })
            .finish()
    }
}

impl<'dw> Die<'dw> {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(die: *mut ffi::Dwarf_Die) -> &'a Die<'dw> {
        &*(die as *const Die<'dw>)
    }

    #[inline]
    pub fn from_offset(dwarf: &'dw Dwarf<'_>, offset: u64) -> Result<Die<'dw>> {
        let die = Die::default();
        ffi!(dwarf_offdie(dwarf.as_ptr(), offset, die.as_ptr()))?;
        Ok(die)
    }

    #[inline]
    pub fn from_type_offset(dwarf: &'dw Dwarf<'_>, offset: u64) -> Result<Die<'dw>> {
        let die = Die::default();
        ffi!(dwarf_offdie_types(dwarf.as_ptr(), offset, die.as_ptr()))?;
        Ok(die)
    }

    #[inline]
    pub fn from_address(dwarf: &'dw Dwarf<'_>, address: u64) -> Result<Die<'dw>> {
        let die = Die::default();
        ffi!(dwarf_addrdie(dwarf.as_ptr(), address, die.as_ptr()))?;
        Ok(die)
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
    pub fn tag(&self) -> Result<u32> {
        let invalid = ffi::DW_TAG_invalid as libc::c_int;
        let tag = ffi_check!(dwarf_tag(self.as_ptr()) != invalid)?;
        Ok(tag as u32)
    }

    #[inline]
    pub fn name(&self) -> Result<&'dw CStr> {
        let s = ffi!(dwarf_diename(self.as_ptr()))?;
        Ok(unsafe { CStr::from_ptr(s) })
    }

    #[inline]
    pub fn decl_file(&self) -> Result<&'dw CStr> {
        let s = ffi!(dwarf_decl_file(self.as_ptr()))?;
        Ok(unsafe { CStr::from_ptr(s) })
    }

    #[inline]
    pub fn decl_line(&self) -> Result<u32> {
        let mut line = 0;
        ffi!(dwarf_decl_line(self.as_ptr(), &mut line))?;
        Ok(line as u32)
    }

    #[inline]
    pub fn decl_column(&self) -> Result<u32> {
        let mut column = 0;
        ffi!(dwarf_decl_column(self.as_ptr(), &mut column))?;
        Ok(column as u32)
    }

    #[inline]
    pub fn high_pc(&self) -> Result<u64> {
        let mut pc = 0;
        ffi!(dwarf_highpc(self.as_ptr(), &mut pc))?;
        Ok(pc)
    }

    #[inline]
    pub fn low_pc(&self) -> Result<u64> {
        let mut pc = 0;
        ffi!(dwarf_lowpc(self.as_ptr(), &mut pc))?;
        Ok(pc)
    }

    #[inline]
    pub fn entry_pc(&self) -> Result<u64> {
        let mut pc = 0;
        ffi!(dwarf_entrypc(self.as_ptr(), &mut pc))?;
        Ok(pc)
    }

    #[inline]
    pub fn has_pc(&self, pc: u64) -> Result<bool> {
        let has = ffi!(dwarf_haspc(self.as_ptr(), pc))?;
        Ok(has != 0)
    }

    #[inline]
    pub fn ranges(&self) -> DieRanges<'dw> {
        DieRanges {
            offset: 0,
            base: 0,
            die: self.clone(),
        }
    }

    #[inline]
    pub fn byte_size(&self) -> Result<u32> {
        let size = ffi!(dwarf_bytesize(self.as_ptr()))?;
        Ok(size as u32)
    }

    #[inline]
    pub fn bit_size(&self) -> Result<u32> {
        let size = ffi!(dwarf_bitsize(self.as_ptr()))?;
        Ok(size as u32)
    }

    #[inline]
    pub fn bit_offset(&self) -> Result<u32> {
        let offset = ffi!(dwarf_bitoffset(self.as_ptr()))?;
        Ok(offset as u32)
    }

    #[inline]
    pub fn array_order(&self) -> Result<u32> {
        let order = ffi!(dwarf_arrayorder(self.as_ptr()))?;
        Ok(order as u32)
    }

    #[inline]
    pub fn source_language(&self) -> Result<u32> {
        let lang = ffi!(dwarf_srclang(self.as_ptr()))?;
        Ok(lang as u32)
    }

    #[inline]
    pub fn unit(&self) -> Result<Die<'dw>> {
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
    pub fn child(&self) -> Result<Option<Die<'dw>>> {
        let die = Die::default();
        let rc = ffi!(dwarf_child(self.as_ptr(), die.as_ptr()))?;
        if rc == 0 {
            Ok(Some(die))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn siblingof(&self) -> Result<Option<Die<'dw>>> {
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
    pub fn iter_children(&self) -> DieChildren<'dw> {
        DieChildren {
            first: true,
            finished: false,
            die: self.clone(),
        }
    }

    pub fn for_each_child<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Die<'dw>) -> Result<bool>
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
    pub fn has_attr(&self, name: u32) -> Result<bool> {
        let b = ffi!(dwarf_hasattr(self.as_ptr(), name))?;
        Ok(b != 0)
    }

    #[inline]
    pub fn has_attr_integrate(&self, name: u32) -> Result<bool> {
        let b = ffi!(dwarf_hasattr_integrate(self.as_ptr(), name))?;
        Ok(b != 0)
    }

    #[inline]
    pub fn attr(&self, name: u32) -> Result<Attribute<'dw>> {
        let attr = Attribute::default();
        ffi!(dwarf_attr(self.as_ptr(), name, attr.as_ptr()))?;
        Ok(attr)
    }

    #[inline]
    pub fn attr_integrate(&self, name: u32) -> Result<Attribute<'dw>> {
        let attr = Attribute::default();
        ffi!(dwarf_attr_integrate(self.as_ptr(), name, attr.as_ptr()))?;
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
    pub fn attrs(&self) -> Result<Vec<Attribute<'dw>>> {
        let mut v = Vec::with_capacity(self.attr_count()?);
        unsafe {
            self.getattrs(|a| { v.push(a.clone()); ffi::DWARF_CB_OK })?;
        }
        Ok(v)
    }

    pub fn for_each_attr<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Attribute<'dw>) -> Result<bool>
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
        where F: FnMut(&Attribute<'dw>) -> Result<bool>
    {
        let mut result = Ok(());

        self.getattrs(|attr| dwarf_cb_map(f(attr), &mut result))?;

        result
    }

    unsafe fn getattrs<F>(&self, mut f: F) -> Result<isize>
        where F: FnMut(&Attribute<'dw>) -> libc::c_uint
    {
        let argp = &mut f as *mut F as *mut libc::c_void;
        return ffi!(dwarf_getattrs(self.as_ptr(), Some(callback::<'dw, F>), argp, 0));

        unsafe extern "C" fn callback<'a, F>(attr: *mut ffi::Dwarf_Attribute,
                                             argp: *mut libc::c_void)
                                         -> libc::c_int
            where F: FnMut(&Attribute<'a>) -> libc::c_uint
        {
            let f = &mut *(argp as *mut F);
            f(Attribute::from_ptr(attr)) as libc::c_int
        }
    }

    pub fn for_each_func<F>(&self, mut f: F) -> Result<()>
        where F: FnMut(&Die<'dw>) -> Result<bool>
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
        where F: FnMut(&Die<'dw>) -> Result<bool>
    {
        let mut result = Ok(());

        self.getfuncs(|func| dwarf_cb_map(f(func), &mut result))?;

        result
    }

    unsafe fn getfuncs<F>(&self, mut f: F) -> Result<isize>
        where F: FnMut(&Die<'dw>) -> libc::c_uint
    {
        let argp = &mut f as *mut F as *mut libc::c_void;
        return ffi!(dwarf_getfuncs(self.as_ptr(), Some(callback::<'dw, F>), argp, 0));

        unsafe extern "C" fn callback<'a, F>(func: *mut ffi::Dwarf_Die,
                                             argp: *mut libc::c_void)
                                             -> libc::c_int
            where F: FnMut(&Die<'a>) -> libc::c_uint
        {
            let f = &mut *(argp as *mut F);
            f(Die::from_ptr(func)) as libc::c_int
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf_Die {
        self.inner.get()
    }
}

impl<'dw> Clone for Die<'dw> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            Die {
                inner: ffi::Dwarf_Die { ..*self.as_ptr() }.into(),
                phantom: PhantomData,
            }
        }
    }
}


#[derive(Debug)]
pub struct DieChildren<'dw> {
    first: bool,
    finished: bool,
    die: Die<'dw>,
}

impl<'dw> Iterator for DieChildren<'dw> {
    type Item = Result<Die<'dw>>;

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


#[derive(Debug)]
pub struct DieRanges<'dw> {
    offset: isize,
    base: u64,
    die: Die<'dw>,
}

impl<'dw> Iterator for DieRanges<'dw> {
    type Item = Result<Range<u64>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut start = 0;
        let mut end = 0;

        let rc = ffi!(dwarf_ranges(
            self.die.as_ptr(),
            self.offset,
            &mut self.base,
            &mut start,
            &mut end
        ));

        match rc {
            Ok(0) => {
                self.offset = 0;
                None
            }
            Ok(offset) => {
                self.offset = offset;
                Some(Ok(start..end))
            }
            Err(e) => {
                self.offset = 0;
                Some(Err(e))
            }
        }
    }
}


#[inline]
fn dwarf_cb_map<T>(cont: Result<bool>, result: &mut Result<T>) -> libc::c_uint {
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
    payload: Option<Box<dyn Any + Send>>
}

impl CallbackGuard {
    #[inline]
    fn new() -> CallbackGuard {
        CallbackGuard {
            payload: None
        }
    }

    fn call<F>(&mut self, f: F) -> libc::c_uint
        where F: FnMut() -> libc::c_uint
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


#[cfg(test)]
mod tests {
    use super::Dwarf;

    #[test]
    fn die_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<crate::Die<'static>>(),
                   size_of::<crate::ffi::Dwarf_Die>());
    }

    #[test]
    fn die_align() {
        use std::mem::align_of;
        assert_eq!(align_of::<crate::Die<'static>>(),
                   align_of::<crate::ffi::Dwarf_Die>());
    }

    fn current() -> Dwarf<'static> {
        use std::env;
        let exe = env::current_exe().unwrap();
        Dwarf::open(exe).unwrap()
    }

    #[test]
    fn attr_callback() {
        for cu in current().compile_units() {
            let die = cu.unwrap().get_die().unwrap();
            die.for_each_attr(|_| Ok(true)).unwrap();
        }
    }

    #[test]
    fn attr_callback_unchecked() {
        for cu in current().compile_units() {
            let die = cu.unwrap().get_die().unwrap();
            unsafe {
                die.for_each_attr_unchecked(|_| Ok(true)).unwrap();
            }
        }
    }

    #[test]
    #[should_panic]
    fn attr_callback_panic() {
        for cu in current().compile_units() {
            let die = cu.unwrap().get_die().unwrap();
            die.for_each_attr(|_| panic!()).unwrap();
        }
    }
}
