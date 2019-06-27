//! Print all `DW_AT_producer` strings

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    for arg in env::args_os().skip(1) {
        let dw = libdw::Dwarf::open(arg)?;

        for cu in dw.compile_units() {
            let die = cu?.get_die()?;

            if let Ok(attr) = die.attr(libdw::raw::DW_AT_producer) {
                if let Ok(s) = attr.get_string() {
                    println!("{:?}", s);
                }
            }
        }
    }
    Ok(())
}
