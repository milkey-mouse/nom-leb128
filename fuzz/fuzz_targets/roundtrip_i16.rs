#![no_main]
#[macro_use]
mod macros;

roundtrip_signed!(i16, leb128_i16);
