//! Implementations of [`IntegerId`](crate::IntegerId) for foreign types.

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
    (@main_body $target:path, $int:ident, one => $one:expr, max => $max:expr) => {
        type Int = $int;
        const MIN_ID: Option<Self> = {
            // while using NonZero::MIN might be nice, that requires rust 1.70
            // SAFETY: One is not zero
            unsafe {
                Some(<$target>::new_unchecked($one))
            }
        };
        const MAX_ID: Option<Self> = {
            // SAFETY: Maximum is not zero
            unsafe {
                Some(<$target>::new_unchecked($max))
            }
        };
        const MIN_ID_INT: Option<Self::Int> = Some($one);
        const MAX_ID_INT: Option<Self::Int> = Some($max);
        // SAFETY: Range is correct
        const TRUSTED_RANGE: Option<crate::trusted::TrustedRangeToken<Self>> = unsafe { Some(crate::trusted::TrustedRangeToken::assume_valid()) };

        #[inline]
        fn from_int_checked(id: Self::Int) -> Option<Self> {
            <$target>::new(id)
        }

        #[inline]
        unsafe fn from_int_unchecked(id: Self::Int) -> Self {
            // SAFETY: Guaranteed by caller
            unsafe {
                <$target>::new_unchecked(id)
            }
        }

        #[inline]
        fn to_int(self) -> Self::Int {
            self.get()
        }
    };
    (@counter_body $target:path, $int:ident) => {
        const START: Self = match <Self as crate::IntegerId>::MIN_ID {
            Some(valid) => valid,
            None => panic!("type should be inhabited")
        };
        const START_INT: $int = Self::START.get();
    };
    (@stdlib $($target:ident => $int:ident),*) => {$(
        impl crate::IntegerId for core::num::$target {
            impl_nonzero_int!(@main_body core::num::$target, $int, one => 1, max => $int::MAX);
        }
        impl crate::IntegerIdContiguous for core::num::$target {}
        impl crate::IntegerIdCounter for core::num::$target {
            impl_nonzero_int!(@counter_body core::num::$target, $int);
        }
    )*}
}
impl_nonzero_int!(
    @stdlib
    NonZeroU8 => u8,
    NonZeroU16 => u16,
    NonZeroU32 => u32,
    NonZeroU64 => u64,
    NonZeroU128 => u128,
    NonZeroUsize => usize
);
#[cfg(feature = "primint-nonzero")]
mod primint_nonzero {
    use primint::UnsignedPrimInt;

    impl<T: UnsignedPrimInt> crate::IntegerId for primint::num::NonZero<T> {
        impl_nonzero_int!(@main_body primint::num::NonZero<T>, T, one => primint::one(), max => primint::max_value());
    }
    impl<T: UnsignedPrimInt> crate::IntegerIdContiguous for primint::num::NonZero<T> {}
    impl<T: UnsignedPrimInt> crate::IntegerIdCounter for primint::num::NonZero<T> {
        impl_nonzero_int!(@counter_body primint::num::NonZero<T>, T);
    }
}

#[cfg(any(feature = "nonmax", feature = "primint-nonmax"))]
macro_rules! do_nonmax_impl {
    (@main_body $target:path, $int:ident) => {
        type Int = $int;
        const MIN_ID: Option<Self> = Some({
            assert!(primint::is_signed::<$int>());
            // SAFETY: Zero is never the maximum value
            unsafe { <$target>::new_unchecked(primint::zero()) }
        });
        const MAX_ID: Option<Self> = Some(<$target>::MAX);
        const MIN_ID_INT: Option<Self::Int> = Some(primint::zero());
        const MAX_ID_INT: Option<Self::Int> = Some(<$target>::MAX.get());
        // SAFETY: Range is correct
        const TRUSTED_RANGE: Option<crate::trusted::TrustedRangeToken<Self>> = unsafe { Some(crate::trusted::TrustedRangeToken::assume_valid()) };

        #[inline]
        fn from_int_checked(id: Self::Int) -> Option<Self> {
            <$target>::new(id)
        }
        #[inline]
        unsafe fn from_int_unchecked(id: Self::Int) -> Self {
            // SAFETY: Guaranteed by caller
            unsafe { <$target>::new_unchecked(id) }
        }
        #[inline]
        fn to_int(self) -> Self::Int {
            self.get()
        }
    };
    (@nonmax_crate $($target:ident => $int:ident),*) => {$(
        impl crate::IntegerId for nonmax::$target {
            do_nonmax_impl!(@main_body nonmax::$target, $int);
        }
        impl crate::IntegerIdContiguous for nonmax::$target {}
        impl crate::IntegerIdCounter for nonmax::$target {
            const START: Self = nonmax::$target::ZERO;
            const START_INT: Self::Int = 0;
        }
    )*};
}
#[cfg(feature = "nonmax")]
do_nonmax_impl!(
    @nonmax_crate
    NonMaxU8 => u8,
    NonMaxU16 => u16,
    NonMaxU32 => u32,
    NonMaxU64 => u64,
    NonMaxU128 => u128,
    NonMaxUsize => usize
);

#[cfg(feature = "primint-nonmax")]
mod primint_nonmax {
    use primint::UnsignedPrimInt;

    impl<T: UnsignedPrimInt> crate::IntegerId for primint::num::NonMax<T> {
        do_nonmax_impl!(@main_body primint::num::NonMax<T>, T);
    }
    impl<T: UnsignedPrimInt> crate::IntegerIdContiguous for primint::num::NonMax<T> {}
    impl<T: UnsignedPrimInt> crate::IntegerIdCounter for primint::num::NonMax<T> {
        const START: Self = primint::num::NonMax::<T>::MIN;
        const START_INT: T = primint::zero();
    }
}

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
