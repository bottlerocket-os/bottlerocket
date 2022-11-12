# model-derive

Current version: 0.1.0

## Overview

This module provides a attribute-style procedural macro, `model`, that makes sure a struct is
ready to be used as an API model.

The goal is to reduce cognitive overhead when reading models.
We do this by automatically specifying required attributes on structs and fields.

Several arguments are available to override default behavior; see below.

## Changes it makes

### Visibility

All types must be public, so `pub` is added.
Override this (at a per-struct or per-field level) by specifying your own visibility.

### Derives

All structs must serde-`Serializable` and -`Deserializable`, and comparable via `PartialEq`.
`Debug` is added for convenience.
`Default` can also be added by specifying the argument `impl_default = true`.

### Serde

Structs have a `#[serde(...)]` attribute added to deny unknown fields and rename fields to kebab-case.
The struct can be renamed (for ser/de purposes) by specifying the argument `rename = "bla"`.

Fields have a `#[serde(...)]` attribute added to skip `Option` fields that are `None`.
This is because we accept updates in the API that are structured the same way as the model, but we don't want to require users to specify fields they aren't changing.
This can be disabled by specifying the argument `add_option = false`.

### Option

Fields are all wrapped in `Option<...>`.
Similar to the `serde` attribute added to fields, this is because we don't want users to have to specify fields they aren't changing, and can be disabled the same way, by specifying `add_option = false`.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
