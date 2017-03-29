use ffi;
use std::ptr;

use ffi::Dwarf_Off;

use super::Result;
use super::Dwarf;
use super::Die;


#[derive(Debug)]
pub struct CompileUnits<'a> {
    dwarf: &'a Dwarf<'a>,
    offset: Dwarf_Off,
    finished: bool,
}

#[inline]
pub fn compile_units<'a>(dwarf: &'a Dwarf<'a>) -> CompileUnits<'a> {
    CompileUnits {
        dwarf: dwarf,
        offset: 0,
        finished: false,
    }
}

impl<'a> Iterator for CompileUnits<'a> {
    type Item = Result<CompileUnit<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None }

        let offset = self.offset;
        let mut header_size = 0;

        let rc = ffi!(
            dwarf_next_unit(self.dwarf.as_ptr(), offset, &mut self.offset,
                &mut header_size, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
                ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
            );

        match rc {
            Ok(0) => {
                let die_offset = offset + header_size as Dwarf_Off;
                Some(Ok(CompileUnit::new(self.dwarf, die_offset)))
            },
            Ok(_) => { self.finished = true; None },
            Err(e) => { self.finished = true; Some(Err(e)) },
        }
    }
}


#[derive(Debug)]
pub struct TypeUnits<'a> {
    dwarf: &'a Dwarf<'a>,
    offset: Dwarf_Off,
    finished: bool,
}

#[inline]
pub fn type_units<'a>(dwarf: &'a Dwarf<'a>) -> TypeUnits<'a> {
    TypeUnits {
        dwarf: dwarf,
        offset: 0,
        finished: false,
    }
}

impl<'a> Iterator for TypeUnits<'a> {
    type Item = Result<TypeUnit<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None }

        let offset = self.offset;
        let mut header_size = 0;
        let mut signature = 0;
        let mut type_offset = 0;

        let rc = ffi!(
            dwarf_next_unit(self.dwarf.as_ptr(), offset, &mut self.offset,
                &mut header_size, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
                ptr::null_mut(), &mut signature, &mut type_offset)
            );

        match rc {
            Ok(0) => {
                let die_offset = offset + header_size as Dwarf_Off;
                let type_offset = offset + type_offset;
                Some(Ok(TypeUnit::new(self.dwarf, die_offset, type_offset, signature)))
            },
            Ok(_) => { self.finished = true; None },
            Err(e) => { self.finished = true; Some(Err(e)) },
        }
    }
}


#[derive(Debug)]
pub struct CompileUnit<'a> {
    dwarf: &'a Dwarf<'a>,
    die_offset: Dwarf_Off,
}

impl<'a> CompileUnit<'a> {
    #[inline]
    fn new(dwarf: &'a Dwarf<'a>, die_offset: Dwarf_Off) -> CompileUnit<'a>
    {
        CompileUnit {
            dwarf: dwarf,
            die_offset: die_offset,
        }
    }

    #[inline]
    pub fn get_die(&self) -> Result<Die<'a>> {
        self.dwarf.offdie(self.die_offset)
    }
}


#[derive(Debug)]
pub struct TypeUnit<'a> {
    dwarf: &'a Dwarf<'a>,
    die_offset: Dwarf_Off,
    type_offset: Dwarf_Off,
    signature: u64,
}

impl<'a> TypeUnit<'a> {
    #[inline]
    fn new(dwarf: &'a Dwarf<'a>, die_offset: Dwarf_Off,
           type_offset: Dwarf_Off, signature: u64)
        -> TypeUnit<'a>
    {
        TypeUnit {
            dwarf: dwarf,
            die_offset: die_offset,
            type_offset: type_offset,
            signature: signature,
        }
    }

    #[inline]
    pub fn get_die(&self) -> Result<Die<'a>> {
        self.dwarf.offdie_types(self.die_offset)
    }

    #[inline]
    pub fn get_type_die(&self) -> Result<Die<'a>> {
        self.dwarf.offdie_types(self.type_offset)
    }

    #[inline]
    pub fn signature(&self) -> u64 {
        self.signature
    }
}
