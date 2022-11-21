# storewolf

Current version: 0.1.0

## Introduction

storewolf creates the filesystem datastore used by the API system.

It creates the datastore at a provided path and populates any default settings, as given in the
TOML files of the current variant's `defaults.d` directory, unless the datastore already exists.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
