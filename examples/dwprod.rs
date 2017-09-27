/// Print all `DW_AT_producer` strings

extern crate libdw;

use std::env;
use std::fs;

use std::error::Error;

fn main() {
    try_main().unwrap();
}

fn try_main() -> Result<(), Box<Error>> {
    for arg in env::args_os().skip(1) {
        let f = fs::File::open(arg)?;
        let dw = libdw::Dwarf::from_fd(&f)?;

        for cu in dw.compile_units() {
            let die = cu?.get_die()?;

            if let Ok(attr) = die.attr(libdw::raw::DW_AT_producer) {
                if let Ok(s) = attr.to_cstr() {
                    println!("{:?}", s);
                }
            }
        }
    }
    Ok(())
}
