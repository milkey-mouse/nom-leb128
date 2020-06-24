#![no_main]
#[macro_use]
mod macros;

roundtrip_signed!(i32, leb128_i32);
