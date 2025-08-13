use crate::{IdExhaustedError, UniqueIdAllocator};
use intid::IntegerIdCounter;

/// A type that allocates integer ids,
/// with the ability to free unused ids back to storage.
///
/// This will minimize the integer value of the keys,
/// reducing memory needed for lookup tables and bitsets.
/// It is useful in conjunction with the "direct" maps/sets of the [idmap crate][idmap].
///
/// If the ability to free unused ids is not necessary,
/// consider [`crate::UniqueIdAllocator`] or [`crate::UniqueIdAllocatorAtomic`].
/// These are more efficient and do not require an allocator.
///
/// There is not any way to iterate over all currently allocated ids.
/// With the current implementation (a [`BinaryHeap`]),
/// it would be difficult to implement without any allocation.
///
/// [idmap]: https://docs.rs/idmap/
/// [`BinaryHeap`]: alloc::collections::BinaryHeap
pub struct IdAllocator<T: IntegerIdCounter> {
    next_id: crate::UniqueIdAllocator<T>,
    /// Tracks freed ids for reuse, preferring smaller ids where possible
    ///
    /// The use of a BinaryHeap here is inspired by the `thread-local` crate.
    /// No part of the implementation was copied.
    heap: alloc::collections::BinaryHeap<core::cmp::Reverse<intid::OrderByInt<T>>>,
}
impl<T: IntegerIdCounter> Default for IdAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IntegerIdCounter> IdAllocator<T> {
    /// Create a new allocator, with ids starting at [`T::START`] (usually zero).
    ///
    /// [`T::START`]: IntegerIdCounter::START
    #[inline]
    #[rustversion::attr(since(1.80), const)]
    pub fn new() -> Self {
        Self::with_start(T::START)
    }

    /// Create a new allocator, with ids starting at the specified value.
    #[inline]
    #[rustversion::attr(since(1.80), const)]
    pub fn with_start(start: T) -> Self {
        IdAllocator {
            next_id: UniqueIdAllocator::with_start(start),
            heap: alloc::collections::BinaryHeap::new(),
        }
    }

    /// Allocate a new id, reusing freed ids wherever possible.
    ///
    /// Returns an error if no more ids are available.
    #[inline]
    pub fn try_alloc(&mut self) -> Result<T, IdExhaustedError<T>> {
        match self.heap.pop() {
            Some(existing) => Ok(existing.0 .0),
            None => self.next_id.try_alloc(),
        }
    }

    /// Allocate a new id, reusing freed ids wherever possible.
    ///
    /// Panics if there are no ids available.
    #[track_caller]
    #[inline]
    #[must_use]
    pub fn alloc(&mut self) -> T {
        match self.try_alloc() {
            Ok(id) => id,
            Err(e) => e.panic(),
        }
    }

    /// Free all existing ids, resetting the allocator.
    #[inline]
    pub fn free_all(&mut self) {
        self.heap.clear();
        self.next_id.reset();
    }

    /// Free the specified id, making it available
    ///
    /// Used ids will be used in preference to creating new ones.
    #[inline]
    pub fn free(&mut self, id: T) {
        self.heap.push(core::cmp::Reverse(intid::OrderByInt(id)));
    }
}
