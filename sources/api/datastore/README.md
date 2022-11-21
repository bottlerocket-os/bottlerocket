# datastore

Current version: 0.1.0

## Background

A 'data store' in Bottlerocket is responsible for storing key/value pairs and metadata about those pairs, with the ability to commit changes in transactions.

For more detail about their usage, see [apiserver](../apiserver).

## Library

This library provides a trait defining the exact requirements, along with basic implementations for filesystem and memory data stores.

There's also a common error type and some methods that implementations of DataStore should generally share, like scalar serialization.

We represent scalars -- the actual values stored under a datastore key -- using JSON, just to have a convenient human-readable form.
(TOML doesn't allow raw scalars.  The JSON spec doesn't seem to either, but this works, and the format is so simple for scalars that it could be easily swapped out if needed.)

## Serialization and deserialization

The `serialization` module provides code to serialize Rust types into a mapping of datastore-acceptable keys (a.b.c) and values.

The `deserialization` module provides code to deserialize datastore-acceptable keys (a.b.c) and values into Rust types.

## Current limitations

* The user (e.g. apiserver) needs to handle locking.
* There's no support for rolling back transactions.
* The `serialization` module can't handle complex types under lists; it assumes lists can be serialized as scalars.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
