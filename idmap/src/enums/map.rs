//! Defines the [`EnumMap`] type.

use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};

use crate::direct::macros::impl_direct_map_iter;
use crate::utils::{box_alloc_uninit, box_assume_init};
use intid::array::Array;
use intid::{uint, EnumId, EquivalentId, IntegerId};

/// A map from an [`EnumId`] key to values,
/// implemented using an inline array.
///
/// This is similar to a [`EnumMap`] and is also "direct",
/// although that is omitted from the name for conciseness.
/// Implementing the [`EnumId`] trait implies that the ids are relatively compact,
/// although this is not a strict requirement.
///
/// There is no entry API because the overhead of lookups is very small.
#[derive(Clone)]
pub struct EnumMap<K: EnumId, V> {
    table: K::Array<Option<V>>,
    len: u32,
    marker: PhantomData<K>,
}
impl<K: EnumId, V> Default for EnumMap<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl<K: EnumId, V> EnumMap<K, V> {
    /// Create a new map with no entries.
    #[inline]
    pub fn new() -> Self {
        let mut res = MaybeUninit::<Self>::uninit();
        Self::init(&mut res);
        // SAFETY: Initialized by `init` function
        unsafe { res.assume_init() }
    }
    /// Create a new map with no entries, allocating memory on the heap instead of the stack.
    ///
    /// Using `Box::new(EnumMapDirect::new())` could require moving the underlying table
    /// from the stack to the heap, as LLVM can struggle at eliminating copies.
    /// This method avoids that copy by always allocating in-place.
    #[inline]
    pub fn new_boxed() -> Box<Self> {
        let mut res = box_alloc_uninit::<Self>();
        Self::init(&mut *res);
        // SAFETY: Initialized by `init` function,
        unsafe { box_assume_init(res) }
    }
    #[inline]
    fn init(res: &mut MaybeUninit<Self>) -> &mut Self {
        Self::verify_len();
        // SAFETY: Known that pointer is valid and this struct has a `table` field
        // We use old macro instead of new syntax to support the MSRV
        let table: *mut K::Array<_> = unsafe { core::ptr::addr_of_mut!((*res.as_mut_ptr()).table) };
        // Valid since K::Array is really just a `[T; LEN]`
        let table = table.cast::<V>();
        // SAFETY: Memory is known to be valid, and [MaybeUninit<T>] does not require initialization
        let slice = unsafe {
            core::slice::from_raw_parts_mut(table as *mut MaybeUninit<Option<V>>, Self::TABLE_LEN)
        };
        for val in slice {
            // No need for panic safety because `None` has a nop Drop
            val.write(None);
        }
        // SAFETY: We know that the result pointer valid since it is a mutable reference
        // Now we are just initializing the other fields besides `table`
        unsafe { (*res.as_mut_ptr()).len = 0 };
        // SAFETY: We have initialized all the fields at this point
        unsafe { res.assume_init_mut() }
    }

    const TABLE_LEN: usize = <K::Array<Option<V>> as Array<Option<V>>>::LEN;

    fn verify_len() {
        let type_name = core::any::type_name::<K>();
        let expected_len = match K::MAX_ID_INT {
            None => 0,
            Some(max_id) => uint::to_usize_checked(max_id)
                .and_then(|x| x.checked_add(1))
                .unwrap_or_else(|| panic!("max_id for {type_name} overflows usize")),
        };
        assert_eq!(
            expected_len,
            Self::TABLE_LEN,
            "Unexpected array length for {type_name}"
        );
    }

    /// Determine the index of the specified key.
    #[inline]
    #[allow(clippy::unused_self)] // intentional
    fn index_of(&self, key: impl EquivalentId<K>) -> usize {
        uint::to_usize_wrapping(IntegerId::to_int(key.as_id()))
    }

    /// The number of entries in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Return true if this map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear all entries in the map.
    #[inline]
    pub fn clear(&mut self) {
        for val in self.table.as_mut() {
            *val = None;
        }
        self.len = 0;
    }

    /// Check if the specified key is present in the map.
    #[inline]
    pub fn contains_key(&self, id: impl EquivalentId<K>) -> bool {
        self.get(id).is_some()
    }

    /// Get the value associated with the specified key, or `None` if missing.
    #[inline]
    pub fn get(&self, id: impl EquivalentId<K>) -> Option<&V> {
        self.table.as_ref()[self.index_of(id)].as_ref()
    }

    /// Get a mutable reference to the value associated with the specified key,
    /// or `None` if missing.
    #[inline]
    pub fn get_mut(&mut self, id: impl EquivalentId<K>) -> Option<&mut V> {
        let index = self.index_of(id);
        self.table.as_mut()[index].as_mut()
    }

    /// Insert a key and a value, returning the previous value.
    #[inline]
    pub fn insert(&mut self, id: K, value: V) -> Option<V> {
        let index = self.index_of(id);
        let old_value = self.table.as_mut()[index].replace(value);
        if old_value.is_none() {
            self.len += 1;
        }
        old_value
    }

    /// Remove a value associated with the given,
    /// returning the previous value ifp resent.
    #[inline]
    pub fn remove(&mut self, id: impl EquivalentId<K>) -> Option<V> {
        let index = self.index_of(id);
        let old_value = self.table.as_mut()[index].take();
        if old_value.is_some() {
            self.len -= 1;
        }
        old_value
    }

    /// Iterate over the key-value pairs in the map.
    ///
    /// Guaranteed to be sorted by the integer id of the key.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            marker: PhantomData,
            len: self.len,
            source: self.table.as_ref().iter().enumerate(),
        }
    }

    /// Mutably iterate over the key-value pairs in the map.
    ///
    /// Guaranteed to be sorted by the integer id of the key.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            marker: PhantomData,
            len: self.len,
            source: self.table.as_mut().iter_mut().enumerate(),
        }
    }

    /// Iterate over the entries in the map,
    /// removing entries when the callback returns false.
    ///
    /// See also [`std::collections::HashMap::retain`].
    pub fn retain(&mut self, mut func: impl FnMut(K, &mut V) -> bool) {
        for (index, entry) in self.table.as_mut().iter_mut().enumerate() {
            let Some(ref mut entry_value) = entry else {
                continue;
            };
            // SAFETY: If entry exists, the key is guaranteed to be valid
            let key = unsafe { K::from_int_unchecked(intid::uint::from_usize_wrapping(index)) };
            if !func(key, entry_value) {
                *entry = None; // gotta love NLL
                self.len -= 1;
            }
        }
    }
}
impl<K: EnumId, V: PartialEq> PartialEq for EnumMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.table.as_ref() == other.table.as_ref()
    }
}
impl<K: EnumId, V: Eq> Eq for EnumMap<K, V> {}
impl<K: EnumId, V> Index<K> for EnumMap<K, V> {
    type Output = V;

    #[inline]
    #[track_caller]
    fn index(&self, index: K) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}
impl<K: EnumId, V> IndexMut<K> for EnumMap<K, V> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}
impl<'a, K: EnumId, V> Index<&'a K> for EnumMap<K, V> {
    type Output = V;

    #[inline]
    #[track_caller]
    fn index(&self, index: &'a K) -> &Self::Output {
        self.get(*index).expect("index out of bounds")
    }
}
impl<'a, K: EnumId, V> IndexMut<&'a K> for EnumMap<K, V> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: &'a K) -> &mut Self::Output {
        self.get_mut(*index).expect("index out of bounds")
    }
}
impl<K: EnumId, V> Extend<(K, V)> for EnumMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}
impl<'a, K: EnumId, V: Clone> Extend<(K, &'a V)> for EnumMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, &'a V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value.clone());
        }
    }
}
impl<K: EnumId, V> FromIterator<(K, V)> for EnumMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}
impl<'a, K: EnumId, V: Clone> FromIterator<(K, &'a V)> for EnumMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, &'a V)>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}
impl<K: EnumId, V> IntoIterator for EnumMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len,
            source: self.table.into_iter().enumerate(),
            marker: PhantomData,
        }
    }
}
impl<'a, K: EnumId, V> IntoIterator for &'a EnumMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: EnumId, V> IntoIterator for &'a mut EnumMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<K: EnumId, V: Debug> Debug for EnumMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/// An iterator consuming the entries in a [`EnumMap`]/
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct IntoIter<K: EnumId, V> {
    #[allow(clippy::type_complexity)] // we are actually hiding the complexity
    source: core::iter::Enumerate<<K::Array<Option<V>> as Array<Option<V>>>::Iter>,
    len: u32,
    marker: PhantomData<K>,
}
impl_direct_map_iter!(IntoIter<K: EnumId, V> {
    fn map(key, value) -> (K, V) {
        (key, value)
    }
});
/// An iterator over the entries in a [`EnumMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct Iter<'a, K: EnumId, V> {
    source: core::iter::Enumerate<core::slice::Iter<'a, Option<V>>>,
    len: u32,
    marker: PhantomData<K>,
}
impl_direct_map_iter!(Iter<'a, K: EnumId, V> {
    fn map(key, value) -> (K, &'a V) {
        (key, value)
    }
});

/// A mutable iterator over the entries in a [`EnumMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct IterMut<'a, K: EnumId, V> {
    source: core::iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    len: u32,
    marker: PhantomData<K>,
}
impl_direct_map_iter!(IterMut<'a, K: EnumId, V> {
    fn map(key, value) -> (K, &'a mut V) {
        (key, value)
    }
});

/// A iterator over the values in a [`EnumMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct Values<'a, K: EnumId, V> {
    source: core::iter::Enumerate<core::slice::Iter<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_map_iter!(Values<'a, K: EnumId, V> {
    fn map(_key, value) -> &'a V {
        value
    }
});

/// A mutable iterator over the values in a [`EnumMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct ValuesMut<'a, K: EnumId, V> {
    source: core::iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_map_iter!(ValuesMut<'a, K: EnumId, V> {
    fn map(_key, value) -> &'a mut V {
        value
    }
});

/// A iterator over the keys in a [`EnumMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct Keys<'a, K: IntegerId, V> {
    source: core::iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_map_iter!(Keys<'a, K: IntegerId, V> {
    fn map(key, _value) -> K {
        key
    }
});

/// Creates a [`EnumMap`] from a set of key-value pairs.
#[macro_export]
macro_rules! enum_map {
    () => ($crate::enums::EnumMap::new());
    ($($key:expr => $value:expr),+ $(,)?) => ({
        let mut res = $crate::enums::EnumMap::new();
        $(res.insert($key, $value);)*
        res
    });
}
