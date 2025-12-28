//! Utilities for bitsets typesl.
use core::ops::{BitAnd, BitAndAssign, Not, Shl};
use intid::uint::{one, trailing_zeros, zero};
use intid::UnsignedPrimInt;

pub trait BitsetWord:
    UnsignedPrimInt
    + Shl<u32, Output = Self>
    + BitAnd<Output = Self>
    + BitAndAssign
    + Not<Output = Self>
{
}
impl BitsetWord for u32 {}
impl BitsetWord for usize {}
impl BitsetWord for u64 {}

pub mod ones;

#[inline]
pub fn retain_word<W: BitsetWord, F: FnMut(u32) -> bool>(
    original_word: W,
    mut func: F,
) -> (W, u32) {
    let mut remaining = original_word;
    let mut result = original_word;
    let mut removed = 0;
    while remaining != zero() {
        let bit = trailing_zeros(remaining);
        let mask: W = one::<W>() << bit;
        debug_assert_ne!(result & mask, zero());
        if !func(bit) {
            result &= !mask;
            removed += 1;
        }
        remaining &= !mask;
    }
    debug_assert!(removed <= 32);
    (result, removed)
}
