//! An implementation of the [`Array`] trait,
//! used as a workaround for the limitations of const generics.

use core::iter::FusedIterator;

/// A single word in a bitset.
///
/// Currently, this is an alias for [`u64`].
/// It needs to be fixed-size for the derive macro to work correctly.
pub type BitsetLimb = u64;

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
    fn perform_clone(&self) -> Self
    where
        T: Clone;
    type Iter: ArrayIntoIter<T>;
    fn into_iter(self) -> Self::Iter;
}

pub trait ArrayIntoIter<T>:
    Sized
    + Iterator<Item = T>
    + DoubleEndedIterator
    + ExactSizeIterator
    + FusedIterator
    + sealed::Sealed
{
    fn perform_clone(&self) -> Self
    where
        T: Clone;
}
impl<T, const LEN: usize> ArrayIntoIter<T> for core::array::IntoIter<T, LEN> {
    #[inline]
    fn perform_clone(&self) -> core::array::IntoIter<T, LEN>
    where
        T: Clone,
    {
        self.clone()
    }
}
impl<T, const LEN: usize> sealed::Sealed for core::array::IntoIter<T, LEN> {}

impl<T, const LEN: usize> Array<T> for [T; LEN] {
    const LEN: usize = LEN;
    #[inline]
    fn perform_clone(&self) -> Self
    where
        T: Clone,
    {
        self.clone()
    }
    type Iter = core::array::IntoIter<T, LEN>;

    #[inline]
    fn into_iter(self) -> Self::Iter {
        <Self as IntoIterator>::into_iter(self)
    }
}
impl<T, const LEN: usize> sealed::Sealed for [T; LEN] {}

mod sealed {
    pub trait Sealed {}
}
