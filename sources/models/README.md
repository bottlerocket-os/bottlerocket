# models

Current version: 0.1.0

## API models

Bottlerocket has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

The model here defines a top-level `Settings` structure, and delegates the actual implementation to a ["settings plugin"](https://github.com/bottlerocket-os/bottlerocket-settings-sdk/tree/develop/bottlerocket-settings-plugin).
Settings plugin are written in Rust as a "cdylib" crate, and loaded at runtime.

Each settings plugin must define its own private `Settings` structure.
It can use pre-defined structures inside, or custom ones as needed.

`apiserver::datastore` offers serialization and deserialization modules that make it easy to map between Rust types and the data store, and thus, all inputs and outputs are type-checked.

At the field level, standard Rust types can be used, or ["modeled types"](https://github.com/bottlerocket-os/bottlerocket-settings-sdk/tree/develop/bottlerocket-settings-models/modeled-types) that add input validation.

The `#[model]` attribute on Settings and its sub-structs reduces duplication and adds some required metadata; see [its docs](model-derive/) for details.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
