//! Reimplementation of elfutils/tests/allfcts.c

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
        // TODO setup_alt

        for cu in dw.compile_units() {
            let die = cu?.get_die()?;

            // TODO explicitly stop and resume
            die.for_each_func(|func| {
                // TODO file:line:name
                println!("?:?:? {:#x}", func.offset());
                Ok(true)
            })?;
        }
    }
    Ok(())
}
