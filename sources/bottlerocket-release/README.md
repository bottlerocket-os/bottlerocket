# bottlerocket-release

Current version: 0.1.0

## Background

This library lets you get a BottlerocketRelease struct that represents the data in the standard os-release file, or another file you point to.
The VERSION_ID is returned as a semver::Version for convenience.

The information is pulled at runtime because build_id changes frequently and would cause unnecessary rebuilds.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
