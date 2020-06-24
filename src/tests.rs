use super::*;
use arrayvec::ArrayVec;
use nom::{
    error::{
        ErrorKind::TooLarge,
        VerboseError,
        VerboseErrorKind::{Context, Nom},
    },
    Err::{Error, Incomplete},
};
use num_traits::{AsPrimitive, PrimInt, Signed, Unsigned};

// TODO: when https://github.com/rust-lang/rust/issues/43408 is fixed,
// write_*_leb128 can use exactly-sized ArrayVecs with const leb128_size
const MAX_ENCODED_SIZE: usize = leb128_size::<u128>();

#[inline]
pub fn write_unsigned_leb128<T>(mut n: T) -> ArrayVec<[u8; MAX_ENCODED_SIZE]>
where
    T: AsPrimitive<u8> + PrimInt + Unsigned,
    u8: AsPrimitive<T>,
{
    let mut vec = ArrayVec::new();
    loop {
        if n < 0x80.as_() {
            vec.push(n.as_());
            return vec;
        } else {
            vec.push((n.as_() & 0x7f) | 0x80);
            n = n >> 7;
        }
    }
}

#[inline]
pub fn write_signed_leb128<T>(mut n: T) -> ArrayVec<[u8; MAX_ENCODED_SIZE]>
where
    T: AsPrimitive<u8> + PrimInt + Signed,
{
    let mut vec = ArrayVec::new();
    loop {
        let mut byte = n.as_() & 0x7f;
        n = n >> 7;

        let more = !(((n == T::zero()) && ((byte & 0x40) == 0))
            || ((n == -T::one()) && ((byte & 0x40) != 0)));
        if more {
            // mark this byte to show that more bytes will follow
            byte |= 0x80;
        }

        vec.push(byte);

        if !more {
            return vec;
        }
    }
}

macro_rules! test_roundtrip {
    ($int_ty:ident, $encoder:ident, $decoder:ident, $iter:expr) => {
        #[test]
        fn $int_ty() {
            for n in $iter {
                let encoded = $encoder(n);
                // TODO: use nom::VerboseError instead of ()
                let decoded = $decoder::<_, VerboseError<_>>(encoded.as_slice()).unwrap();

                assert_eq!(decoded, (&[] as &[u8], n));
            }
        }
    };
}

macro_rules! test_bruteforce {
    ($int_ty:ident, $encoder:ident, $decoder:ident) => {
        // u8 and u16 are so small we might as well "brute-force" all possibilities.
        test_roundtrip!($int_ty, $encoder, $decoder, ($int_ty::MIN..$int_ty::MAX));
    };
}

test_bruteforce!(u8, write_unsigned_leb128, leb128_u8);
test_bruteforce!(u16, write_unsigned_leb128, leb128_u16);
test_bruteforce!(i8, write_signed_leb128, leb128_i8);
test_bruteforce!(i16, write_signed_leb128, leb128_i16);

macro_rules! test_tricky_unsigned {
    ($int_ty:ident, $decoder:ident) => {
        // test particularly "tricky" numbers
        test_roundtrip!(
            $int_ty,
            write_unsigned_leb128,
            $decoder,
            [0, 1, 128, 255, 256, 65535, 65536, $int_ty::MAX]
                .iter()
                .copied()
        );
    };
}

test_tricky_unsigned!(u32, leb128_u32);
test_tricky_unsigned!(u64, leb128_u64);
test_tricky_unsigned!(u128, leb128_u128);
test_tricky_unsigned!(usize, leb128_usize);

macro_rules! test_tricky_signed {
    ($int_ty:ident, $decoder:ident) => {
        test_roundtrip!(
            $int_ty,
            write_signed_leb128,
            $decoder,
            [
                0,
                1,
                128,
                255,
                256,
                65535,
                65536,
                $int_ty::MAX,
                -1,
                -128,
                -255,
                -256,
                -65535,
                -65536,
                $int_ty::MIN,
            ]
            .iter()
            .copied()
        );
    };
}

test_tricky_signed!(i32, leb128_i32);
test_tricky_signed!(i64, leb128_i64);
test_tricky_signed!(i128, leb128_i128);
test_tricky_signed!(isize, leb128_isize);

// TODO: add some should_fail tests
#[test]
fn data_after_num() {
    let mut vec = write_unsigned_leb128(1337u32);
    vec.extend(b"hello".iter().copied());

    let (slice, decoded) = leb128_u32::<_, VerboseError<_>>(vec.as_slice()).unwrap();

    assert_eq!(slice, b"hello");
    assert_eq!(decoded, 1337);
}

#[test]
fn truncated_num() {
    let mut vec = write_unsigned_leb128(u64::MAX);
    vec.truncate(2);

    let res = leb128_u32::<_, VerboseError<_>>(vec.as_slice());

    assert_eq!(res, Err(Incomplete(NEED_ONE)));
}

#[test]
fn overflow() {
    let vec = write_unsigned_leb128(u64::MAX);

    let res = leb128_u16::<_, VerboseError<_>>(vec.as_slice());

    if let Err(Error(verbose_error)) = res {
        assert_eq!(verbose_error.errors[0].1, Nom(TooLarge));
        assert_eq!(
            verbose_error.errors[1].1,
            Context("LEB128 integer is too big to fit in u16")
        );
    } else {
        panic!();
    }
}

#[test]
fn minimal_error() {
    let vec = write_unsigned_leb128(1337u32);

    leb128_u32::<_, ()>(vec.as_slice()).unwrap();
}
