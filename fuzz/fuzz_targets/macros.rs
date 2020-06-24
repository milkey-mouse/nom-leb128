#![allow(unused_macros)]

macro_rules! roundtrip_unsigned {
    ($int_ty:ident, $decoder:ident) => {
        libfuzzer_sys::fuzz_target!(|num: $int_ty| {
            // round-trip: encode/decode and compare to original

            let encoded = nom_leb128::write_unsigned_leb128(num);
            let decoded = nom_leb128::$decoder::<_, ()>(encoded.as_slice());

            assert_eq!(decoded, Ok((&[] as &[u8], num)));
        });
    };
}

macro_rules! roundtrip_signed {
    ($int_ty:ident, $decoder:ident) => {
        libfuzzer_sys::fuzz_target!(|num: $int_ty| {
            let encoded = nom_leb128::write_signed_leb128(num);
            let decoded = nom_leb128::$decoder::<_, ()>(encoded.as_slice());

            assert_eq!(decoded, Ok((&[] as &[u8], num)));
        });
    };
}
