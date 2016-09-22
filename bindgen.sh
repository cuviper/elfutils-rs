#!/bin/bash

set -e -x

opts=(--ctypes-prefix=libc --no-rust-enums --convert-macros
--macro-int-types=uint,uint,uint,ulonglong,sint,sint,sint,slonglong)

bindgen "${opts[@]}" /usr/include/gelf.h --match=elf.h \
    --output libelf-sys/src/lib.rs
echo 'include!{"imports.rs"}' >>$_


bindgen "${opts[@]}" /usr/include/dwarf.h --match=dwarf.h \
    --output libdw-sys/src/dwarf.rs
sed -i -e 's/\bEnum/DW_&/g' $_

bindgen "${opts[@]}" /usr/include/elfutils/libdwfl.h --match=elfutils \
    --output libdw-sys/src/lib.rs
echo 'include!{"imports.rs"}' >>$_
