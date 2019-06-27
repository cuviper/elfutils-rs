//! Reimplementation of elfutils/tests/allfcts.c

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    for arg in env::args_os().skip(1) {
        let dw = libdw::Dwarf::open(arg)?;
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
