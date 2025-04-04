use core::mem;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Range, Shl, Shr};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bitfield<T, const N: usize> {
    pub arr: [T; N],
}

impl<T, const N: usize> Bitfield<T, N>
where
    T: BitfieldPrimitive,
{
    pub fn from_arr(arr: [T; N]) -> Self {
        Self { arr }
    }

    pub fn into_arr(self) -> [T; N] {
        self.arr
    }

    pub fn as_arr(&self) -> &[T; N] {
        &self.arr
    }

    pub fn as_mut_arr(&mut self) -> &mut [T; N] {
        &mut self.arr
    }

    pub fn zeroed() -> Self {
        Self::from_arr([T::zero(); N])
    }

    pub fn get_bits(&self, range: Range<usize>) -> T {
        get_bits(&self.arr, range)
    }

    pub fn set_bits(&mut self, range: Range<usize>, bits: T) {
        set_bits(&mut self.arr, range, bits)
    }
}

pub trait BitfieldPrimitive:
    BitfieldPrimitiveSealed
    + Copy
    + Eq
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitOrAssign
    + BitAndAssign
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + From<bool> // HACK for generic 0
{
}

trait BitfieldPrimitiveExt: BitfieldPrimitive {
    const NUM_BITS: usize = mem::size_of::<Self>() * 8;

    fn zero() -> Self {
        false.into()
    }

    fn mask(range: Range<usize>) -> Self {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= Self::NUM_BITS);
        let num_bits = range.end - range.start;
        // avoid overflow
        match num_bits {
            0 => Self::zero(),
            _ if num_bits == Self::NUM_BITS => !Self::zero(),
            _ => !(!Self::zero() << num_bits) << range.start,
        }
    }

    fn take(self, num_bits: usize) -> Self {
        self & Self::mask(0..num_bits)
    }
}

impl<T: BitfieldPrimitive> BitfieldPrimitiveExt for T {}

impl BitfieldPrimitive for u128 {}
impl BitfieldPrimitive for u64 {}
impl BitfieldPrimitive for u32 {}
impl BitfieldPrimitive for u16 {}
impl BitfieldPrimitive for u8 {}

use bitfield_primative_sealing::BitfieldPrimitiveSealed;

mod bitfield_primative_sealing {
    pub trait BitfieldPrimitiveSealed {}

    impl BitfieldPrimitiveSealed for u128 {}
    impl BitfieldPrimitiveSealed for u64 {}
    impl BitfieldPrimitiveSealed for u32 {}
    impl BitfieldPrimitiveSealed for u16 {}
    impl BitfieldPrimitiveSealed for u8 {}
}

pub trait BitfieldPrimitiveMaybeSigned: BitfieldPrimitiveMaybeSignedSealed {
    type Unsigned: BitfieldPrimitive;

    fn cast_from_unsigned(val: Self::Unsigned) -> Self;
    fn cast_to_unsigned(val: Self) -> Self::Unsigned;
}

impl<T> BitfieldPrimitiveMaybeSigned for T
where
    T: BitfieldPrimitive + BitfieldPrimitiveMaybeSignedSealed,
{
    type Unsigned = Self;

    fn cast_from_unsigned(val: Self::Unsigned) -> Self {
        val
    }

    fn cast_to_unsigned(val: Self) -> Self::Unsigned {
        val
    }
}

macro_rules! impl_bitfield_primitive_maybe_unsigned {
    ($maybe_signed:ty, $unsigned:ty) => {
        impl BitfieldPrimitiveMaybeSigned for $maybe_signed {
            type Unsigned = $unsigned;

            fn cast_from_unsigned(val: Self::Unsigned) -> Self {
                val as Self
            }

            fn cast_to_unsigned(val: Self) -> Self::Unsigned {
                val as Self::Unsigned
            }
        }
    };
}

impl_bitfield_primitive_maybe_unsigned!(i128, u128);
impl_bitfield_primitive_maybe_unsigned!(i64, u64);
impl_bitfield_primitive_maybe_unsigned!(i32, u32);
impl_bitfield_primitive_maybe_unsigned!(i16, u16);
impl_bitfield_primitive_maybe_unsigned!(i8, u8);

use bitfield_primative_maybe_signed_sealing::BitfieldPrimitiveMaybeSignedSealed;

mod bitfield_primative_maybe_signed_sealing {
    use super::BitfieldPrimitiveSealed;

    pub trait BitfieldPrimitiveMaybeSignedSealed {}

    impl<T: BitfieldPrimitiveSealed> BitfieldPrimitiveMaybeSignedSealed for T {}

    impl BitfieldPrimitiveMaybeSignedSealed for i128 {}
    impl BitfieldPrimitiveMaybeSignedSealed for i64 {}
    impl BitfieldPrimitiveMaybeSignedSealed for i32 {}
    impl BitfieldPrimitiveMaybeSignedSealed for i16 {}
    impl BitfieldPrimitiveMaybeSignedSealed for i8 {}
}

//

pub fn get_bits<T: BitfieldPrimitive, const N: usize, U: BitfieldPrimitive + TryFrom<T>>(
    arr: &[T; N],
    range: Range<usize>,
) -> U {
    check_range::<T, N, U>(&range);

    let num_bits = range.end - range.start;
    let index_of_first_primitive = range.start / T::NUM_BITS;
    let offset_into_first_primitive = range.start % T::NUM_BITS;
    let num_bits_from_first_primitive = (T::NUM_BITS - offset_into_first_primitive).min(num_bits);

    let bits_from_first_primitive = (arr[index_of_first_primitive] >> offset_into_first_primitive)
        .take(num_bits_from_first_primitive);

    let mut bits = checked_cast::<T, U>(bits_from_first_primitive);
    let mut num_bits_so_far = num_bits_from_first_primitive;
    let mut index_of_cur_primitive = index_of_first_primitive + 1;

    while num_bits_so_far < num_bits {
        let num_bits_from_cur_primitive = (num_bits - num_bits_so_far).min(T::NUM_BITS);
        let bits_from_cur_primitive = arr[index_of_cur_primitive].take(num_bits_from_cur_primitive);
        bits |= checked_cast::<T, U>(bits_from_cur_primitive) << num_bits_so_far;
        num_bits_so_far += num_bits_from_cur_primitive;
        index_of_cur_primitive += 1;
    }

    bits
}

pub fn get_bits_maybe_signed<
    T: BitfieldPrimitive,
    const N: usize,
    U: BitfieldPrimitiveMaybeSigned,
>(
    arr: &[T; N],
    range: Range<usize>,
) -> U
where
    U::Unsigned: TryFrom<T>,
{
    U::cast_from_unsigned(get_bits(arr, range))
}

pub fn set_bits<T: BitfieldPrimitive, const N: usize, U: BitfieldPrimitive + TryInto<T>>(
    arr: &mut [T; N],
    range: Range<usize>,
    bits: U,
) {
    check_range::<T, N, U>(&range);

    let num_bits = range.end - range.start;

    // assert!(num_bits == U::NUM_BITS || bits >> num_bits == U::zero());

    let index_of_first_primitive = range.start / T::NUM_BITS;
    let offset_into_first_primitive = range.start % T::NUM_BITS;
    let num_bits_for_first_primitive = (T::NUM_BITS - offset_into_first_primitive).min(num_bits);
    let bits_for_first_primitive = bits.take(num_bits_for_first_primitive);

    arr[index_of_first_primitive] = (arr[index_of_first_primitive]
        & !T::mask(
            offset_into_first_primitive
                ..(offset_into_first_primitive + num_bits_for_first_primitive),
        ))
        | checked_cast(bits_for_first_primitive) << offset_into_first_primitive;

    let mut num_bits_so_far = num_bits_for_first_primitive;
    let mut index_of_cur_primitive = index_of_first_primitive + 1;

    while num_bits_so_far < num_bits {
        let num_bits_for_cur_primitive = (num_bits - num_bits_so_far).min(T::NUM_BITS);
        let bits_for_cur_primitive = (bits >> num_bits_so_far).take(num_bits_for_cur_primitive);
        arr[index_of_cur_primitive] = (arr[index_of_cur_primitive]
            & T::mask(num_bits_for_cur_primitive..T::NUM_BITS))
            | checked_cast(bits_for_cur_primitive);
        num_bits_so_far += num_bits_for_cur_primitive;
        index_of_cur_primitive += 1;
    }
}

pub fn set_bits_maybe_signed<
    T: BitfieldPrimitive,
    const N: usize,
    U: BitfieldPrimitiveMaybeSigned,
>(
    arr: &mut [T; N],
    range: Range<usize>,
    bits: U,
) where
    U::Unsigned: TryInto<T>,
{
    set_bits(arr, range, U::cast_to_unsigned(bits))
}

fn check_range<T: BitfieldPrimitive, const N: usize, U: BitfieldPrimitive>(range: &Range<usize>) {
    assert!(range.start <= range.end);
    assert!(range.end <= N * T::NUM_BITS);
    assert!(range.end - range.start <= U::NUM_BITS);
}

fn checked_cast<T: TryInto<U>, U>(val: T) -> U {
    val.try_into().map_err(|_| unreachable!()).unwrap()
}
