# early-boot-config

Current version: 0.1.0

## Introduction

early-boot-config sends user data to the Bottlerocket API.

Variants include their required user data provider binaries via packages.  early-boot-config discovers these binaries at runtime in `/usr/libexec/early-boot-config/data-providers.d` and runs them in order, sending any user data found to the API.

User data provider binaries each implement the ability to obtain user data from a single source.  Sources include local files, AWS Instance Metadata Service (IMDS), among others.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
