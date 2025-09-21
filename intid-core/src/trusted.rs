//! Since [`IntegerId`] and [`IntegerIdContiguous`] are safe traits,
//! it is not possible for unsafe code to rely on them for correctness.
//!
//! There is an exception for [`IntegerId::from_int_unchecked`],
//! which is required to accept any value returned from [`IntegerId::to_int`].
//! If [`IntegerIdContiguous`] is implemented,
//! the `from_int_unchecked` function must also accept all values in the range `Self::MIN_ID..=Self::MAX_ID`.
//!
//! These assumptions allow zero-cost conversions from `T::Int` to `T`,
//! but do not affect fully-safe implementations of `IntegerId`
//! where [`IntegerId::from_int_unchecked`] simply delegates to [`IntegerId::from_int`].
//!
//! Because it is a fully safe function in a fully safe trait,
//! there is no way for us to trust its results.
//!
//! In order to work around this, we provide special "trust tokens".
//! These tokens are associated constants on the trait,
//! so that they are always resolved at compile time.
//! These require invoking an `unsafe` code to construct the token,
//! to prevent fully safe code from being able to trigger undefined behavior.
//!
//! [`IntegerIdContiguous`]: crate::IntegerIdContiguous

use core::marker::PhantomData;

use crate::IntegerId;

/// Indicates that an [`IntegerId`] unsafely guarantees that the result of [`IntegerId::to_int`]
/// will always fall in the range `IntegerId::MIN_INT..=IntegerId::MAX_ID`.
///
/// Also guarantees that the [`IntegerId::to_int`] is implemented in the expected manner.
///
/// Just because the type implements something like [`bytemuck::Contiguous`] does not mean
/// that it is valid to create the token, as [`IntegerId::to_int`] could still be implemented incorrectly.
#[derive(Copy, Clone)]
pub struct TrustedRangeToken<T: IntegerId> {
    marker: PhantomData<&'static T>,
}
impl<T: IntegerId> TrustedRangeToken<T> {
    /// Promise that the type `T` satisfies the appropriate correctness requirements.
    ///
    /// # Safety
    /// If the [`IntegerId`] does not meet the requirements,
    /// this is immediate undefined behavior (similar to constructing a `!` type).
    pub const unsafe fn assume_valid() -> Self {
        TrustedRangeToken {
            marker: PhantomData,
        }
    }

    /// Promise that the type `T` satisfies the appropriate correctness whenever `U` promises to.
    ///
    /// This function is helpful for implementing newtype wrappers around an arbitrary inner type.
    ///
    /// This is equivalent to `T::TRUSTED_RANGE::map(|| unsafe { TrustedRangeToken::assume_valid() })`,
    /// but works in a `const` context.
    ///
    /// # Safety
    /// You must ensure that `U` can be trusted with the requirements of [`TrustedRangeToken`]
    /// whenever `U` meets those same requirements.
    pub const unsafe fn assume_valid_if<U: IntegerId>() -> Option<Self> {
        if <U as IntegerId>::TRUSTED_RANGE.is_some() {
            // SAFETY: Caller guarantees that T is trusted whenever U is
            Some(unsafe { TrustedRangeToken::<T>::assume_valid() })
        } else {
            None
        }
    }
}

/*
/// Indicates
pub struct TrustedContiguousToken<T> {

}
impl<T: IntegerId> TrustedContiguousToken<T> {
}
*/
