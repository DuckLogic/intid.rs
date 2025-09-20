//! Defines the [`IntegerId`] trait, for types that can be identified by an integer value.
//!
//! This contains all the same types that the [`intid`] crate does,
//! but has no dependency on [`intid_derive`] (even when the `intid/derive` feature is enabled).
//! This reduces compile times, similar to the separation between `serde_core` and `serde` introduced in [serde-rs/serde#2608].
//!
//! It may be convenient to rename the `intid_core` dependency to `intid` using [dependency renaming].
//! ```toml
//! intid = { version = "0.3", package = "intid_core" }
//! ```
//! This renaming comes at no loss of clarity,
//! since the items in `intid_core` are simply a subset of the items in the `intid` crate.
//! If for some reason you decide to use `intid_derive` directly without depending on `intid`,
//! then you will need to do this renaming since the derived code references the `intid` crate.
//!
//! [dependency renaming]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml
//! [serde-rs/serde#2608]: https://github.com/serde-rs/serde/pull/2608
//! [`intid`]: https://docs.rs/intid/latest/intid
//! [`intid_derive`]: https://docs.rs/intid-derive/latest/intid_derive
#![no_std]
#![cfg_attr(feature = "nightly", feature(never_type,))]

use core::fmt::Debug;

#[macro_use]
mod macros;
mod impls;
pub mod trusted;
pub mod uint;
pub mod utils;

pub use uint::UnsignedPrimInt;

/// An identifier which can be sensibly converted to/from an unsigned integer value.
///
///
/// The type should not carry any information beyond that of the integer index,
/// and be able to losslessly convert back and forth from [`Self::Int`].
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
/// # use intid_core::IntegerId;
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
/// including [`IntegerIdContiguous`] and [`IntegerIdCounter`].
/// So an unsafe implementation of `from_int_unchecked` can be similarly trusted to accept
/// all integer values between [`IntegerId::MIN_ID`] and [`IntegerId::MAX_ID`].
///
/// This restriction allows avoiding unnecessary checks when ids are stored to/from another data structure.
/// Despite this requirement, I still consider this trait safe to implement,
/// because safety can only be violated by an unsafe implementation of`from_int_unchecked`.
///
/// This type should not have interior mutability.
/// This is guaranteed by the `Copy` bound.
pub trait IntegerId: Copy + Eq + Debug + Send + Sync + 'static {
    /// The underlying integer type.
    ///
    /// Every valid instance of `Self` should correspond to a valid `Self::Int`.
    /// However, the other direction may not always be true.
    type Int: uint::UnsignedPrimInt;
    /// The value of this type with the smallest integer value,
    /// or `None` if this type is uninhabited.
    const MIN_ID: Option<Self>;
    /// The value of this type with the largest integer value,
    /// or `None` if this type is uninhabited.
    const MAX_ID: Option<Self>;
    /// The value of [`Self::MIN_ID`] a primitive integer,
    /// or `None` if this type is uninhabited.
    ///
    /// This is necessary because trait methods cannot be marked `const`.
    const MIN_ID_INT: Option<Self::Int>;
    /// The value of [`Self::MAX_ID`] a primitive integer,
    /// or `None` if this type is uninhabited.
    ///
    /// This is necessary because trait methods cannot be marked `const`.
    const MAX_ID_INT: Option<Self::Int>;

    /// Indicates that the type's implementation of [`IntegerId::to_int`] can trusted
    /// to only return values in the range `MIN_ID_INT..=MAX_ID_INT`.
    ///
    /// This can be relied upon by unsafe code, since the token is `unsafe` to construct.
    const TRUSTED_RANGE: Option<trusted::TrustedRangeToken<Self>> = None;

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

/// Indicates that an id occupies contiguous range of contiguous values,
/// and all values between [`IntegerId::MIN_ID`] and [`IntegerId::MAX_ID`] are valid.
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
/// More specifically, all integers between [`IntegerId::MIN_ID`] and [`IntegerId::MAX_ID`] must be valid
/// and cannot fail when passed to [`IntegerId::from_int_checked`].
pub trait IntegerIdContiguous: IntegerId {}

/// An [`IntegerId`] that can be sensibly used as a counter,
/// starting at a [`Self::START`] value and being incremented from there.
///
/// This is used by the `intid-allocator` crate to provide an atomic counter to allocate new ids.
/// It also provides more complex allocators that can reuse ids that have been freed.
///
/// This type cannot be implemented for uninhabited types like [`core::convert::Infallible`] or `!`,
/// as there is no valid implementation of [`Self::START`].
pub trait IntegerIdCounter: IntegerId + IntegerIdContiguous {
    /// Where a counter a should start from.
    ///
    /// This should be the [`Default`] value if one is defined.
    /// It is usually equal to the [`IntegerId::MIN_ID`],
    /// but this is not required.
    const START: Self;
    /// The value of [`Self::START`] as a [`T::Int`](IntegerId::Int).
    ///
    /// This is necessary because trait methods ([`IntegerId::to_int`])
    /// can not currently be const methods.
    const START_INT: Self::Int;

    /// Increment this value by the specified offset,
    /// returning `None` if the value overflows or is invalid.
    ///
    /// This should behave consistently with [`IntegerIdContiguous`]
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
    /// This should behave consistently with [`IntegerIdContiguous`]
    /// and [`IntegerId::from_int_checked`].
    /// However, that can not be relied upon for memory safety.
    ///
    /// This is implemented as an associated method to avoid namespace pollution.
    #[inline]
    fn checked_sub(this: Self, offset: Self::Int) -> Option<Self> {
        uint::checked_sub(this.to_int(), offset).and_then(Self::from_int_checked)
    }
}

/// A type that can be for lookup as an [`IntegerId`].
///
/// Used for key lookup in maps, similar to [`core::borrow::Borrow`] or [`equivalent::Equivalent`].
/// These traits are not suitable for id maps,
/// which need conversion to integers rather than hashing/equality.
///
/// [`equivalent::Equivalent`]: https://docs.rs/equivalent/latest/equivalent/trait.Equivalent.html
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
