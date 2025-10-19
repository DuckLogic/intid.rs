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

pub use self::map::EnumMap;
