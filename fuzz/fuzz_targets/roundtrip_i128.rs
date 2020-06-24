#![no_main]
#[macro_use]
mod macros;

roundtrip_signed!(i128, leb128_i128);
