//! Defines the [`UnsignedPrimInt`] trait and its generic operations.
//!
//! These are module functions rather than trait functions
//! to avoid polluting the primitive integer namespaces.

use core::fmt::{Debug, Display, Formatter};
use core::hash::Hash;

mod sealed;

macro_rules! maybe_trait_bound {
    ($name:ident, cfg($flag:meta), $bound:path) => {
        #[cfg($flag)]
        #[doc(hidden)]
        pub trait $name: $bound {}
        #[cfg(not($flag))]
        #[doc(hidden)]
        pub trait $name {}
        #[cfg($flag)]
        impl<T: $bound> $name for T {}
        #[cfg(not($flag))]
        impl<T> $name for T {}
    };
}

maybe_trait_bound!(
    MaybeNumTrait,
    cfg(feature = "num-traits"),
    num_traits::PrimInt
);
maybe_trait_bound!(MaybePod, cfg(feature = "bytemuck"), bytemuck::Pod);
maybe_trait_bound!(
    MaybeContiguous,
    cfg(feature = "bytemuck"),
    bytemuck::Contiguous
);

/// An unsigned primitive integer.
///
/// Most methods in this trait are only available through the [`intid::uint`](crate::uint) module
/// in order to avoid conflict with inherit implementations and other traits.
/// You can get access to more functionality by enabling the `num-traits` or `bytemuck` features,
/// which will add [`num_traits::PrimInt`] and [`bytemuck::Pod`] bounds respectively.
pub trait UnsignedPrimInt:
    Eq
    + Hash
    + Ord
    + Copy
    + Default
    + Debug
    + Display
    + sealed::PrivateUnsignedInt
    + MaybeNumTrait
    + MaybePod
    + MaybeContiguous
{
}

/// Add the specified value to the integer,
/// returning `None` if overflow occurs.
#[inline]
pub fn checked_add<T: UnsignedPrimInt>(left: T, right: T) -> Option<T> {
    sealed::PrivateUnsignedInt::checked_add(left, right)
}

/// Subtract the specified value from the integer,
/// returning `None` if overflow occurs.
#[inline]
pub fn checked_sub<T: UnsignedPrimInt>(left: T, right: T) -> Option<T> {
    sealed::PrivateUnsignedInt::checked_sub(left, right)
}

/// Convert a primitive integer to a [`usize`],
/// returning `None` if overflow occurs.
#[inline]
pub fn to_usize_checked<T: UnsignedPrimInt>(val: T) -> Option<usize> {
    T::to_usize_checked(val)
}

/// Convert a primitive integer to a [`usize`],
/// wrapping around on overflow.
#[inline]
pub fn to_usize_wrapping<T: UnsignedPrimInt>(val: T) -> usize {
    T::to_usize_wrapping(val)
}

/// Convert a primitive integer to a [`usize`],
/// returning `None` if overflow occurs.
#[inline]
pub fn from_usize_checked<T: UnsignedPrimInt>(val: usize) -> Option<T> {
    T::from_usize_checked(val)
}

/// Convert a primitive integer to a [`usize`],
/// wrapping around if overflow occurs.
#[inline]
pub fn from_usize_wrapping<T: UnsignedPrimInt>(val: usize) -> T {
    T::from_usize_wrapping(val)
}

/// Determine the zero value of the specified `UnsignedPrimInt`.
///
/// This function always succeeds (a `NonZero` is not a primitive integer)
#[inline]
pub const fn zero<T: UnsignedPrimInt>() -> T {
    T::ZERO
}

/// Determine the one value of the specified `UnsignedPrimInt`.
#[inline]
pub const fn one<T: UnsignedPrimInt>() -> T {
    T::ONE
}

/// Determine the maximum value of the specified [`UnsignedPrimInt`].
#[inline]
pub const fn max_value<T: UnsignedPrimInt>() -> T {
    T::MAX
}

/// Attempt to describe the specified [`UnsignedPrimInt`]
/// in a format suitable for debugging or panic messages.
///
/// This differs from the standard `Display` and `Debug` implementation,
/// because `T::MAX` is special-cased.
///
/// *WARNING*: This representation may change without warning in the future,
/// so the exact representation should not be relied upon.
///
/// ## Examples
/// ```
/// use intid::uint::debug_desc;
/// assert_eq!(
///     debug_desc(3u32).to_string(),
///     "3"
/// );
/// assert_eq!(
///     debug_desc(u32::MAX).to_string(),
///     "u32::MAX"
/// )
/// ```
#[cold]
pub fn debug_desc<T: UnsignedPrimInt>(value: T) -> DebugDesc<T> {
    DebugDesc(value)
}

/// The description of an unsigned integer returned by [`debug_desc`].
#[derive(Clone)]
pub struct DebugDesc<T: UnsignedPrimInt>(T);
impl<T: UnsignedPrimInt> Display for DebugDesc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.0 == T::MAX {
            f.write_str(T::TYPE_NAME)?;
            f.write_str("::MAX")
        } else {
            <T as Display>::fmt(&self.0, f)
        }
    }
}
impl<T: UnsignedPrimInt> Debug for DebugDesc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

/// Panic with a message indicating that an ID is not valid.
///
/// Used to implement the panic in [`crate::IntegerId::from_int`].
#[inline(never)]
#[track_caller]
#[cold]
pub(crate) fn invalid_id<T: UnsignedPrimInt>(id: T) -> ! {
    panic!("Invalid id: {}", debug_desc(id))
}
