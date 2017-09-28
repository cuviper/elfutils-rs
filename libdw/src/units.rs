use ffi;
use std::ptr;

use ffi::Dwarf_Off;

use super::Result;
use super::Dwarf;
use super::Die;


#[derive(Debug)]
pub struct CompileUnits<'dw> {
    dwarf: &'dw Dwarf<'dw>,
    offset: Dwarf_Off,
    finished: bool,
}

#[inline]
pub fn compile_units<'dw>(dwarf: &'dw Dwarf<'dw>) -> CompileUnits<'dw> {
    CompileUnits {
        dwarf: dwarf,
        offset: 0,
        finished: false,
    }
}

impl<'dw> Iterator for CompileUnits<'dw> {
    type Item = Result<CompileUnit<'dw>>;

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
pub struct TypeUnits<'dw> {
    dwarf: &'dw Dwarf<'dw>,
    offset: Dwarf_Off,
    finished: bool,
}

#[inline]
pub fn type_units<'dw>(dwarf: &'dw Dwarf<'dw>) -> TypeUnits<'dw> {
    TypeUnits {
        dwarf: dwarf,
        offset: 0,
        finished: false,
    }
}

impl<'dw> Iterator for TypeUnits<'dw> {
    type Item = Result<TypeUnit<'dw>>;

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
pub struct CompileUnit<'dw> {
    dwarf: &'dw Dwarf<'dw>,
    die_offset: Dwarf_Off,
}

impl<'dw> CompileUnit<'dw> {
    #[inline]
    fn new(dwarf: &'dw Dwarf<'dw>, die_offset: Dwarf_Off) -> CompileUnit<'dw>
    {
        CompileUnit {
            dwarf: dwarf,
            die_offset: die_offset,
        }
    }

    #[inline]
    pub fn get_die(&self) -> Result<Die<'dw>> {
        Die::from_offset(self.dwarf, self.die_offset)
    }
}


#[derive(Debug)]
pub struct TypeUnit<'dw> {
    dwarf: &'dw Dwarf<'dw>,
    die_offset: Dwarf_Off,
    type_offset: Dwarf_Off,
    signature: u64,
}

impl<'dw> TypeUnit<'dw> {
    #[inline]
    fn new(dwarf: &'dw Dwarf<'dw>, die_offset: Dwarf_Off,
           type_offset: Dwarf_Off, signature: u64)
        -> TypeUnit<'dw>
    {
        TypeUnit {
            dwarf: dwarf,
            die_offset: die_offset,
            type_offset: type_offset,
            signature: signature,
        }
    }

    #[inline]
    pub fn get_die(&self) -> Result<Die<'dw>> {
        Die::from_type_offset(self.dwarf, self.die_offset)
    }

    #[inline]
    pub fn get_type_die(&self) -> Result<Die<'dw>> {
        Die::from_type_offset(self.dwarf, self.type_offset)
    }

    #[inline]
    pub fn signature(&self) -> u64 {
        self.signature
    }
}
