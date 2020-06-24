#![no_main]
#[macro_use]
mod macros;

roundtrip_unsigned!(usize, leb128_usize);
