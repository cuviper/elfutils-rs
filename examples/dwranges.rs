//! Print the ranges of each CU.

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    for arg in env::args_os().skip(1) {
        let dw = libdw::Dwarf::open(arg)?;

        for cu in dw.compile_units() {
            let die = cu?.get_die()?;
            match die.name() {
                Ok(name) => println!("CU: {:?}", name),
                Err(e) => println!("CU: Err({})", e),
            }

            for range in die.ranges() {
                let range = range?;
                println!("\t{:x}..{:x}", range.start, range.end);
            }
        }
    }
    Ok(())
}
