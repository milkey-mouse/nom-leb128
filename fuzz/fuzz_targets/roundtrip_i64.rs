#![no_main]
#[macro_use]
mod macros;

roundtrip_signed!(i64, leb128_i64);
