intid.rs [![Crates.io](https://img.shields.io/crates/v/intid.svg)](https://crates.io/crates/intid) [![Documentation](https://docs.rs/intid/badge.svg)](https://docs.rs/idntid)
==========
A set of libraries and data structures for operating on integer-like ids. Supports the common rust pattern of wrapping integer ids in newtypes.

The [idmap](idmap/README.md) crate provides a strongly typed wrapper around a `Vec<Option<V>>` lookup table (`DirectIdMap`) and a similar wrapper around a bitset (`DirectIdSet`).

The `intid-allocator` crate provides a way to efficiently track and reuse unused ids, minimizing a memory needed for lookup tables.

The `intid` crate defines the foundational `IntegerId` trait. Enabling the `derive` feature adds a derive macro.

## License
Licensed under either the [Apache 2.0 License](./LICENSE-APACHE.txt) or [MIT License](./LICENSE-MIT.txt) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
