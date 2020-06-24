#![no_main]
#[macro_use]
mod macros;

roundtrip_signed!(i8, leb128_i8);
