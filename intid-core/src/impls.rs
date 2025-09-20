//! Implementations of [`IntegerId`] for foreign types.

macro_rules! impl_primint {
    ($($target:ident),*) => {$(
        impl crate::IntegerId for $target {
            type Int = $target;
            const MIN_ID: Option<Self> = Some(0);
            const MAX_ID: Option<Self> = Some($target::MAX);
            const MIN_ID_INT: Option<Self::Int> = Some(0);
            const MAX_ID_INT: Option<Self::Int> = Some($target::MAX);
            // SAFETY: Range is correct
            const TRUSTED_RANGE: Option<crate::trusted::TrustedRangeToken<Self>> = unsafe { Some(crate::trusted::TrustedRangeToken::assume_valid()) };
            #[inline]
            fn from_int_checked(id: Self::Int) -> Option<Self> {
                Some(id)
            }
            #[inline]
            fn to_int(self) -> Self::Int {
                self
            }
        }
        impl crate::IntegerIdContiguous for $target {}
        impl crate::IntegerIdCounter for $target {
            const START: Self = 0;
            const START_INT: Self = 0;
        }
    )*};
}
impl_primint!(u8, u16, u32, u64, u128, usize);
// Can't use generic NonZero, because that requires Rust 1.79
macro_rules! impl_nonzero_int {
    ($($target:ident => $int:ident),*) => {$(
        impl crate::IntegerId for core::num::$target {
            type Int = $int;
            const MIN_ID: Option<Self> = {
                // while using NonZero::MIN might be nice, that requires rust 1.70
                // SAFETY: One is not zero
                unsafe {
                    Some(core::num::$target::new_unchecked(1))
                }
            };
            const MAX_ID: Option<Self> = {
                // SAFETY: Maximum is not zero
                unsafe {
                    Some(core::num::$target::new_unchecked($int::MAX))
                }
            };
            const MIN_ID_INT: Option<Self::Int> = Some(1);
            const MAX_ID_INT: Option<Self::Int> = Some($int::MAX);
            // SAFETY: Range is correct
            const TRUSTED_RANGE: Option<crate::trusted::TrustedRangeToken<Self>> = unsafe { Some(crate::trusted::TrustedRangeToken::assume_valid()) };

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
        impl crate::IntegerIdContiguous for core::num::$target {}
        impl crate::IntegerIdCounter for core::num::$target {
            const START: Self = match <Self as crate::IntegerId>::MIN_ID {
                Some(valid) => valid,
                None => panic!("type should be inhabited")
            };
            const START_INT: $int = Self::START.get();
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
            const MIN_ID: Option<Self> = Some(nonmax::$target::ZERO);
            const MAX_ID: Option<Self> = Some(nonmax::$target::MAX);
            const MIN_ID_INT: Option<Self::Int> = Some(0);
            const MAX_ID_INT: Option<Self::Int> = Some(nonmax::$target::MAX.get());
            // SAFETY: Range is correct
            const TRUSTED_RANGE: Option<crate::trusted::TrustedRangeToken<Self>> = unsafe { Some(crate::trusted::TrustedRangeToken::assume_valid()) };

            #[inline]
            fn from_int_checked(id: Self::Int) -> Option<Self> {
                nonmax::$target::new(id)
            }
            #[inline]
            unsafe fn from_int_unchecked(id: Self::Int) -> Self {
                // SAFETY: Guaranteed by caller
                unsafe { nonmax::$target::new_unchecked(id) }
            }
            #[inline]
            fn to_int(self) -> Self::Int {
                self.get()
            }
        }
        impl crate::IntegerIdContiguous for nonmax::$target {
        }
        impl crate::IntegerIdCounter for nonmax::$target {
            const START: Self = nonmax::$target::ZERO;
            const START_INT: Self::Int = 0;
        }
    )*};
}
#[cfg(feature = "nonmax")]
do_nonmax_impl!(NonMaxU8 => u8, NonMaxU16 => u16, NonMaxU32 => u32, NonMaxU64 => u64, NonMaxU128 => u128, NonMaxUsize => usize);

macro_rules! impl_uninhabited {
    ($target:ty) => {
        impl crate::IntegerId for $target {
            type Int = u8;
            const MIN_ID: Option<Self> = None;
            const MAX_ID: Option<Self> = None;
            const MIN_ID_INT: Option<Self::Int> = None;
            const MAX_ID_INT: Option<Self::Int> = None;
            const TRUSTED_RANGE: Option<crate::trusted::TrustedRangeToken<Self>> = {
                // SAFETY: Range is correct (vacuously)
                unsafe { Some(crate::trusted::TrustedRangeToken::assume_valid()) }
            };

            #[track_caller]
            #[inline]
            fn from_int(id: Self::Int) -> Self {
                panic!(
                    "Cannot initialize uninhabited type {this} with {id}",
                    this = stringify!($target),
                )
            }

            #[inline]
            fn from_int_checked(_id: Self::Int) -> Option<Self> {
                None
            }

            #[inline]
            unsafe fn from_int_unchecked(_id: Self::Int) -> Self {
                // SAFETY: Caller guarantees this is called only if `id` is a valid index,
                // and there are no valid indices
                unsafe {
                    core::hint::unreachable_unchecked();
                }
            }

            #[inline]
            fn to_int(self) -> Self::Int {
                match self {}
            }
        }
        impl crate::IntegerIdContiguous for $target {}
    };
}
impl_uninhabited!(core::convert::Infallible);
#[cfg(feature = "nightly")]
impl_uninhabited!(!);
