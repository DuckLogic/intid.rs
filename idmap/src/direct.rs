//! Direct maps and set, which take storage proportional to the maximum id.
//!
//! This is roughly equivalent to a `Vec<Option<T>>` for the map and bitset for the set.

pub(crate) mod macros;
pub mod map;
#[cfg(feature = "serde")]
mod serde;
pub mod set;

pub use self::map::DirectIdMap;
pub use self::set::DirectIdSet;
use intid::primint;

/// Panic indicating that an id would exhaust available memory.
#[inline(never)]
#[track_caller]
#[cold]
fn oom_id(id: impl primint::UnsignedPrimInt) -> ! {
    panic!(
        "Storing id would exhaust memory: {}",
        primint::fmt::debug_desc(id),
    )
}
