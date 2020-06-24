#![no_main]
#[macro_use]
mod macros;

roundtrip_unsigned!(u128, leb128_u128);
