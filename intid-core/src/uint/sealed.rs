pub trait PrivateUnsignedInt: Sized {
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    fn checked_cast<U: super::UnsignedPrimInt>(self) -> Option<U>;
    /// The type name as a short unqualified string.
    const TYPE_NAME: &'static str;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn from_usize_checked(val: usize) -> Option<Self>;
    fn from_usize_wrapping(val: usize) -> Self;
    #[allow(clippy::wrong_self_convention)]
    fn to_usize_wrapping(this: Self) -> usize;
    #[allow(clippy::wrong_self_convention)]
    fn to_usize_checked(this: Self) -> Option<usize>;
}
macro_rules! impl_primint {
    ($($target:ident),*) => ($(
        impl super::UnsignedPrimInt for $target {}
        impl super::ConvertPrimInts for $target {}
        impl PrivateUnsignedInt for $target {
            const TYPE_NAME: &'static str = stringify!($target);
            const ZERO: Self = {
                assert!($target::MIN == 0, "signed integer");
                0
            };
            const ONE: Self = 1;
            const MAX: Self = $target::MAX;
            #[inline]
            fn checked_cast<U: super::UnsignedPrimInt>(self) -> Option<U> {
                U::try_from(self).ok()
            }
            #[inline]
            fn checked_add(self, other: Self) -> Option<Self> {
                <$target>::checked_add(self, other)
            }
            #[inline]
            fn checked_sub(self, other: Self) -> Option<Self> {
                <$target>::checked_sub(self, other)
            }
            #[inline]
            fn from_usize_checked(val: usize) -> Option<Self> {
                <$target>::try_from(val).ok()
            }
            #[inline]
            #[allow(clippy::cast_possible_truncation)] // desired functionality
            fn from_usize_wrapping(val: usize) -> Self {
                val as $target
            }
            #[inline]
            #[allow(clippy::cast_possible_truncation)] // desired functionality
            fn to_usize_wrapping(this: Self) -> usize {
                this as usize
            }
            #[inline]
            fn to_usize_checked(this: Self) -> Option<usize> {
                usize::try_from(this).ok()
            }
        }
    )*);
}
impl_primint!(u8, u16, u32, u64, u128, usize);
