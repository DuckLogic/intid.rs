//! Direct maps and set, which take storage proportional to the maximum id.
//!
//! This is roughly equivalent to a `Vec<Option<T>>` for the map and bitset for the set.

pub mod map;
#[cfg(feature = "serde")]
mod serde;
pub mod set;

use intid::IntegerId;
pub use self::map::DirectIdMap;
pub use self::set::DirectIdSet;
use intid::uint::UnsignedPrimInt;

/// Panic indicating that an id would exhaust available memory.
#[inline(never)]
#[track_caller]
#[cold]
fn oom_id(id: impl UnsignedPrimInt) -> ! {
    panic!(
        "Storing id would exhaust memory: {}",
        intid::uint::debug_desc(id),
    )
}

/// Private trait used to
trait IntegerIdExt: IntegerId {
    /// Invokes [`Self::from_int_unchecked`] with a `usize` argument.
    #[inline]
    unsafe fn from_usize_unchecked(index: usize) -> Self {
        Self::from_int_unchecked(intid::uint::from_usize_wrapping(index))
    }
    #[inline]
    fn to_usize_checked(&self) -> Option<usize> {
        intid::uint::to_usize_checked(self.to_int())
    }
}
impl<K: IntegerId> IntegerIdExt for K {}
