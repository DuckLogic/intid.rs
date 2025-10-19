//! Implements [`DirectIdMap`], a thin wrapper over a [`Vec<Option<T>>`].

use crate::direct::oom_id;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use intid::{EquivalentId, IntegerId};

/// A map implemented as a [`Vec<Option<T>>`],
/// which takes space proportional to the size of the maximum id.
///
/// There is no entry API because the overhead of lookups is very small.
#[derive(Clone)]
pub struct DirectIdMap<K: IntegerId, V> {
    // Optimization idea: If `Option<V>` does not support the nullable-pointer optimization,
    // fallback to using a bitset + MaybeUninit.
    // In some cases, this could save a significant amount of space.
    values: Vec<Option<V>>,
    len: usize,
    marker: PhantomData<K>,
}
impl<K: IntegerId, V> Default for DirectIdMap<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl<K: IntegerId, V> DirectIdMap<K, V> {
    /// Create a new map with no entries.
    #[inline]
    pub const fn new() -> Self {
        DirectIdMap {
            values: Vec::new(),
            len: 0,
            marker: PhantomData,
        }
    }
    /// The number of entries in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return true if this map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear all entries in the map.
    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
        self.len = 0;
    }

    /// Trim unused capacity.
    pub fn shrink_to_fit(&mut self) {
        while matches!(self.values.last(), Some(None)) {
            self.values.pop();
        }
        self.values.shrink_to_fit();
    }

    /// Check if the specified key is present in the map.
    #[inline]
    pub fn contains_key(&self, id: impl EquivalentId<K>) -> bool {
        self.get(id).is_some()
    }

    /// Get the value associated with the specified key, or `None` if missing.
    #[inline]
    pub fn get(&self, id: impl EquivalentId<K>) -> Option<&V> {
        let id = id.as_id();
        self.values
            .get(intid::uint::to_usize_checked(id.to_int())?)?
            .as_ref()
    }

    /// Get a mutable reference to the value associated with the specified key,
    /// or `None` if missing.
    #[inline]
    pub fn get_mut(&mut self, id: impl EquivalentId<K>) -> Option<&mut V> {
        let id = id.as_id();
        self.values
            .get_mut(intid::uint::to_usize_checked(id.to_int())?)?
            .as_mut()
    }

    /// Insert a key and a value, returning the previous value.
    #[inline]
    pub fn insert(&mut self, id: K, value: V) -> Option<V> {
        let id = id.to_int();
        let id = intid::uint::to_usize_checked(id).unwrap_or_else(|| oom_id(id));
        self.grow_to(id);
        let old_value = self.values[id].replace(value);
        if old_value.is_none() {
            self.len += 1;
        }
        old_value
    }

    /// Remove a value associated with the given,
    /// returning the previous value ifp resent.
    #[inline]
    pub fn remove(&mut self, id: impl EquivalentId<K>) -> Option<V> {
        let id = id.as_id().to_int();
        let id = intid::uint::to_usize_checked(id).unwrap_or_else(|| oom_id(id));
        if id >= self.values.len() {
            return None;
        }
        let old_value = self.values[id].take();
        if old_value.is_some() {
            self.len -= 1;
        }
        old_value
    }

    #[inline]
    fn grow_to(&mut self, max_id: usize) {
        if self.values.len() <= max_id {
            self.grow_fallback(max_id);
        }
    }
    #[cold]
    fn grow_fallback(&mut self, max_id: usize) {
        // amortized growth
        let new_len = core::cmp::max(
            self.len().checked_mul(2).expect("capacity overflow"),
            max_id.checked_add(1).unwrap_or_else(|| oom_id(max_id)),
        );
        assert!(new_len >= self.values.len());
        assert!(new_len > max_id);
        self.values.resize_with(new_len, || None);
    }

    /// Iterate over the key-value pairs in the map.
    ///
    /// Guaranteed to be sorted by the integer id of the key.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            marker: PhantomData,
            len: self.len,
            source: self.values.iter().enumerate(),
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
            source: self.values.iter_mut().enumerate(),
        }
    }

    /// Iterate over the entries in the map,
    /// removing entries when the callback returns false.
    ///
    /// See also [`std::collections::HashMap::retain`].
    pub fn retain(&mut self, mut func: impl FnMut(K, &mut V) -> bool) {
        for (index, entry) in self.values.iter_mut().enumerate() {
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
impl<K: IntegerId, V: PartialEq> PartialEq for DirectIdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.values == other.values
    }
}
impl<K: IntegerId, V: Eq> Eq for DirectIdMap<K, V> {}
impl<K: IntegerId, V> Index<K> for DirectIdMap<K, V> {
    type Output = V;

    #[inline]
    #[track_caller]
    fn index(&self, index: K) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}
impl<K: IntegerId, V> IndexMut<K> for DirectIdMap<K, V> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}
impl<'a, K: IntegerId, V> Index<&'a K> for DirectIdMap<K, V> {
    type Output = V;

    #[inline]
    #[track_caller]
    fn index(&self, index: &'a K) -> &Self::Output {
        self.get(*index).expect("index out of bounds")
    }
}
impl<'a, K: IntegerId, V> IndexMut<&'a K> for DirectIdMap<K, V> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: &'a K) -> &mut Self::Output {
        self.get_mut(*index).expect("index out of bounds")
    }
}
impl<K: IntegerId, V> Extend<(K, V)> for DirectIdMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}
impl<'a, K: IntegerId, V: Clone> Extend<(K, &'a V)> for DirectIdMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, &'a V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value.clone());
        }
    }
}

impl<K: IntegerId, V> FromIterator<(K, V)> for DirectIdMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}
impl<'a, K: IntegerId, V: Clone> FromIterator<(K, &'a V)> for DirectIdMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, &'a V)>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}
impl<K: IntegerId, V> IntoIterator for DirectIdMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len,
            source: self.values.into_iter().enumerate(),
            marker: PhantomData,
        }
    }
}
impl<'a, K: IntegerId, V> IntoIterator for &'a DirectIdMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: IntegerId, V> IntoIterator for &'a mut DirectIdMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<K: IntegerId, V: Debug> Debug for DirectIdMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
macro_rules! impl_direct_iter {
    ($target:ident<$($l:lifetime,)? $kt:ident, $vt:ident> {
        fn map($k:ident, $v:ident) -> $item_ty:ty {
            $map:expr
        }
    }) => {
        impl<$($l,)* $kt: IntegerId, $vt> Iterator for $target<$($l,)* $kt, $vt> {
            type Item = $item_ty;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match self.source.next() {
                        Some((index, Some($v))) => {
                            // SAFETY: Value exists => index is valid
                            let $k = unsafe {
                                $kt::from_int_unchecked(intid::uint::from_usize_wrapping(index))
                            };
                            self.len -= 1;
                            return Some($map)
                        },
                        Some((_, None)) => continue,
                        None => return None,
                    }
                }
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.len, Some(self.len))
            }
        }
        impl<$($l,)* $kt: IntegerId, $vt> DoubleEndedIterator for $target<$($l,)* $kt, $vt> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                loop {
                    match self.source.next_back() {
                        Some((index, Some($v))) => {
                            // SAFETY: Value exists => index is valid
                            let $k = unsafe {
                                $kt::from_int_unchecked(intid::uint::from_usize_wrapping(index))
                            };
                            self.len -= 1;
                            return Some($map)
                        },
                        Some((_, None)) => continue,
                        None => return None,
                    }
                }
            }
        }
        impl<$($l,)* $kt: IntegerId, $vt> ExactSizeIterator for $target<$($l,)* $kt, $vt> {}
        impl<$($l,)* $kt: IntegerId, $vt> core::iter::FusedIterator for $target<$($l,)* $kt, $vt> {}
    }
}
/// An iterator consuming the entries in a [`DirectIdMap`]/
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct IntoIter<K: IntegerId, V> {
    source: core::iter::Enumerate<alloc::vec::IntoIter<Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_iter!(IntoIter<K, V> {
    fn map(key, value) -> (K, V) {
        (key, value)
    }
});
/// An iterator over the entries in a [`DirectIdMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct Iter<'a, K: IntegerId, V> {
    source: core::iter::Enumerate<core::slice::Iter<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_iter!(Iter<'a, K, V> {
    fn map(key, value) -> (K, &'a V) {
        (key, value)
    }
});

/// A mutable iterator over the entries in a [`DirectIdMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct IterMut<'a, K: IntegerId, V> {
    source: core::iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_iter!(IterMut<'a, K, V> {
    fn map(key, value) -> (K, &'a mut V) {
        (key, value)
    }
});

/// A iterator over the values in a [`DirectIdMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct Values<'a, K: IntegerId, V> {
    source: core::iter::Enumerate<core::slice::Iter<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_iter!(Values<'a, K, V> {
    fn map(_key, value) -> &'a V {
        value
    }
});

/// A mutable iterator over the values in a [`DirectIdMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct ValuesMut<'a, K: IntegerId, V> {
    source: core::iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_iter!(ValuesMut<'a, K, V> {
    fn map(_key, value) -> &'a mut V {
        value
    }
});

/// A iterator over the keys in a [`DirectIdMap`].
///
/// Guaranteed to be ordered by the integer value of the key.
pub struct Keys<'a, K: IntegerId, V> {
    source: core::iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    len: usize,
    marker: PhantomData<K>,
}
impl_direct_iter!(Keys<'a, K, V> {
    fn map(key, _value) -> K {
        key
    }
});

/// Creates a [`DirectIdMap`] from a set of key-value pairs.
#[macro_export]
macro_rules! direct_idmap {
    () => ($crate::direct::DirectIdMap::new());
    ($($key:expr => $value:expr),+ $(,)?) => ({
        let mut res = $crate::direct::DirectIdMap::new();
        $(res.insert($key, $value);)*
        res
    });
}
