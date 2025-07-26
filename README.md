idmap.rs [![Crates.io](https://img.shields.io/crates/v/idmap.svg)](https://crates.io/crates/idmap) [![Documentation](https://docs.rs/idmap/badge.svg)](https://docs.rs/idmap)
==========
Efficient maps of integer id keys to values.

A `DirectIdMap` is a strongly typed wrapper around a `Vec<Option<V>>` lookup table, and the `DirectIdSet` is a similar wrapper around a bitset.

The `intid-allocator` crate provides a way to efficiently allocate and free integer ids,
which reduces the memory needed to use these lookup tables.

This is based on an `IntegerId` trait defined in the `intid` crate.
The trait can be derived for newtype structs and C-like enums using the `intid-derive` procedural macro.

## License
Licensed under either the [Apache 2.0 License](./LICENSE-APACHE.txt) or [MIT License](./LICENSE-MIT.txt) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
