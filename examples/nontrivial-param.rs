//! Scan for nontrivial parameters, like [cuviper/nontrivial-param][1].
//!
//! [1]: https://github.com/cuviper/nontrivial-param

use cpp_demangle::Symbol;
use libdw::{raw, Die};
use libdwfl::Dwfl;

use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};

fn main() -> Result<(), Box<dyn Error>> {
    for arg in env::args_os().skip(1) {
        let dwfl = Dwfl::open(arg)?;
        for dw in dwfl.dwarfs() {
            for cu in dw?.compile_units() {
                let die = cu?.get_die()?;
                if let Ok(raw::DW_TAG_compile_unit) = die.tag() {
                    die.for_each_func(process_function)?;
                }
            }
        }
    }
    Ok(())
}

fn process_function(function: &Die<'_>) -> libdw::Result<bool> {
    let file = match function.decl_file() {
        Ok(file) if !in_system_header(file) => file,
        _ => return Ok(true),
    };

    let mut printed_function_name = false;
    for child in function.iter_children() {
        let child = child?;
        if let Ok(raw::DW_TAG_formal_parameter) = child.tag() {
            if has_nontrivial_type(&child) {
                if !printed_function_name {
                    eprintln!("{:?}: In function {:?}:", file, function_name(function));
                    printed_function_name = true;
                }

                // TODO dwarf_decl_line
                let line = child.decl_line().map(|i| i as i32).unwrap_or(-1);
                let name = child.name().unwrap_or_default();
                eprintln!(
                    "{:?}:{}: note: parameter {:?} type is not trivial",
                    file, line, name
                );
            }
        }
    }
    Ok(true)
}

fn in_system_header(file: &CStr) -> bool {
    let bytes = file.to_bytes();
    bytes.starts_with(b"/usr/") && !bytes.starts_with(b"/usr/src/debug/")
}

fn has_nontrivial_type(die: &Die<'_>) -> bool {
    die.attr(raw::DW_AT_type)
        .and_then(|attr| attr.get_die())
        .map(|ty| match ty.tag() {
            Ok(raw::DW_TAG_class_type) | Ok(raw::DW_TAG_structure_type) => true,
            Ok(raw::DW_TAG_const_type)
            | Ok(raw::DW_TAG_volatile_type)
            | Ok(raw::DW_TAG_typedef) => has_nontrivial_type(&ty),
            _ => false,
        })
        .unwrap_or(false)
}

fn function_name<'dw>(function: &'dw Die<'_>) -> Cow<'dw, CStr> {
    let linkage_name = function
        .attr_integrate(raw::DW_AT_linkage_name)
        .or_else(|_| function.attr_integrate(raw::DW_AT_MIPS_linkage_name))
        .and_then(|attr| attr.get_string());

    if let Ok(name) = linkage_name {
        demangle(name)
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(name))
    } else if let Ok(name) = function.name() {
        Cow::Borrowed(name)
    } else {
        Cow::Borrowed(CStr::from_bytes_with_nul(b"(null)\0").unwrap())
    }
}

fn demangle(name: &CStr) -> Option<CString> {
    if let Ok(s) = name.to_str() {
        if let Ok(demangled) = rustc_demangle::try_demangle(s) {
            return CString::new(demangled.to_string()).ok();
        }
    }
    if let Ok(demangled) = Symbol::new(name.to_bytes()) {
        return CString::new(demangled.to_string()).ok();
    }
    None
}
