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
const fn _leb128_size<T>() -> usize {
    let bits = size_of::<T>() * 8;
    (bits + 6) / 7 // equivalent to ceil(bits/7) w/o floats
}

#[cfg(fuzzing)]
pub const fn leb128_size<T>() -> usize {
    _leb128_size::<T>()
}

macro_rules! impl_generic_leb128 {
    ($fn_name:ident, $int_ty:ident, $post:tt) => {
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
                } else if pos == _leb128_size::<$int_ty>() - 1 {
                    return Err(nom::Err::Error(E::add_context(
                        input.clone(),
                        concat!("LEB128 integer is too big to fit in ", stringify!($int_ty)),
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
#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
