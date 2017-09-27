use ffi;

use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::ptr;
use std::slice;

use super::Result;
use super::Dwarf;
use super::Die;

pub struct Attribute<'a> {
    inner: UnsafeCell<ffi::Dwarf_Attribute>,
    phantom: PhantomData<&'a Dwarf<'a>>,
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    String(&'a CStr),
    Unsigned(u64),
    Signed(i64),
    Address(u64),
    Die(Die<'a>),
    Bytes(&'a [u8]),
    Bool(bool),
    #[doc(hidden)] // non-exhaustive
    UnknownForm(u32),
}

impl<'a> Default for Attribute<'a> {
    #[inline]
    fn default() -> Self {
        Attribute {
            inner: ffi::Dwarf_Attribute {
                code: 0,
                form: 0,
                valp: ptr::null_mut(),
                cu: ptr::null_mut(),
            }.into(),
            phantom: PhantomData,
        }
    }
}

impl<'a> fmt::Debug for Attribute<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("Attribute")
            .field(unsafe { &*self.as_ptr() })
            .finish()
    }
}

impl<'a> Attribute<'a> {
    #[inline]
    unsafe fn from_raw(attr: *mut ffi::Dwarf_Attribute) -> Self {
        Attribute {
            inner: UnsafeCell::new(*attr),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn attr(&self) -> u32 {
        raw_ffi!(dwarf_whatattr(self.as_ptr()))
    }

    #[inline]
    pub fn form(&self) -> u32 {
        raw_ffi!(dwarf_whatform(self.as_ptr()))
    }

    #[inline]
    pub fn has_form(&self, form: u32) -> Result<bool> {
        let b = ffi!(dwarf_hasform(self.as_ptr(), form))?;
        Ok(b != 0)
    }

    #[inline]
    pub fn to_cstr(&self) -> Result<&'a CStr> {
        let s = ffi!(dwarf_formstring(self.as_ptr()))?;
        Ok(unsafe { CStr::from_ptr(s) })
    }

    #[inline]
    pub fn to_unsigned(&self) -> Result<u64> {
        let mut data = 0;
        ffi!(dwarf_formudata(self.as_ptr(), &mut data))?;
        Ok(data)
    }

    #[inline]
    pub fn to_signed(&self) -> Result<i64> {
        let mut data = 0;
        ffi!(dwarf_formsdata(self.as_ptr(), &mut data))?;
        Ok(data)
    }

    #[inline]
    pub fn to_address(&self) -> Result<u64> {
        let mut addr = 0;
        ffi!(dwarf_formaddr(self.as_ptr(), &mut addr))?;
        Ok(addr)
    }

    #[inline]
    pub fn to_die(&self) -> Result<Die<'a>> {
        let die = Die::default();
        ffi!(dwarf_formref_die(self.as_ptr(), die.as_ptr()))?;
        Ok(die)
    }

    #[inline]
    pub fn to_bytes(&self) -> Result<&'a [u8]> {
        let mut block = ffi::Dwarf_Block { length: 0, data: ptr::null_mut() };
        ffi!(dwarf_formblock(self.as_ptr(), &mut block))?;
        Ok(unsafe { slice::from_raw_parts(block.data, block.length as usize) })
    }

    #[inline]
    pub fn to_bool(&self) -> Result<bool> {
        let mut flag = false;
        ffi!(dwarf_formflag(self.as_ptr(), &mut flag))?;
        Ok(flag)
    }

    pub fn to_value(&self) -> Result<AttributeValue<'a>> {
        use self::AttributeValue as V;
        let value = match self.form() {
            ffi::DW_FORM_addr => V::Address(self.to_address()?),

            ffi::DW_FORM_indirect |
            ffi::DW_FORM_strp |
            ffi::DW_FORM_string |
            ffi::DW_FORM_GNU_strp_alt => V::String(self.to_cstr()?),

            ffi::DW_FORM_ref_addr |
            ffi::DW_FORM_ref_udata |
            ffi::DW_FORM_ref8 |
            ffi::DW_FORM_ref4 |
            ffi::DW_FORM_ref2 |
            ffi::DW_FORM_ref1 |
            ffi::DW_FORM_ref_sig8 |
            ffi::DW_FORM_GNU_ref_alt => V::Die(self.to_die()?),

            ffi::DW_FORM_sec_offset |
            ffi::DW_FORM_udata |
            ffi::DW_FORM_data8 |
            ffi::DW_FORM_data4 |
            ffi::DW_FORM_data2 |
            ffi::DW_FORM_data1 => V::Unsigned(self.to_unsigned()?),

            ffi::DW_FORM_sdata => V::Signed(self.to_signed()?),

            ffi::DW_FORM_flag_present |
            ffi::DW_FORM_flag => V::Bool(self.to_bool()?),

            ffi::DW_FORM_exprloc |
            ffi::DW_FORM_block4 |
            ffi::DW_FORM_block2 |
            ffi::DW_FORM_block1 |
            ffi::DW_FORM_block => V::Bytes(self.to_bytes()?),

            form => V::UnknownForm(form),
        };
        Ok(value)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::Dwarf_Attribute {
        self.inner.get()
    }
}

impl<'a> Clone for Attribute<'a> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Attribute::from_raw(self.as_ptr()) }
    }
}
