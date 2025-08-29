use crate::IdExhaustedError;
use core::cell::Cell;
use intid::{uint, IntegerIdCounter};

#[cfg(feature = "atomic")]
pub mod atomic;

/// Allocates unique integer ids.
///
/// Guarantees that each call to the [`Self::alloc`] function will return a unique id,
/// unless [`Self::reset`] is called.
///
/// Ids start at [`IntegerIdCounter::START`] by default, counting upwards from there.
#[derive(Clone, Debug)]
pub struct UniqueIdAllocator<T: IntegerIdCounter> {
    next_id: Cell<Option<T>>,
}
impl<T: IntegerIdCounter> Default for UniqueIdAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: IntegerIdCounter> UniqueIdAllocator<T> {
    /// Return the maximum currently used id,
    /// or `None` if no ids have been allocated yet.
    #[inline]
    pub fn max_used_id(&self) -> Option<T> {
        self.next_id
            .get()
            .and_then(|id| IntegerIdCounter::checked_sub(id, uint::one()))
    }

    /// Create a new allocator,
    /// using [`T::START`] as the first id (usually zero).
    ///
    /// [`T::START`]: IntegerIdCounter::START
    #[inline]
    pub const fn new() -> Self {
        Self::with_start(T::START)
    }

    /// Create a new allocator,
    /// using the specified value as the first id.
    ///
    /// Equivalent to calling [`Self::new`] then [`Self::set_next_id`] in sequence.
    #[inline]
    pub const fn with_start(start: T) -> Self {
        UniqueIdAllocator {
            next_id: Cell::new(Some(start)),
        }
    }

    /// Attempt to allocate a new id,
    /// panicking if none are available.
    ///
    /// See [`Self::try_alloc`] for a version that returns an error
    #[inline]
    #[track_caller]
    pub fn alloc(&self) -> T {
        match self.try_alloc() {
            Ok(id) => id,
            Err(e) => e.panic(),
        }
    }

    /// Attempt to allocate a new id,
    /// returning an error if there are no more available.
    #[inline]
    pub fn try_alloc(&self) -> Result<T, IdExhaustedError<T>> {
        let old_id = self.next_id.get().ok_or_else(IdExhaustedError::new)?;
        self.next_id
            .set(IntegerIdCounter::checked_add(old_id, intid::uint::one()));
        Ok(old_id)
    }

    /// Set the id that will be returned from the [`Self::alloc`] function.
    ///
    /// Like a call to [`Self::reset`], this may cause the counter to unexpectedly jump backwards.
    /// It may also cause the counter to jump unexpectedly forwards.
    /// Keep the allocator private if this behavior is undesired.
    #[inline]
    pub fn set_next_id(&self, next_id: T) {
        self.next_id.set(Some(next_id))
    }

    /// Reset the allocator to a pristine state,
    /// beginning allocations all over again.
    ///
    /// This is equivalent to running `*allocator = UniqueIdAllocator::new()`,
    /// but does not require a `&mut Self` reference.
    ///
    /// This may cause unexpected behavior if ids are expected to be monotonically increasing,
    /// or if the new ids conflict with ones still in use.
    /// To avoid this, keep the id allocator private.
    ///
    /// See also the [`Self::set_next_id`] function,
    /// which can cause the counter to jump forwards in addition to jumping backwards.
    #[inline]
    pub fn reset(&self) {
        self.set_next_id(T::START);
    }
}
