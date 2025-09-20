use core::iter::StepBy;
use core::num::NonZero;
use crate::IntegerIdContiguous;

pub fn contiguous<T: IntegerIdContiguous>() -> IterContiguous<T> {
    IterContiguous {
        next: T::MIN_ID_INT,
    }
}

/// Indicates that the result of [`IterContiguous::len`] overflowed a [`u64`].
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct IterLengthOverflowError;

pub struct IterContiguous<T: IntegerIdContiguous> {
    /// The next value to be returned from the iterator.
    ///
    /// Invariants:
    /// - When not `None`, `T::MIN_ID_INT <= next.to_int <= T::MAX_ID_INT`
    next: Option<T>,
}
impl<T: IntegerIdContiguous> IterContiguous<T> {
    pub fn len(&self) -> Result<u64, IterLengthOverflowError> {
        match self.next {
            None => Ok(0),
            Some(current) => {
                // Cannot overflow because Some(next) <= T::MAX_ID
                //
                // We can make this addition unchecked only if we trust the range
                let delta = if T::TRUSTED_RANGE.is_some() {
                    // SAFETY: We trust the range and our own invariants
                    unsafe {
                        crate::uint::unchecked_sub(
                            T::MAX_ID_INT.unwrap(),
                            current.to_int()
                        )
                    }
                } else {
                    T::MAX_ID_INT.unwrap() - current.to_int()
                };
                u64::try_from(delta).ok_or(IterLengthOverflowError)
            }
        }
    }
}

impl<T: IntegerIdContiguous> core::iter::FusedIterator for IterContiguous<T> {}
impl<T: IntegerIdContiguous> Iterator for IterContiguous<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {

    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        todo!()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        todo!()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.next.unwrap().to_int();
    }
}
impl<T: IntegerIdContiguous> ExactSizeIterator for IterContiguous<T> where T::Int: SmallerThanUsize {}

/// Implemented for integer types smaller than a [`usize`].
///
/// Not implemented for `u32` on 64-bit platforms because that would be a portability hazard.
/// It is implemented for `u16` on 32-bit/64-bit platforms,
/// because supporting 16-bit platforms is rare in modern codebases.
trait SmallerThanUsize {}
#[cfg(not(target_pointer_width = "16"))]
impl SmallerThanUsize for u16 {}
impl SmallerThanUsize for u8 {}
