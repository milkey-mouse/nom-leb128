macro_rules! roundtrip_unsigned {
    ($int_ty:ident, $decoder:ident) => {
        libfuzzer_sys::fuzz_target!(|num: $int_ty| {
            // round-trip: encode/decode and compare to original

            let encoded = {
                let mut vec =
                    arrayvec::ArrayVec::<[_; nom_leb128::leb128_size::<$int_ty>()]>::new();
                let mut n = num;
                loop {
                    if n < 0x80 {
                        vec.push(n as u8);
                        break;
                    } else {
                        vec.push(((n & 0x7f) | 0x80) as u8);
                        n >>= 7;
                    }
                }
                vec
            };

            let decoded = nom_leb128::$decoder::<_, ()>(encoded.as_slice());

            assert_eq!(decoded, Ok((&[] as &[u8], num)));
        });
    };
}

macro_rules! roundtrip_signed {
    ($int_ty:ident, $decoder:ident) => {
        libfuzzer_sys::fuzz_target!(|num: $int_ty| {
            let encoded = {
                let mut vec =
                    arrayvec::ArrayVec::<[_; nom_leb128::leb128_size::<$int_ty>()]>::new();
                let mut n = num;
                loop {
                    let mut byte = (n as u8) & 0x7f;
                    n >>= 7;

                    let more =
                        !(((n == 0) && ((byte & 0x40) == 0)) || ((n == -1) && ((byte & 0x40) != 0)));

                    if more {
                        // mark this byte to show that more bytes will follow
                        byte |= 0x80;
                    }

                    vec.push(byte);

                    if !more {
                        break;
                    }
                }
                vec
            };

            let decoded = nom_leb128::$decoder::<_, ()>(encoded.as_slice());

            assert_eq!(decoded, Ok((&[] as &[u8], num)));
        });
    };
}
