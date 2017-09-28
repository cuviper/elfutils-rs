extern crate libelf;
extern crate libdw_sys as ffi;

pub mod raw {
    pub use ffi::*;
}

#[macro_use]
mod error;
pub use error::{Error, Result};

mod dwarf;
pub use dwarf::Dwarf;

mod units;
pub use units::{CompileUnits, CompileUnit, TypeUnits, TypeUnit};

mod die;
pub use die::{Die, DieChildren};

mod attr;
pub use attr::{Attribute, AttributeValue};


#[cfg(test)]
mod tests {
    use super::Dwarf;

    #[test]
    fn self_file() {
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        Dwarf::from_fd(&f).unwrap();
    }

    #[test]
    fn self_elf() {
        use libelf::Elf;
        use std::env;
        let exe = env::current_exe().unwrap();
        let elf = Elf::open(exe).unwrap();
        Dwarf::from_elf(&elf).unwrap();
    }

    #[test]
    fn attr_callback() {
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        let dw = Dwarf::from_fd(&f).unwrap();

        for cu in dw.compile_units() {
            let die = cu.unwrap().get_die().unwrap();
            die.for_each_attr(|_| Ok(true)).unwrap();
        }
    }

    #[test]
    fn attr_callback_unchecked() {
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        let dw = Dwarf::from_fd(&f).unwrap();

        for cu in dw.compile_units() {
            let die = cu.unwrap().get_die().unwrap();
            unsafe {
                die.for_each_attr_unchecked(|_| Ok(true)).unwrap();
            }
        }
    }

    #[test]
    #[should_panic]
    fn attr_callback_panic() {
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        let dw = Dwarf::from_fd(&f).unwrap();

        for cu in dw.compile_units() {
            let die = cu.unwrap().get_die().unwrap();
            die.for_each_attr(|_| panic!()).unwrap();
        }
    }

    #[test]
    fn attr_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<::Attribute<'static>>(),
                   size_of::<::ffi::Dwarf_Attribute>());
    }

    #[test]
    fn attr_align() {
        use std::mem::align_of;
        assert_eq!(align_of::<::Attribute<'static>>(),
                   align_of::<::ffi::Dwarf_Attribute>());
    }

    #[test]
    fn die_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<::Die<'static>>(),
                   size_of::<::ffi::Dwarf_Die>());
    }

    #[test]
    fn die_align() {
        use std::mem::align_of;
        assert_eq!(align_of::<::Die<'static>>(),
                   align_of::<::ffi::Dwarf_Die>());
    }
}
