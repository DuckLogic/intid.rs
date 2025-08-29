//! Implements an `IdSet` using a bitset
//!
//! IdSets are to HashSets as IdMaps are to HashMaps

use core::cmp::Ordering;
use core::fmt::{self, Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::iter;
use core::marker::PhantomData;
use core::ops::Index;
use iter::FusedIterator;

use fixedbitset::{FixedBitSet, Ones};
use intid::{EquivalentId, IntegerId};

/// A set whose members implement [IntegerId].
///
/// This is implemented as a bitset,
/// so memory is proportional to the highest integer index.
#[derive(Clone)]
pub struct DirectIdSet<T: IntegerId> {
    handle: FixedBitSet,
    len: usize,
    marker: PhantomData<T>,
}
impl<T: IntegerId> DirectIdSet<T> {
    /// Create a new [DirectIdSet] with no elements.
    #[inline]
    pub const fn new() -> Self {
        DirectIdSet {
            handle: FixedBitSet::new(),
            len: 0,
            marker: PhantomData,
        }
    }

    /// Initialize the set with the given capacity
    ///
    /// Since this is a direct set,
    /// this hints at the maximum valid id and not the length.
    #[inline]
    pub fn with_capacity(max_id: usize) -> Self {
        DirectIdSet {
            handle: FixedBitSet::with_capacity(max_id),
            len: 0,
            marker: PhantomData,
        }
    }

    /// Inserts the specified element into the set,
    /// returning `true` if it was already in the set and `false` if it wasn't.
    #[inline]
    pub fn insert(&mut self, value: T) -> bool {
        let value = value.to_int();
        let index: usize =
            intid::uint::to_usize_checked(value).unwrap_or_else(|| super::oom_id(value));
        let was_present = self.handle.contains(index);
        self.handle.grow_and_insert(index);
        if !was_present {
            self.len += 1;
        }
        was_present
    }

    /// Remove the specified value from the set,
    /// returning whether it was previously present.
    #[inline]
    pub fn remove(&mut self, value: impl EquivalentId<T>) -> bool {
        let value = value.as_id().to_int();
        let Some(index) = intid::uint::to_usize_checked(value) else {
            return false; // overflow -> not present
        };
        if index >= self.handle.len() {
            false
        } else {
            // SAFETY: Checked bounds
            let was_present = unsafe { self.handle.contains_unchecked(index) };
            // SAFETY: Checked bounds
            unsafe { self.handle.remove_unchecked(index) };
            if was_present {
                self.len -= 1;
            }
            was_present
        }
    }

    /// Check if this set contains the specified value
    #[inline]
    pub fn contains(&self, value: impl EquivalentId<T>) -> bool {
        let value = value.as_id().to_int();
        // is_some_and requires 1.70
        match intid::uint::to_usize_checked(value) {
            None => false,
            Some(x) => self.handle.contains(x),
        }
    }

    /// Iterate over the values in this set.
    ///
    /// Guaranteed to be ordered by the integer value of the key.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            len: self.len,
            handle: self.handle.ones(),
            marker: PhantomData,
        }
    }

    /// Clear the values in this set
    #[inline]
    pub fn clear(&mut self) {
        self.handle.clear();
        self.len = 0;
    }

    /// The number of entries in this set
    ///
    /// An [DirectIdSet] internally tracks this length, so this is a `O(1)` operation
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// If this set is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Retain values in the set if the specified closure returns true
    ///
    /// Otherwise, they are removed
    pub fn retain<F: FnMut(T) -> bool>(&mut self, mut func: F) {
        for (word_index, word) in self.handle.as_mut_slice().iter_mut().enumerate() {
            let (updated_word, word_removed) = retain_word(*word, |bit| {
                let id = (word_index * 32) + (bit as usize);
                // Safety: If present in the map, it is known to be valid
                let key = unsafe { T::from_int_unchecked(intid::uint::from_usize_wrapping(id)) };
                func(key)
            });
            *word = updated_word;
            self.len -= word_removed as usize;
        }
    }
}
/// The type of a word in a [`FixedBitSet`].
type Word = fixedbitset::Block;
#[inline]
fn retain_word<F: FnMut(u32) -> bool>(original_word: Word, mut func: F) -> (Word, u32) {
    let mut remaining = original_word;
    let mut result = original_word;
    let mut removed = 0;
    while remaining != 0 {
        let bit = remaining.trailing_zeros();
        let mask: Word = 1 << bit;
        debug_assert_ne!(result & mask, 0);
        if !func(bit) {
            result &= !mask;
            removed += 1;
        }
        remaining &= !mask;
    }
    debug_assert!(removed <= 32);
    (result, removed)
}
impl<T: IntegerId> Default for DirectIdSet<T> {
    #[inline]
    fn default() -> Self {
        DirectIdSet::new()
    }
}
impl<T: IntegerId> PartialEq for DirectIdSet<T> {
    #[inline]
    fn eq(&self, other: &DirectIdSet<T>) -> bool {
        self.len == other.len && self.handle == other.handle
    }
}
impl<T: IntegerId> Eq for DirectIdSet<T> {}
impl<T: IntegerId> Debug for DirectIdSet<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}
impl<T: IntegerId> Extend<T> for DirectIdSet<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter.into_iter() {
            self.insert(value);
        }
    }
}
impl<'a, T: IntegerId> Extend<&'a T> for DirectIdSet<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied())
    }
}
impl<T: IntegerId> FromIterator<T> for DirectIdSet<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut set = DirectIdSet::new();
        set.extend(iter);
        set
    }
}

impl<'a, T: IntegerId> FromIterator<&'a T> for DirectIdSet<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        iter.into_iter().copied().collect()
    }
}

impl<'a, T: IntegerId + 'a> IntoIterator for &'a DirectIdSet<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<T: IntegerId> IntoIterator for DirectIdSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len,
            marker: PhantomData,
            handle: self.handle.into_ones(),
        }
    }
}

impl<'a, T: IntegerId + 'a> Index<&'a T> for DirectIdSet<T> {
    type Output = bool;

    #[inline]
    fn index(&self, index: &'a T) -> &Self::Output {
        &self[*index]
    }
}
impl<T: IntegerId> Index<T> for DirectIdSet<T> {
    type Output = bool;

    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        const TRUE_REF: &bool = &true;
        const FALSE_REF: &bool = &false;
        if self.contains(index) {
            TRUE_REF
        } else {
            FALSE_REF
        }
    }
}
impl<T: IntegerId + Hash> Hash for DirectIdSet<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.len());
        // guaranteed to be ordered key
        for value in self.iter() {
            value.hash(state);
        }
    }
}
impl<T: IntegerId + PartialOrd> PartialOrd for DirectIdSet<T> {
    #[inline]
    fn partial_cmp(&self, other: &DirectIdSet<T>) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}
impl<T: IntegerId + Ord> Ord for DirectIdSet<T> {
    #[inline]
    fn cmp(&self, other: &DirectIdSet<T>) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

macro_rules! do_impl_iter {
    ($target:ident$(<$lt:lifetime>)?) => {
        impl<T: IntegerId> Iterator for $target<$($lt,)* T> {
            type Item = T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self.handle.next() {
                    Some(index) => {
                        self.len -= 1;
                        // SAFETY: Id is present => id is valid
                        Some(unsafe { T::from_int_unchecked(intid::uint::from_usize_wrapping(index)) })
                    }
                    None => {
                        debug_assert_eq!(self.len, 0);
                        None
                    }
                }
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.len, Some(self.len))
            }
            #[inline]
            fn count(self) -> usize
            where
                Self: Sized,
            {
                self.len
            }
        }
        impl<T: IntegerId> DoubleEndedIterator for $target<$($lt,)* T> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                match self.handle.next_back() {
                    Some(index) => {
                        self.len -= 1;
                        // SAFETY: Id is present => id is valid
                        Some(unsafe { T::from_int_unchecked(intid::uint::from_usize_wrapping(index)) })
                    }
                    None => {
                        debug_assert_eq!(self.len, 0);
                        None
                    }
                }
            }
        }
        impl<T: IntegerId> ExactSizeIterator for $target<$($lt,)* T> {}
        impl<T: IntegerId> FusedIterator for $target<$($lt,)* T> {}
    };
}
/// An iterator over the values in an [DirectIdSet].
///
/// TODO: Cannot implement `Clone` because [`fixedbitset::Ones`] doesn't support it yet.
/// It was added in [PR #130], but no public release has been made yet.
///
/// [PR #130]: https://github.com/petgraph/fixedbitset/pull/130
pub struct Iter<'a, T: IntegerId> {
    len: usize,
    handle: Ones<'a>,
    marker: PhantomData<T>,
}
do_impl_iter!(Iter<'_>);

/// An iterator over the values in an [`DirectIdSet`],
/// consuming ownership the set.
///
/// *NOTE*: Cannot implement `Clone` because [`fixedbitset::IntoOnes`] doesn't support it.
pub struct IntoIter<T: IntegerId> {
    handle: fixedbitset::IntoOnes,
    len: usize,
    marker: PhantomData<T>,
}
do_impl_iter!(IntoIter);

#[cfg(feature = "petgraph_0_8")]
impl<T: IntegerId> petgraph_0_8::visit::VisitMap<T> for DirectIdSet<T> {
    #[inline]
    fn visit(&mut self, a: T) -> bool {
        !self.insert(a)
    }
    #[inline]
    fn is_visited(&self, value: &T) -> bool {
        self.contains(*value)
    }
    #[inline]
    fn unvisit(&mut self, a: T) -> bool {
        self.remove(a)
    }
}

/// Creates a [`DirectIdSet`] from a list of values
#[macro_export]
macro_rules! direct_idset {
    () => ($crate::direct::DirectIdSet::new());
    ($($value:expr),+ $(,)?) => ({
        let mut set = $crate::direct::DirectIdSet::new();
        $(set.insert($value);)*
        set
    });
}
