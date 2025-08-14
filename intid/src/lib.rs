//! Defines the [`IntegerId`] trait, for types that can be identified by an integer value.
#![no_std]

use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::{Hash, Hasher};

mod impls;
pub mod uint;

#[cfg(feature = "derive")]
pub use intid_derive::IntegerId;

pub use uint::UnsignedPrimInt;

/// An identifier which can be sensibly converted to/from an unsigned integer value.
///
///
/// The type should not carry any information beyond that of the integer index,
/// and be able to lossleslly convert back and forth from [`Self::Int`].
/// It is possible that not all values of the underlying integer type are valid,
/// allowing [`core::num::NonZero`] and C-like enums to implement this trait.
///
///
/// This is intended mostly for newtype wrappers around integer indexes,
/// and the primitive integer types themselves.
///
/// The value of the underlying integer must be consistent.
/// It cannot change over the course of the program's lifetime.
///
/// ## Safety
/// With one exception, this trait is safe to implement and cannot be relied upon by memory safety.
///
/// If the implementation of [`IntegerId::from_int_unchecked`] makes any sort of unsafe assumptions
/// about the validity of the input, then the rest of the trait must be implemented correctly.
/// This means that implementations of this trait fall into two categories:
/// 1. Potentially incorrect implemented entirely using safe code, where `from_int_unchecked(x)`
///    is equivalent to calling `from_int_checked(x).unwrap()`;
/// 2. Traits where `from_int_unchecked` could trigger undefined behavior on an invalid value,
///    but every other part of this trait can be trusted to be implemented correctly.
///
/// In both these cases, the following code is always safe:
/// ```no_run
/// # use intid::IntegerId;
/// fn foo<T: IntegerId>(x: T) -> T {
///     let y = x.to_int();
///     let z = unsafe { T::from_int_unchecked(y) };
///     z
/// }
/// ```
/// In case 1,  it doesn't matter if [`x.to_int()`](Self::to_int) produces garbage data,
/// because `T::from_int_unchecked` method is safe to call.
/// In case 2, the `to_int` method can be trusted to produce a valid value `y` that cannot fail
/// when passed to `T::from_int_unchecked`.
///
/// The requirement for correctness in this case also apply to all sub-traits in this crate,
/// including [`ContiguousIntegerId`] and [`IntegerIdCounter`].
/// So an unsafe implementation of `from_int_unchecked` can be similarly trusted to accept
/// all integer values between [`ContiguousIntegerId::MIN_ID`] and [`ContiguousIntegerId::MAX_ID`].
///
/// This restriction allows avoiding unnecessary checks when ids are stored to/from another data structure.
/// Despite this requirement, I still consider this trait safe to implement,
/// because safety can only be violated by an unsafe implementation of`from_int_unchecked`.
///
/// This type should not have interior mutability.
/// This is guaranteed by the `Copy` bound.
pub trait IntegerId: Copy + Eq + Debug + 'static {
    /// The underlying integer type.
    ///
    /// Every valid instance of `Self` should correspond to a valid `Self::Int`.
    /// However, the other direction may not always be true.
    type Int: uint::UnsignedPrimInt;

    /// Create an id from the underlying integer value,
    /// panicking if the value is invalid.
    ///
    /// ## Correctness
    /// A value returned by this method should never trigger
    /// an error if passed to [`Self::from_int_checked`].
    /// This means the validity of certain ids can't change over the course of the program.
    #[inline]
    #[track_caller]
    fn from_int(id: Self::Int) -> Self {
        match Self::from_int_checked(id) {
            Some(success) => success,
            None => uint::invalid_id(id),
        }
    }

    /// Create an id from the underlying integer value,
    /// returning `None` if the value is invalid.
    fn from_int_checked(id: Self::Int) -> Option<Self>;

    /// Create an id from the underlying integer value,
    /// triggering undefined behavior if the value is invalid.
    ///
    /// ## Safety
    /// If the corresponding [`Self::from_int_checked`] method would fail,
    /// this triggers undefined behavior.
    /// The default implementation just invokes [`Self::from_int`].
    #[inline]
    unsafe fn from_int_unchecked(id: Self::Int) -> Self {
        Self::from_int(id)
    }

    /// Convert this id into an underlying integer type.
    ///
    /// This method can never fail,
    /// since valid instances `Self` always correspond to valid instances of `Self::Int`.
    fn to_int(self) -> Self::Int;
}
/// Indicates that an ida occupies contiguous range of contiguous values,
/// between [`Self::MIN_ID`] and [`Self::MAX_ID`] inclusive.
///
/// This is similar to [`bytemuck::Contiguous`].
/// However, since it is safe to implement,
/// it must not be relied upon for correctness.
///
/// ## Safety
/// This trait is safe to implement, so may not usually be relied upon for memory safety.
///
/// However, if [`Self::from_int_unchecked`](IntegerId::from_int_unchecked) makes unsafe assumptions (satisfying the condition set forth in the [`IntegerId`] safety docs),
/// then this trait must also be implemented correctly.
/// More specifically, all integers between [`Self::MIN_ID`] and [`Self::MAX_ID`] must be valid
/// and cannot fail when passed to [`IntegerId::from_int_checked`].
pub trait ContiguousIntegerId: IntegerId {
    /// The value of this type with the smallest integer value.
    const MIN_ID: Self;
    /// The value of this type with the largest integer value.
    const MAX_ID: Self;
}

/// An [`IntegerId`] that can be sensibly used as a counter,
/// starting at a [`Self::START`] value and being incremented from there.
///
/// This is used by the `intid-allocator` crate to provide an atomic counter to allocate new ids.
/// It also provides more complex allocators that can reuse ids that have been freed.
pub trait IntegerIdCounter: IntegerId + ContiguousIntegerId {
    /// Where a counter a should start from.
    ///
    /// This should be the [`Default`] value if one is defined.
    const START: Self;
    /// Where a counter a should start from.
    ///
    /// This should just be the value of [`Self::START`] as a [`T::Int`](IntegerId::Int).
    /// If not, unexpected behavior can occur (but no UB by itself).
    ///
    /// This is necessary because trait methods ([`IntegerId::to_int`])
    /// can not currently be const methods.
    const START_INT: Self::Int;

    /// Increment this value by the specified offset,
    /// returning `None` if the value overflows or is invalid.
    ///
    /// This should behave consistently with [`ContiguousIntegerId`]
    /// and [`IntegerId::from_int_checked`].
    /// However, that can not be relied upon for memory safety.
    ///
    /// This is implemented as an associated method to avoid namespace pollution.
    #[inline]
    fn checked_add(this: Self, offset: Self::Int) -> Option<Self> {
        uint::checked_add(this.to_int(), offset).and_then(Self::from_int_checked)
    }

    /// Increment this value by the specified offset,
    /// returning `None` if the value overflows or is invalid.
    ///
    /// This should behave consistently with [`ContiguousIntegerId`]
    /// and [`IntegerId::from_int_checked`].
    /// However, that can not be relied upon for memory safety.
    ///
    /// This is implemented as an associated method to avoid namespace pollution.
    #[inline]
    fn checked_sub(this: Self, offset: Self::Int) -> Option<Self> {
        uint::checked_sub(this.to_int(), offset).and_then(Self::from_int_checked)
    }
}

/// A wrapper around an [`IntegerId`] which implements [`Eq`], [`Ord`], and [`Hash`]
/// based on the integer value.
#[derive(Copy, Clone, Debug)]
pub struct OrderByInt<T: IntegerId>(pub T);
impl<T: IntegerId> Ord for OrderByInt<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_int().cmp(&other.0.to_int())
    }
}
impl<T: IntegerId> PartialOrd for OrderByInt<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: IntegerId> Eq for OrderByInt<T> {}
impl<T: IntegerId> PartialEq for OrderByInt<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.to_int() == other.0.to_int()
    }
}
impl<T: IntegerId> Hash for OrderByInt<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_int().hash(state)
    }
}

/// A type that can be for lookup as an [`IntegerId`].
///
/// Used for key lookup in maps, similar to [core::borrow::Borrow] or [equivalent::Equivalent].
/// These traits are not suitable for id maps,
/// which need conversion to integers rather than hashing/equality.
///
/// [equivalent::Equivalent]: https://docs.rs/equivalent/latest/equivalent/trait.Equivalent.html
pub trait EquivalentId<K: IntegerId> {
    /// Convert this type to an id `K`.
    fn as_id(&self) -> K;
}
impl<K: IntegerId> EquivalentId<K> for K {
    #[inline]
    fn as_id(&self) -> K {
        *self
    }
}
impl<K: IntegerId> EquivalentId<K> for &'_ K {
    #[inline]
    fn as_id(&self) -> K {
        **self
    }
}
impl<K: IntegerId> EquivalentId<K> for &'_ mut K {
    #[inline]
    fn as_id(&self) -> K {
        **self
    }
}
