idmap.rs
==========
Efficient maps of integer id keys to values.

A `DirectIdMap` is a strongly typed wrapper around a `Vec<Option<V>>` lookup table, and the `DirectIdSet` is a similar wrapper around a bitset.

Part of the [intid.rs](../README.md) set of crates.
