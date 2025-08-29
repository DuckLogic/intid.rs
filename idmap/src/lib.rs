//! Efficient maps of integer ids to values.
#![cfg_attr(feature = "nightly", feature(trusted_len))]
#![deny(missing_docs, deprecated_safe_2024)]
#![cfg_attr(not(doc), no_std)]

extern crate alloc;

pub mod direct;

pub extern crate intid;

pub use self::direct::{DirectIdMap, DirectIdSet};
