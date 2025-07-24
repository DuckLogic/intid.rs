//! Implementations of [`IntegerId`] for foreign types.

macro_rules! impl_primint {
    ($($target:ident),*) => {$(
        impl crate::IntegerId for $target {
            type Int = $target;
            const START: Option<Self> = Some(0);
            #[inline]
            fn from_int_checked(id: Self::Int) -> Option<Self> {
                Some(id)
            }
            #[inline]
            fn to_int(self) -> Self::Int {
                self
            }
        }
    )*};
}
impl_primint!(u8, u16, u32, u64, u128, usize);
// Can't use generic NonZero, because that requires Rust 1.79
macro_rules! impl_nonzero_int {
    ($($target:ident => $int:ident),*) => {$(
        impl crate::IntegerId for core::num::$target {
            type Int = $int;
            const START: Option<Self> = {
                // while using NonZero::MIN might be nice, that requires rust 1.70
                // SAFETY: One is not zero
                unsafe {
                    Some(core::num::$target::new_unchecked(1))
                }
            };

            #[inline]
            fn from_int_checked(id: Self::Int) -> Option<Self> {
                core::num::$target::new(id)
            }

            #[inline]
            unsafe fn from_int_unchecked(id: Self::Int) -> Self {
                // SAFETY: Guaranteed by caller
                unsafe {
                    core::num::$target::new_unchecked(id)
                }
            }

            #[inline]
            fn to_int(self) -> Self::Int {
                self.get()
            }
        }
    )*}
}
impl_nonzero_int!(
    NonZeroU8 => u8,
    NonZeroU16 => u16,
    NonZeroU32 => u32,
    NonZeroU64 => u64,
    NonZeroU128 => u128,
    NonZeroUsize => usize
);

#[cfg(feature = "nonmax")]
macro_rules! do_nonmax_impl {
    ($($target:ident => $int:ident),*) => {$(
        impl crate::IntegerId for nonmax::$target {
            type Int = $int;
            const START: Option<Self> = Some(nonmax::$target::ZERO);
            #[inline]
            fn from_int_checked(id: Self::Int) -> Option<Self> {
                nonmax::$target::new(id)
            }
            #[inline]
            unsafe fn from_int_unchecked(id: Self::Int) -> Self {
                nonmax::$target::new_unchecked(id)
            }
            #[inline]
            fn to_int(self) -> Self::Int {
                self.get()
            }
        }
    )*};
}
#[cfg(feature = "nonmax")]
do_nonmax_impl!(NonMaxU8 => u8, NonMaxU16 => u16, NonMaxU32 => u32, NonMaxU64 => u64, NonMaxU128 => u128, NonMaxUsize => usize);
