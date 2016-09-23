extern crate libc;
extern crate libelf;
extern crate libdw_sys as ffi;


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
        dump(&Dwarf::from_fd(&f).unwrap());
    }

    #[test]
    fn self_elf() {
        use libelf::Elf;
        use std::{fs, env};
        let exe = env::current_exe().unwrap();
        let f = fs::File::open(exe).unwrap();
        let elf = Elf::from_fd(&f).unwrap();
        dump(&Dwarf::from_elf(&elf).unwrap());
    }

    fn dump<'a>(dw: &'a Dwarf<'a>) {
        for cu in dw.compile_units() {
            if let Ok(cu) = cu {
                println!("{:?}", cu);
                if let Ok(die) = cu.get_die() {
                    recurse_die(0, die);
                }
            }
        }

        for tu in dw.type_units() {
            let tu = tu.unwrap();
            let die = tu.get_die().unwrap();
            println!("{{{:016x}}} {:?}", tu.signature(), tu);
            recurse_die(0, die);
        }

        fn recurse_die<'a>(indent: usize, die: ::die::Die<'a>) {
            println!("{0:1$}{2:?}", "", 2 * indent, die);
            die.with_attrs(|a| {
                println!("\t{0:1$}{2:?}", "", 2 * indent, a);
                true
            }).unwrap();
            for child in die.children() {
                let child = child.unwrap();
                recurse_die(indent + 1, child)
            }
        }
    }
}
