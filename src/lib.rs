use nom::{
    error::{make_error, ContextError, ErrorKind, ParseError},
    IResult, InputIter, InputLength, Needed, Slice,
};
use num_traits::{PrimInt, Signed, WrappingNeg};
use std::{
    mem::size_of,
    num::NonZeroUsize,
    ops::{BitOrAssign, RangeFrom},
};

// TODO: when stable, use const NonZeroUsize::new(1).unwrap()
const NEED_ONE: nom::Needed = Needed::Size(unsafe { NonZeroUsize::new_unchecked(1) });

/// Maximum LEB128-encoded size of an integer type
const fn leb128_size<T>() -> usize {
    let bits = size_of::<T>() * 8;
    (bits + 6) / 7 // equivalent to ceil(bits/7) w/o floats
}

macro_rules! impl_generic_leb128 {
    ($fn_name:ident, $int_ty:ident, $post:tt, $int_name:expr) => {
        #[doc="Recognizes an LEB128-encoded number that fits in a `"]
        #[doc=$int_name]
        #[doc="`."]
        #[inline]
        pub fn $fn_name<I, E>(input: I) -> IResult<I, $int_ty, E>
        where
            I: Clone + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
            E: ParseError<I> + ContextError<I>,
        {
            let mut res = 0;
            let mut shift = 0;

            for (pos, byte) in input.iter_indices() {
                if (byte & 0x80) == 0 {
                    res |= (byte as $int_ty) << shift;
                    $post(&mut res, shift, byte);
                    return Ok((input.slice(pos + 1..), res));
                } else if pos == leb128_size::<$int_ty>() - 1 {
                    return Err(nom::Err::Error(E::add_context(
                        input.clone(),
                        concat!("LEB128 integer is too big to fit in ", $int_name),
                        make_error(input, ErrorKind::TooLarge),
                    )));
                } else {
                    res |= ((byte & 0x7F) as $int_ty) << shift;
                }
                shift += 7;
            }

            Err(nom::Err::Incomplete(NEED_ONE))
        }
    };
    ($fn_name:ident, $int_ty:ident, $post:tt) => {
        impl_generic_leb128!($fn_name, $int_ty, $post, stringify!($int_ty));
    };
}

macro_rules! impl_unsigned_leb128 {
    ($fn_name:ident, $int_ty:ident) => {
        impl_generic_leb128!($fn_name, $int_ty, (|_, _, _| {}));
    };
}

impl_unsigned_leb128!(leb128_u8, u8);
impl_unsigned_leb128!(leb128_u16, u16);
impl_unsigned_leb128!(leb128_u32, u32);
impl_unsigned_leb128!(leb128_u64, u64);
impl_unsigned_leb128!(leb128_u128, u128);
impl_unsigned_leb128!(leb128_usize, usize);

#[inline]
fn sign_extend<T: BitOrAssign + PrimInt + Signed + WrappingNeg>(
    res: &mut T,
    shift: usize,
    byte: u8,
) {
    // leb128_generic skips the last shift update for efficiency on unsigned ints

    if (shift < size_of::<T>() * 8 - 7) && ((byte & 0x40) != 0) {
        // sign extend
        *res |= (T::one() << (shift + 7)).wrapping_neg()
    }
}

macro_rules! impl_signed_leb128 {
    ($fn_name:ident, $int_ty:ident) => {
        impl_generic_leb128!($fn_name, $int_ty, sign_extend);
    };
}

impl_signed_leb128!(leb128_i8, i8);
impl_signed_leb128!(leb128_i16, i16);
impl_signed_leb128!(leb128_i32, i32);
impl_signed_leb128!(leb128_i64, i64);
impl_signed_leb128!(leb128_i128, i128);
impl_signed_leb128!(leb128_isize, isize);

// TODO: tests
#[cfg(any(test, fuzzing))]
mod tests {
    use num_traits::{AsPrimitive, PrimInt, Signed, Unsigned};
    use arrayvec::ArrayVec;
    use super::leb128_size;

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

            let more = !(((n == T::zero()) && ((byte & 0x40) == 0)) || ((n == -T::one()) && ((byte & 0x40) != 0)));
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

    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}

#[cfg(fuzzing)]
pub use tests::{write_signed_leb128, write_unsigned_leb128};
