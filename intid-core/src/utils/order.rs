//! Implements [`OrderByInt`].

use crate::{EquivalentId, IntegerId, IntegerIdContiguous, IntegerIdCounter};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

/// A wrapper around an [`IntegerId`] which implements [`Eq`], [`Ord`], and [`Hash`]
/// based on the integer value.
#[derive(Copy, Clone, Debug, Default)]
pub struct OrderByInt<T: IntegerId>(pub T);
impl<T: IntegerId> IntegerId for OrderByInt<T> {
    impl_newtype_id_body!(for OrderByInt(T));
}
impl<T: IntegerIdContiguous> IntegerIdContiguous for OrderByInt<T> {}
impl<T: IntegerIdCounter> IntegerIdCounter for OrderByInt<T> {
    const START: Self = OrderByInt(T::START);
    const START_INT: Self::Int = T::START_INT;
}
impl<T: IntegerId> Ord for OrderByInt<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_int().cmp(&other.0.to_int())
    }
}
impl<T: IntegerId> PartialOrd for OrderByInt<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: IntegerId> Eq for OrderByInt<T> {}
impl<T: IntegerId> PartialEq for OrderByInt<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.to_int() == other.0.to_int()
    }
}
impl<T: IntegerId> PartialEq<T> for OrderByInt<T> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T: IntegerId> PartialOrd<T> for OrderByInt<T> {
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        Some(self.cmp(&OrderByInt(*other)))
    }
}
impl<T: IntegerId> Hash for OrderByInt<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_int().hash(state);
    }
}
impl<T: IntegerId> EquivalentId<T> for OrderByInt<T> {
    #[inline]
    fn as_id(&self) -> T {
        self.0
    }
}
impl<T: IntegerId> EquivalentId<T> for &'_ OrderByInt<T> {
    #[inline]
    fn as_id(&self) -> T {
        self.0
    }
}
impl<T: IntegerId> From<T> for OrderByInt<T> {
    #[inline]
    fn from(value: T) -> Self {
        OrderByInt(value)
    }
}
impl<T: IntegerId> AsRef<T> for OrderByInt<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: IntegerId> AsMut<T> for OrderByInt<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
/// NOTE: We cannot implement `Borrow`, because our `Hash + Eq` might be different.
#[cfg(any())]
impl<T> Borrow for OrderByInt<T> {}
