//! Maps and sets for [`EnumId`] types, which do not perform any allocation.
//!
//! All of these collections store their data inline without any allocation,
//! so you may want to [`Box`] them to reduce the stack space that is used.
//! Each collection has special constructors to do this in place without moving.
//!
//! [`EnumId`]: intid::EnumId

pub mod map;
#[cfg(feature = "serde")]
mod serde;
pub mod set;

use intid::array::BitsetLimb;
use intid::{uint, EnumId};

pub use self::map::EnumMap;
pub use self::set::EnumSet;

pub(crate) struct VerifiedEnumInfo {
    pub array_len: usize,
    pub bitset_len: usize,
}

/// Verify the specified [`EnumId`] type sensibly implements
/// [`EnumId::Array`], [`EnumId::BitSet`] and [`EnumId::COUNT`].
///
/// Needs to be passed a specific value type `V` to check the length of [`EnumId::Array`]
///
/// Unless this function ultimately panics,
/// all of these checks will be constant-folded away.
#[inline]
#[track_caller]
pub(crate) fn verify_enum_type<K: EnumId, V>() -> VerifiedEnumInfo {
    let type_name = core::any::type_name::<K>();
    let expected_array_len = u32::from(match K::MAX_ID_INT {
        None => 0,
        Some(max_id) => uint::checked_cast::<K::Int, u16>(max_id)
            .and_then(|x| x.checked_add(1))
            .unwrap_or_else(|| panic!("max_id for {type_name} overflows a u16")),
    });
    let actual_array_len = <K::Array<V> as intid::array::Array<V>>::LEN;
    assert_eq!(
        expected_array_len as u64, actual_array_len as u64,
        "Unexpected array length for {type_name}"
    );
    assert!(
        K::COUNT <= expected_array_len,
        "Unexpected EnumId::COUNT = {count} > {expected_array_len} for {type_name}",
        count = K::COUNT
    );
    let actual_bitset_len = <K::BitSet as intid::array::Array<BitsetLimb>>::LEN;
    let expected_bitset_len = (expected_array_len + (BitsetLimb::BITS - 1)) / BitsetLimb::BITS;
    assert_eq!(
        actual_bitset_len as u64, expected_bitset_len as u64,
        "Unexpected bitset length for {type_name} (array_len = {expected_array_len})"
    );
    VerifiedEnumInfo {
        array_len: expected_array_len as usize,
        bitset_len: expected_bitset_len as usize,
    }
}
