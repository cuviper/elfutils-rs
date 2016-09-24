extern crate libc;
extern crate libelf;
extern crate libdw_sys as ffi;


#[macro_use]
mod error;
pub use error::{Error, Result};

mod dwarf;
pub use dwarf::Dwarf;

mod units;
pub use units::{CompileUnits, CompileUnit, TypeUnits, TypeUnit};

mod die;
pub use die::Die;


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
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        let elf = Elf::from_fd(&f).unwrap();
        Dwarf::from_elf(&elf).unwrap();
    }
}
