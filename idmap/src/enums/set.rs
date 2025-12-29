//! Implements an [`EnumSet`] using a bitset.

use crate::direct::macros::impl_direct_set_iter;
use crate::utils::bitsets::ones::OnesIter;
use crate::utils::bitsets::retain_word;
use alloc::boxed::Box;
use core::cmp::Ordering;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ops::Index;
use intid::array::{Array, BitsetLimb};
use intid::{EnumId, EquivalentId};

/// A set whose members implement [`EnumId`].
///
/// This is implemented as a bitset,
/// so memory is proportional to [`EnumId::COUNT`].
#[derive(Clone)]
pub struct EnumSet<T: EnumId> {
    limbs: T::BitSet,
    /// It is possible to avoid storing this field by using a [popcount] instruction
    /// like [`u64::count_ones`]
    ///
    /// On older architectures, popcount can be very slow.
    /// Even on recent Intel architectures, the instruction has a 3-cycle latency.
    /// We don't want the `len()` call to any slower than [`crate::DirectIdSet`],
    /// so we unconditionally store the length even when [`T::COUNT`] is small.
    ///
    /// Intel AVX2 has instructions to accelerate popcount computation,
    /// as discussed in [this paper] and implemented in [this library].
    /// We could consider implementing this behind a cfg-flag
    /// if the space savings become significant enough.
    ///
    /// It is safe to use a `u32` because `EnumId::MAX_ID + 1` is guaranteed to always fit in it.
    /// Restricting [`EnumId`] to 16-bits does not give any space advantage here.
    /// As long as the limbs in the array are at least 4-byte aligned,
    /// a 16-bit length requires 2 bytes of padding and so is effectively the same size as a 32-bit length.
    /// See [issue #4](https://github.com/DuckLogic/intid.rs/issues/14) for history.
    ///
    /// [popcount]: https://en.wikipedia.org/wiki/Hamming_weight
    /// [this paper]: https://arxiv.org/pdf/1611.07a612
    /// [this library]: https://github.com/kimwalisch/libpopcnt
    len: u32,
    marker: PhantomData<T>,
}
#[inline]
fn divmod_index(index: u32) -> (usize, u32) {
    (
        (index / BitsetLimb::BITS) as usize,
        index % BitsetLimb::BITS,
    )
}
#[inline]
fn bitmask_for(bit_index: u32) -> BitsetLimb {
    let one: BitsetLimb = 1;
    one << bit_index
}
impl<T: EnumId> EnumSet<T> {
    /// Create a new set with no entries.
    #[inline]
    pub fn new() -> Self {
        assert_eq!(
            crate::enums::verify_enum_type::<T, ()>().bitset_len,
            Self::BITSET_LEN
        );
        // We could just zero initialize the whole map
        let _assert_can_zero_init = <Self as crate::utils::Zeroable>::zeroed;
        // However, we initialize field-by-field in case that is somehow faster (skips padding?)
        EnumSet {
            // SAFETY: We know that that limbs is an array of integers, so can be zero-initialized
            limbs: unsafe { core::mem::zeroed() },
            len: 0,
            marker: PhantomData,
        }
    }

    const BITSET_LEN: usize = <T::BitSet as intid::array::Array<BitsetLimb>>::LEN;

    /// Create a new set with no entries, allocating memory on the heap instead of the stack.
    ///
    /// Using `Box::new(EnumSet::new())` could require moving the underlying table
    /// from the stack to the heap, as LLVM can struggle at eliminating copies.
    /// This method avoids that copy by always allocating in-place.
    #[inline]
    pub fn new_boxed() -> Box<Self> {
        assert_eq!(
            crate::enums::verify_enum_type::<T, ()>().bitset_len,
            Self::BITSET_LEN
        );
        crate::utils::Zeroable::zeroed_boxed()
    }

    #[inline]
    fn limbs(&self) -> &[BitsetLimb] {
        self.limbs.as_ref()
    }

    #[inline]
    fn limbs_mut(&mut self) -> &mut [BitsetLimb] {
        self.limbs.as_mut()
    }

    #[cold]
    fn index_overflow() -> ! {
        panic!(
            "An index for `{}` overflowed its claimed maximum",
            core::any::type_name::<T>()
        )
    }

    /// Break apart a key into its word index and bit index.
    ///
    /// Guarantees that the resulting word index will be in-bounds for the bitset.
    ///
    /// # Safety
    /// Relies on the unsafe guarantees of [`IntegerId::TRUSTED_RANGE`] if present.
    /// If this token is missing, this function makes no unsafe assumptions.
    #[inline]
    fn verified_index(key: &T) -> (usize, u32) {
        let index = intid::uint::checked_cast::<_, u32>(key.to_int()).unwrap_or_else(|| {
            if T::TRUSTED_RANGE.is_some() {
                // SAFETY: We have a TRUSTED_RANGE, so cannot overflow a u32
                unsafe { core::hint::unreachable_unchecked() }
            } else {
                Self::index_overflow()
            }
        });
        let (word_index, bit_index) = divmod_index(index);
        // if we don't have a TRUSTED_RANGE, we have to do a length check
        if T::TRUSTED_RANGE.is_none() && word_index >= Self::BITSET_LEN {
            Self::index_overflow();
        }
        (word_index, bit_index)
    }

    /// Inserts the specified element into the set,
    /// returning `true` if it was newly added and `false` if it was already present.
    ///
    /// Return value is consistent with [`HashSet::insert`].
    ///
    /// [`HashSet::insert`]: std::collections::HashSet::insert
    #[inline]
    pub fn insert(&mut self, value: T) -> bool {
        let (word_index, bit_index) = Self::verified_index(&value);
        // SAFETY: Validity of word index checked by verified_index
        let word = unsafe { self.limbs_mut().get_unchecked_mut(word_index) };
        let mask = bitmask_for(bit_index);
        let was_present = (mask & *word) != 0;
        *word |= mask;
        !was_present
    }

    /// Remove the specified value from the set,
    /// returning whether it was previously present.
    ///
    /// Return value is consistent with [`HashSet::remove`].
    ///
    /// [`HashSet::remove`]: std::collections::HashSet::insert
    #[inline]
    pub fn remove(&mut self, value: impl EquivalentId<T>) -> bool {
        let value = value.as_id();
        let (word_index, bit_index) = Self::verified_index(&value);
        // SAFETY: Validity of word index checked by verified_index
        let word = unsafe { self.limbs_mut().get_unchecked_mut(word_index) };
        let mask = bitmask_for(bit_index);
        let was_present = (mask & *word) != 0;
        *word &= !mask;
        was_present
    }

    /// Check if this set contains the specified value
    #[inline]
    pub fn contains(&self, value: impl EquivalentId<T>) -> bool {
        let (word_index, bit_index) = Self::verified_index(&value.as_id());
        // SAFETY: Validity of word index checked by verified_index
        let word = unsafe { self.limbs().get_unchecked(word_index) };
        (word & bitmask_for(bit_index)) != 0
    }

    /// Iterate over the values in this set.
    ///
    /// Guaranteed to be ordered by the integer value of the key.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            len: self.len as usize,
            handle: OnesIter::new(self.limbs().iter().copied()),
            marker: PhantomData,
        }
    }

    /// Clear the values in this set
    #[inline]
    pub fn clear(&mut self) {
        // SAFETY: Since the limbs are an array of integers,
        // they are safe to zero initialize
        unsafe {
            core::ptr::write_bytes(&mut self.limbs, 0, 1);
        }
        self.len = 0;
    }

    /// The number of entries in this set
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
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
        for (word_index, word) in self.limbs.as_mut().iter_mut().enumerate() {
            let (updated_word, word_removed) = retain_word(*word, |bit| {
                let id = (word_index * 32) + (bit as usize);
                // Safety: If present in the map, it is known to be valid
                let key = unsafe { T::from_int_unchecked(intid::uint::from_usize_wrapping(id)) };
                func(key)
            });
            *word = updated_word;
            self.len -= word_removed;
        }
    }
}
// SAFETY: We know that the bitset can be zero-initialized because it is an array of integers
// The only other field is the length, which can also be zero-initialized
unsafe impl<T: EnumId> crate::utils::Zeroable for EnumSet<T> {}

impl<T: EnumId> Default for EnumSet<T> {
    #[inline]
    fn default() -> Self {
        EnumSet::new()
    }
}
impl<T: EnumId> PartialEq for EnumSet<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.limbs() == other.limbs()
    }
}
impl<T: EnumId> Eq for EnumSet<T> {}
impl<T: EnumId> Debug for EnumSet<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}
impl<T: EnumId> Extend<T> for EnumSet<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}
impl<'a, T: EnumId> Extend<&'a T> for EnumSet<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}
impl<T: EnumId> FromIterator<T> for EnumSet<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

impl<'a, T: EnumId> FromIterator<&'a T> for EnumSet<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        iter.into_iter().copied().collect()
    }
}

impl<'a, T: EnumId + 'a> IntoIterator for &'a EnumSet<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<T: EnumId> IntoIterator for EnumSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len as usize,
            marker: PhantomData,
            handle: OnesIter::new(Array::into_iter(self.limbs)),
        }
    }
}

impl<'a, T: EnumId + 'a> Index<&'a T> for EnumSet<T> {
    type Output = bool;

    #[inline]
    fn index(&self, index: &'a T) -> &Self::Output {
        &self[*index]
    }
}
impl<T: EnumId> Index<T> for EnumSet<T> {
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
impl<T: EnumId + Hash> Hash for EnumSet<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.len());
        // guaranteed to be ordered by key
        for value in self {
            value.hash(state);
        }
    }
}
impl<T: EnumId + PartialOrd> PartialOrd for EnumSet<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}
impl<T: EnumId + Ord> Ord for EnumSet<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

/// An iterator over the values in an [`EnumSet`].
///
/// [PR #130]: https://github.com/petgraph/fixedbitset/pull/130
pub struct Iter<'a, T: EnumId> {
    len: usize,
    handle: OnesIter<BitsetLimb, core::iter::Copied<core::slice::Iter<'a, BitsetLimb>>>,
    marker: PhantomData<fn() -> T>,
}
impl_direct_set_iter!(Iter<'a, K: EnumId>);

/// An iterator over the values in an [`EnumSet`],
/// consuming ownership the set.
pub struct IntoIter<T: EnumId> {
    handle: OnesIter<BitsetLimb, <T::BitSet as Array<BitsetLimb>>::Iter>,
    len: usize,
    marker: PhantomData<T>,
}
impl_direct_set_iter!(IntoIter<K: EnumId>);

#[cfg(feature = "petgraph_0_8")]
impl<T: EnumId> petgraph_0_8::visit::VisitMap<T> for EnumSet<T> {
    #[inline]
    fn visit(&mut self, a: T) -> bool {
        self.insert(a)
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

/// Creates an [`EnumSet`] from a list of values
#[macro_export]
macro_rules! direct_enum_map {
    () => ($crate::enums::EnumSet::new());
    ($($value:expr),+ $(,)?) => ({
        let mut set = $crate::enums::EnumSet::new();
        $(set.insert($value);)*
        set
    });
}
