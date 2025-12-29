//! Efficient maps of integer ids to values.
#![cfg_attr(feature = "nightly", feature(trusted_len))]
#![deny(missing_docs, deprecated_safe_2024)]
#![cfg_attr(not(doc), no_std)]
#![allow(
    // triggers for `impl EquivalentId<...>`
    clippy::needless_pass_by_value
)]

extern crate alloc;

pub mod direct;
pub mod enums;
mod utils;

pub extern crate intid;

pub use self::direct::{DirectIdMap, DirectIdSet};
pub use self::enums::{EnumMap, EnumSet};
