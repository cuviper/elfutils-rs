// inspired by / rewrite of:
// https://github.com/philipc/dwarf-bench

#![feature(test)]

extern crate libc;
extern crate libdw;
extern crate libdw_sys;
extern crate test;

use std::env;
use std::fs;
use std::path;

use libdw::{Result, Dwarf, Die};

fn test_path() -> path::PathBuf {
    match env::var_os("BENCH_FILE") {
        Some(file) => file.into(),
        None => env::current_exe().unwrap(),
    }
}


#[bench]
fn info_iter(b: &mut test::Bencher) {
    b.iter(|| -> Result<()> {
        let f = fs::File::open(test_path()).unwrap();
        let dw = Dwarf::from_fd(&f)?;

        for cu in dw.compile_units() {
            recurse_die(&cu?.get_die()?)?;
        }

        Ok(())
    });

    fn recurse_die(die: &Die) -> Result<()> {
        for attr in &die.attrs()? {
            test::black_box(attr);
        }

        if die.has_children()? {
            for child in die.iter_children() {
                recurse_die(&child?)?;
            }
        }

        Ok(())
    }
}


#[bench]
fn info_nested(b: &mut test::Bencher) {
    b.iter(|| -> Result<()> {
        let f = fs::File::open(test_path()).unwrap();
        let dw = Dwarf::from_fd(&f)?;

        for cu in dw.compile_units() {
            recurse_die(&cu?.get_die()?)?;
        }

        Ok(())
    });

    fn recurse_die(die: &Die) -> Result<()> {
        die.for_each_attr(|attr| {
            test::black_box(attr);
            Ok(true)
        })?;

        die.for_each_child(|child| {
            recurse_die(child)?;
            Ok(true)
        })?;

        Ok(())
    }
}



mod orig {
    use libdw_sys as libdw;
    use std;
    use test;

    use std::os::raw;
    use std::os::unix::io::AsRawFd;

    use super::test_path;

    #[bench]
    fn info_elfutils(b: &mut test::Bencher) {
        b.iter(|| {
            let null = std::ptr::null_mut::<raw::c_void>();
            let file = std::fs::File::open(test_path()).unwrap();
            let fd = file.as_raw_fd();
            let dwarf = unsafe {
                libdw::dwarf_begin(fd, libdw::DWARF_C_READ)
            };
            assert!(dwarf != null as *mut libdw::Dwarf);

            let mut offset = 0;
            loop {
                let mut next_offset = 0;
                let mut header_size = 0;
                let mut abbrev_offset = 0;
                let mut address_size = 0;
                let mut offset_size = 0;
                let res = unsafe {
                    libdw::dwarf_nextcu(
                        dwarf,
                        offset,
                        &mut next_offset,
                        &mut header_size,
                        &mut abbrev_offset,
                        &mut address_size,
                        &mut offset_size)
                };
                if res > 0 {
                    break;
                }
                assert_eq!(res, 0);

                let offdie = offset + header_size as u64;
                let mut stack = Vec::new();
                let mut die;
                unsafe {
                    die = std::mem::uninitialized();
                    let res = libdw::dwarf_offdie(dwarf, offdie, &mut die);
                    assert_eq!(res, &mut die as *mut _);
                };
                stack.push(die);

                loop {
                    let res = unsafe {
                        libdw::dwarf_getattrs(&mut die, Some(info_elfutils_attr), null, 0)
                    };
                    assert_eq!(res, 1);

                    let mut next_die;
                    let res = unsafe {
                        next_die = std::mem::uninitialized();
                        libdw::dwarf_child(&mut die, &mut next_die)
                    };
                    assert!(res >= 0);

                    if res > 0 {
                        // No child, so read sibling
                        loop {
                            let res = unsafe {
                                next_die = std::mem::uninitialized();
                                libdw::dwarf_siblingof(&mut die, &mut next_die)
                            };
                            assert!(res >= 0);

                            if res > 0 {
                                // No sibling, so pop parent
                                if stack.len() == 0 {
                                    break;
                                }
                                die = stack.pop().unwrap();
                            } else {
                                // Sibling
                                die = next_die;
                                break;
                            }
                        }
                        if stack.len() == 0 {
                            break;
                        }
                    } else {
                        // Child, so push parent
                        stack.push(die);
                        die = next_die;
                    }
                }

                offset = next_offset;
            }
        });
    }

    unsafe extern "C" fn info_elfutils_attr(_: *mut libdw::Dwarf_Attribute, _: *mut raw::c_void) -> i32{
        0
    }
}
