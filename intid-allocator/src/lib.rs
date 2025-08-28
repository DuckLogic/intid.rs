//! Allocates integer ids which implement [`intid::IntegerIdCounter`].
//!
//! Use [`IdAllocator`] if you want to be able to [free](IdAllocator::free) existing ids for reuse.
//! This will minimize the integer value of the keys, reducing memory needed for lookup tables.
//!
//! Use [`UniqueIdAllocator`] or [`UniqueIdAllocatorAtomic`] if you don't care about reusing existing keys.
//! These are more efficient and never require any allocation.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)] // not needed yet

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
use intid::IntegerId;

#[cfg(feature = "alloc")]
mod reusing;
mod unique;

#[cfg(feature = "alloc")]
pub use self::reusing::IdAllocator;
#[cfg(feature = "atomic")]
pub use self::unique::atomic::UniqueIdAllocatorAtomic;
pub use self::unique::UniqueIdAllocator;

/// Indicates that available ids have been exhausted,
/// and can no longer be allocated.
#[derive(Clone)]
pub struct IdExhaustedError<T: IntegerId> {
    marker: PhantomData<T>,
}
impl<T: IntegerId> IdExhaustedError<T> {
    /// Indicate that ids have been exhausted for the type `T`,
    /// without giving any additional information.
    #[inline]
    #[cold]
    #[allow(clippy::new_without_default)] // doesn't make much sense for an error
    #[must_use]
    pub fn new() -> Self {
        IdExhaustedError {
            marker: PhantomData,
        }
    }

    /// Trigger a descriptive panic due to this error.
    ///
    /// This gives a better panic message than calling [`Result::unwrap`].
    ///
    /// # Panics
    /// Always.
    #[track_caller]
    #[cold]
    pub fn panic(self) -> ! {
        panic!("{self}")
    }
}
impl<T: IntegerId> Display for IdExhaustedError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Ran out of ids for {}", core::any::type_name::<T>())
    }
}
impl<T: IntegerId> Debug for IdExhaustedError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IdExhaustedError")
            .field("type_name", &core::any::type_name::<T>())
            .finish_non_exhaustive()
    }
}

#[rustversion::since(1.81)]
impl<T: IntegerId> core::error::Error for IdExhaustedError<T> {}

#[rustversion::before(1.81)]
#[cfg(feature = "std")]
impl<T: IntegerId> std::error::Error for IdExhaustedError<T> {}
