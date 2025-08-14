//! Defines the [`IntegerKey`] trait.

use crate::IntegerId;

/// A key with an associated integer id,
/// which can be used as an index in a lookup table.
///
/// There is a blanket implementation for all [`IntegerId`].
///
/// This is less restrictive than an [`IntegerId`],
/// because it does not require the type to implement `Copy`.
pub trait IntegerKey: Eq {
    /// The type of integer value that is associated with the key
    /// and used to index into lookup tables.
    type Index: IntegerId;
    /// A borrowed reference to the key.
    type Ref<'a>: 'a;
    /// A mutable reference to the key.
    type MutRef<'a>: 'a;
    /// Data that needs to be stored and cannot be reconstructed from the integer index.
    ///
    /// For an [`IntegerId`], this is `()` because an index is sufficient to recreate the id.
    /// However, if the key holds onto allocated memory then that likely needs to be stored here.
    type Storage: Sized;
    /// Convert this type into its associated storage
    #[allow(clippy::wrong_self_convention)]
    fn into_storage(this: Self) -> Self::Storage;
    /// Reconstruct this key from the storage and index.
    fn from_storage(storage: Self::Storage, index: Self::Index) -> Self;
    /// Create a reference to this key provided a reference to storage and an index.
    fn from_storage_ref(storage: &Self::Storage, index: Self::Index) -> Self::Ref<'_>;
    /// Create a mutable reference to this type provided a reference to storage and an index.
    fn from_storage_mut(storage: &mut Self::Storage, index: Self::Index) -> Self::MutRef<'_>;
    /// Get the integer index associated with this key.
    #[allow(clippy::wrong_self_convention)]
    fn to_index(this: &Self) -> Self::Index;
}
impl<T: IntegerId> IntegerKey for T {
    type Index = Self;
    type Ref<'a> = Self;
    type MutRef<'a> = Self;
    type Storage = ();

    #[inline]
    fn into_storage(_this: Self) -> Self::Storage {
        // no additional storage needed
    }

    #[inline]
    fn from_storage(_storage: Self::Storage, index: Self::Index) -> Self {
        // can be reconstructed just from the index
        index
    }

    #[inline]
    fn from_storage_ref(_storage: &Self::Storage, index: Self::Index) -> Self::Ref<'_> {
        index
    }

    fn from_storage_mut(_storage: &mut Self::Storage, index: Self::Index) -> Self::MutRef<'_> {
        index
    }

    #[inline]
    fn to_index(this: &Self) -> Self::Index {
        *this
    }
}

/// A type that is equivalent to an [`IntegerKey`].
///
/// Used for key lookup in maps, similar to [core::borrow::Borrow] or [equivalent::Equivalent].
/// These traits are not suitable for id maps,
/// which need conversion to integers rather than hashing/equality.
///
/// [equivalent::Equivalent]: https://docs.rs/equivalent/latest/equivalent/trait.Equivalent.html
pub trait EquivalentIntKey<K: IntegerKey> {
    /// Convert this type to an integer index.
    #[allow(clippy::wrong_self_convention)]
    fn to_key_index(this: &Self) -> K::Index;
}
impl<K: IntegerKey> EquivalentIntKey<K> for K {
    #[inline]
    fn to_key_index(this: &Self) -> K::Index {
       K::to_index(this)
    }
}
impl<K: IntegerKey> EquivalentIntKey<K> for &'_ K {
    #[inline]
    fn to_key_index(this: &Self) -> K::Index {
        K::to_index(*this)
    }
}
impl<K: IntegerKey> EquivalentIntKey<K> for &'_ mut K {
    #[inline]
    fn to_key_index(this: &Self) -> K::Index {
        K::to_index(&*this)
    }
}
