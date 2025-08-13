//! Direct maps and set, which take storage proportional to the maximum id.
//!
//! This is roughly equivalent to a `Vec<Option<T>>` for the map and bitset for the set.

pub mod map;
#[cfg(feature = "serde")]
mod serde;
pub mod set;

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
