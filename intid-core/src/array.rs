//! An implementation of the [`Array`] trait,
//! used as a workaround for the limitations of const generics.

/// A single word in a bitset.
///
/// Currently, this is an alias for [`usize`].
pub type BitsetLimb = usize;

/// A fixed-size builtin array.
///
/// This trait exists only a workaround for the limitations of const generics.
///
/// # Safety
/// This trait is sealed, and is only implemented by builtin arrays of fixed length.
/// Consequently, all items can be trusted to be implemented correctly.
pub trait Array<T>: Sized + AsRef<[T]> + AsMut<[T]> + sealed::Sealed {
    /// The length of this array.
    const LEN: usize;
}

impl<T, const LEN: usize> Array<T> for [T; LEN] {
    const LEN: usize = LEN;
}
impl<T, const LEN: usize> sealed::Sealed for [T; LEN] {}

mod sealed {
    pub trait Sealed {}
}
