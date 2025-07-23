use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::num::*;
use std::rc::Rc;
use std::sync::Arc;

/// A type that can be uniquely identified by a 64 bit integer id
pub trait IntegerId: PartialEq + Debug {
    /// Recreate this key based on its associated integer id
    ///
    /// This must be consistent with [IntegerId::id]
    ///
    /// This should assume no overflow in release mode
    /// (unless that would be unsafe). However in debug mode builds
    /// this should check for overflow.
    fn from_id(id: u64) -> Self;
    /// Return the unique id of this value.
    /// If two values are equal, they _must_ have the same id,
    /// and if two values aren't equal, they _must_ have different ids.
    fn id(&self) -> u64;
    /// Return the 32-bit unique id of this value, panicking on overflow
    fn id32(&self) -> u32;
}
macro_rules! nonzero_id {
    ($($target:ident),*) => {$(
        impl IntegerId for $target {
            #[inline]
            #[track_caller]
            fn from_id(id: u64) -> Self {
                let value = IntegerId::from_id(id);
                $target::new(value).unwrap()
            }
            #[inline]
            fn id(&self) -> u64 {
                self.get().id()
            }
            #[inline]
            fn id32(&self) -> u32 {
                self.get().id32()
            }
        }
    )*};
}
nonzero_id!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize);

macro_rules! primitive_id {
    ($($target:ident),*) => {$(
        impl IntegerId for $target {
            #[inline]
            #[track_caller]
            fn from_id(id: u64) -> Self {
                if cfg!(debug_assertions) && <$target>::try_from(id).is_err() {
                    #[allow(unused_comparisons)]
                    {
                        assert!(id as $target >= 0, "Negative id: {}", id as $target);
                    }
                    panic!("Id overflowed a {}: {}", stringify!($target), id);
                }
                id as $target
            }
            #[inline(always)]
            fn id(&self) -> u64 {
                *self as u64
            }
            #[inline]
            fn id32(&self) -> u32 {
                #[allow(unused_comparisons)]
                const SIGNED: bool = $target::MIN < 0;
                // Preserve wonky behavior for signed ints, for backwards compatibility reasons.
                // It never worked very well, requiring inordinate amounts of memory.
                if SIGNED {
                    /*
                     * NOTE: We attempt the lossy conversion to i32 for signed ints, then convert to u32 afterwards.
                     * If we casted directly from i64 -> u32 it'd fail for negatives,
                     * and if we casted from i64 -> u64 first, small negatives would fail to cast since they'd be too large.
                     * For example, -1 would become 0xFFFFFFFF which would overflow a u32,
                     * but if we first cast to a i32 it'd become 0xFFFF which would fit fine.
                     */
                    let full_value = i32::try_from(*self).unwrap_or_else(|_| id_overflowed(*self));
                    full_value as u32
                } else {
                    u32::try_from(*self).unwrap_or_else(|_| id_overflowed(*self))
                }
            }
        }
    )*};
}
primitive_id!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// Support function that panics if an id overflows a u32
#[cold]
#[inline(never)]
#[track_caller]
fn id_overflowed<T: Copy + Display>(id: T) -> ! {
    panic!("ID overflowed a u32: {id}");
}

macro_rules! generic_deref_id {
    ($target:ident) => {
        /// **WARNING**: This implementation is deprecated as of v0.2.22,
        /// and will be removed in v0.3.0.
        impl<T: IntegerId> IntegerId for $target<T> {
            #[inline(always)]
            fn from_id(id: u64) -> Self {
                $target::new(T::from_id(id))
            }
            #[inline]
            fn id(&self) -> u64 {
                (**self).id()
            }

            #[inline]
            fn id32(&self) -> u32 {
                (**self).id32()
            }
        }
    };
}
generic_deref_id!(Rc);
generic_deref_id!(Box);
generic_deref_id!(Arc);

#[cfg(feature = "petgraph")]
impl<T> IntegerId for ::petgraph::graph::NodeIndex<T>
where
    T: ::petgraph::graph::IndexType + IntegerId,
{
    #[inline]
    fn from_id(id: u64) -> Self {
        Self::from(T::from_id(id))
    }
    #[inline]
    fn id(&self) -> u64 {
        T::new(self.index()).id()
    }

    #[inline]
    fn id32(&self) -> u32 {
        T::new(self.index()).id32()
    }
}
