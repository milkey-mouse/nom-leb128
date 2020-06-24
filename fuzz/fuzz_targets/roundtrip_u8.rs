#![no_main]
#[macro_use]
mod macros;

roundtrip_unsigned!(u8, leb128_u8);
