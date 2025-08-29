use crate::IdExhaustedError;
#[allow(unused_imports)] // used by docs
use crate::UniqueIdAllocator;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;
use intid::{uint, IntegerIdCounter};

/// Allocates unique integer ids across multiple threads.
///
/// This is an [`UniqueIdAllocator`] that uses atomic instructions,
/// and so is safe to share across threads.
#[derive(Debug)]
pub struct UniqueIdAllocatorAtomic<T: IntegerIdCounter> {
    // This could be improved by adding a T: bytemuck::NoUninit bound to IntegerIdCounter
    // It would allow us to avoid potentially costly conversions T <-> T::Int
    // and avoid the need for a separate with_start_const function
    //
    // The downside is it would add bytemuck as a required dependency,
    // and require more work in the intid-derive (would we derive nouninit or would bytemuck?)
    // As another alternative, we could switch to crossbeam-utils
    next_id: atomic::Atomic<T::Int>,
    marker: PhantomData<T>,
}
impl<T: IntegerIdCounter> Default for UniqueIdAllocatorAtomic<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: IntegerIdCounter> UniqueIdAllocatorAtomic<T> {
    /// Create a new allocator,
    /// using [`T::START`] as the first id (usually zero).
    ///
    /// [`T::START`]: IntegerIdCounter::START
    #[inline]
    pub const fn new() -> Self {
        UniqueIdAllocatorAtomic {
            next_id: atomic::Atomic::new(T::START_INT),
            marker: PhantomData,
        }
    }

    /// Create a new allocator,
    /// using the specified value as the first id.
    ///
    /// Use [`Self::with_start_const`] if you need a constant function.
    #[inline]
    pub fn with_start(start: T) -> Self {
        UniqueIdAllocatorAtomic {
            next_id: atomic::Atomic::new(start.to_int()),
            marker: PhantomData,
        }
    }

    /// Create a new allocator,
    /// using the specified value as the first id.
    ///
    /// In order to be usable from a `const` function,
    /// this requires that `T` implement the [`bytemuck::NoUninit`] trait
    /// and have the same size and representation as [`T::Int`](intid::IntegerId::Int).
    /// If that does not happen, this method will fail to compile with a const panic.
    ///
    /// ## Safety
    /// This function cannot cause undefined behavior.
    #[track_caller]
    pub const fn with_start_const(start: T) -> Self
    where
        T: bytemuck::NoUninit,
    {
        let start = bytemuck::must_cast::<T, T::Int>(start);
        UniqueIdAllocatorAtomic {
            next_id: atomic::Atomic::new(start),
            marker: PhantomData,
        }
    }

    /// Estimate the maximum currently used id,
    /// or `None` if no ids have been allocated yet.
    ///
    /// Unlike [`UniqueIdAllocator::max_used_id`]
    /// this is only an approximation.
    /// This is because other threads may be concurrently allocating a new id,
    /// and the load uses a [relaxed](core::sync::atomic::Ordering) ordering.
    /// In the current implementation, this should always be an under-estimate,
    /// since the counter only goes upwards.
    /// However, this should not be relied upon.
    #[inline]
    pub fn approx_max_used_id(&self) -> Option<T> {
        IntegerIdCounter::checked_sub(
            T::from_int_checked(self.next_id.load(Ordering::Relaxed))?,
            uint::one(),
        )
    }

    /// Attempt to allocate a new id,
    /// returning an error if exhausted.
    #[inline]
    pub fn try_alloc(&self) -> Result<T, IdExhaustedError<T>> {
        // Effectively this is "fused" because T: IntegerIdCounter => T: IntegerIdContiguous,
        // so once addition overflows all future calls will error
        //
        // See the comment in the Self::reset call for a way to potentially eliminate the CAS loop.
        self.next_id
            .fetch_update(Ordering::AcqRel, Ordering::Relaxed, |x| {
                uint::checked_add(x, uint::one())
            })
            .ok()
            .and_then(T::from_int_checked)
            .ok_or_else(IdExhaustedError::new)
    }

    /// Attempt to allocate a new id,
    /// panicking if exhausted.
    #[inline]
    #[must_use]
    pub fn alloc(&self) -> T {
        match self.try_alloc() {
            Ok(x) => x,
            Err(e) => e.panic(),
        }
    }

    /// Reset the allocator to a pristine state,
    /// beginning allocations all over again.
    ///
    /// This is equivalent to running `*allocator = UniqueIdAllocatorAtomic::new()`,
    /// but is done atomically and does not require a `&mut Self` reference.
    ///
    /// This may cause unexpected behavior if ids are expected to be monotonically increasing,
    /// or if the new ids conflict with ones still in use.
    /// To avoid this, keep the id allocator private.
    ///
    /// There is no counterpart [`UniqueIdAllocator::set_next_id`],
    /// because the ability to force the counter to jump forwards
    /// could prevent future optimizations.
    #[inline]
    pub fn reset(&self) {
        /*
         * I said this might prevent future optimizations.
         * What I am referring to is the potential to convert the CAS loop
         * into a fetch_add similar to how Arc::clone does.
         * Based on the assumption there are fewer than isize::MAX threads,
         * Arc::clone only has to worry about overflow if the counter exceeds that value.
         *
         * This seems like a micro-optimization but it could become important at some point.
         */
        self.next_id.store(T::START.to_int(), Ordering::Release)
    }
}
