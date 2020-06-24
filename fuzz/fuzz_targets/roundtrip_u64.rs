#![no_main]
#[macro_use]
mod macros;

roundtrip_unsigned!(u64, leb128_u64);
