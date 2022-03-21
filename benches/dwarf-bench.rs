// inspired by / rewrite of:
// https://github.com/philipc/dwarf-bench

#![feature(test)]

extern crate test;

use std::env;
use std::path;

use libdw::{Die, Dwarf, Result};

fn test_path() -> path::PathBuf {
    match env::var_os("BENCH_FILE") {
        Some(file) => file.into(),
        None => env::current_exe().unwrap(),
    }
}

#[bench]
fn info_cus(b: &mut test::Bencher) {
    b.iter(|| -> Result<()> {
        let dw = Dwarf::open(test_path())?;

        for cu in dw.compile_units() {
            let die = cu?.get_die()?;
            for child in die.iter_children() {
                test::black_box(&child?);
            }
        }

        Ok(())
    });
}

#[bench]
fn info_dies(b: &mut test::Bencher) {
    b.iter(|| -> Result<()> {
        let dw = Dwarf::open(test_path())?;

        for cu in dw.compile_units() {
            recurse_die(&cu?.get_die()?)?;
        }

        Ok(())
    });

    fn recurse_die(die: &Die<'_>) -> Result<()> {
        test::black_box(die);

        if die.has_children()? {
            for child in die.iter_children() {
                recurse_die(&child?)?;
            }
        }

        Ok(())
    }
}

#[bench]
fn info_iter(b: &mut test::Bencher) {
    b.iter(|| -> Result<()> {
        let dw = Dwarf::open(test_path())?;

        for cu in dw.compile_units() {
            recurse_die(&cu?.get_die()?)?;
        }

        Ok(())
    });

    fn recurse_die(die: &Die<'_>) -> Result<()> {
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
        let dw = Dwarf::open(test_path())?;

        for cu in dw.compile_units() {
            recurse_die(&cu?.get_die()?)?;
        }

        Ok(())
    });

    fn recurse_die(die: &Die<'_>) -> Result<()> {
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

#[bench]
fn info_nest_unchecked(b: &mut test::Bencher) {
    b.iter(|| -> Result<()> {
        let dw = Dwarf::open(test_path())?;

        for cu in dw.compile_units() {
            recurse_die(&cu?.get_die()?)?;
        }

        Ok(())
    });

    fn recurse_die(die: &Die<'_>) -> Result<()> {
        unsafe {
            die.for_each_attr_unchecked(|attr| {
                test::black_box(attr);
                Ok(true)
            })?;
        }

        die.for_each_child(|child| {
            recurse_die(child)?;
            Ok(true)
        })?;

        Ok(())
    }
}

mod orig {
    use libdw_sys as libdw;

    use std::mem::MaybeUninit;
    use std::os::unix::io::AsRawFd;

    use super::test_path;

    #[bench]
    fn info_elfutils(b: &mut test::Bencher) {
        b.iter(|| {
            let null = std::ptr::null_mut::<libc::c_void>();
            let file = std::fs::File::open(test_path()).unwrap();
            let fd = file.as_raw_fd();
            let dwarf = unsafe { libdw::dwarf_begin(fd, libdw::Dwarf_Cmd::DWARF_C_READ) };
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
                        &mut offset_size,
                    )
                };
                if res > 0 {
                    break;
                }
                assert_eq!(res, 0);

                let offdie = offset + header_size as u64;
                let mut stack = Vec::new();
                let mut die = MaybeUninit::uninit();
                unsafe {
                    let res = libdw::dwarf_offdie(dwarf, offdie, die.as_mut_ptr());
                    assert_eq!(res, die.as_mut_ptr());
                };
                stack.push(die);

                loop {
                    let res = unsafe {
                        libdw::dwarf_getattrs(die.as_mut_ptr(), Some(info_elfutils_attr), null, 0)
                    };
                    assert_eq!(res, 1);

                    let mut next_die = MaybeUninit::uninit();
                    let res =
                        unsafe { libdw::dwarf_child(die.as_mut_ptr(), next_die.as_mut_ptr()) };
                    assert!(res >= 0);

                    if res > 0 {
                        // No child, so read sibling
                        loop {
                            let res = unsafe {
                                libdw::dwarf_siblingof(die.as_mut_ptr(), next_die.as_mut_ptr())
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

    unsafe extern "C" fn info_elfutils_attr(
        _: *mut libdw::Dwarf_Attribute,
        _: *mut libc::c_void,
    ) -> i32 {
        0
    }
}
