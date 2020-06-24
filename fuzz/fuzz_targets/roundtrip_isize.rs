#![no_main]
#[macro_use]
mod macros;

roundtrip_signed!(isize, leb128_isize);
