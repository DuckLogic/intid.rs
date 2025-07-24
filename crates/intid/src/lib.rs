//! Defines the [`IntegerId`] trait, for types that can be identified by an integer value.
#![no_std]

use core::fmt::Debug;

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

    /// The initial value for this id.
    ///
    /// This should be the smallest integer `x` such that [`Self::from_int(x)`](Self::from_int) succeeds,
    /// or `None` if there is no reasonable default for this type.
    const START: Option<Self>;

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

/// An identifier with a restricted range of integer values,
/// such that all valid ids fall beneath a reasonably small upper bound.
///
/// This trait is intended primarily for C-like enums where it is reasonable
/// to implement a `Map<K, V>` via a fixed-size array `[V; { K::UPPER_BOUND + 1 }]`
/// and a set as a fixed-size bitset as `[u32; { (K::UPPER_BOUND / u32::BITS) + 1]``
/// For integers larger than a `u32`, this is not reasonable.
pub trait BoundedIntegerId: IntegerId {
    /// The upper bound of the type, past which there are no valid ids.
    ///
    /// ## Safety
    /// In general, this value can not be relied upon for correctness.
    const UPPER_BOUND: usize;
}
