#![no_main]
#[macro_use]
mod macros;

roundtrip_unsigned!(u16, leb128_u16);
