# thar-be-settings

Current version: 0.1.0

## Background

thar-be-settings is a simple configuration applier.

It is intended to be called from, and work directly with, the API server in Thar, the OS.
After a settings change, this program queries the API to determine which services and configuration files are affected by that change.
Once it has done so, it renders and rewrites the affected configuration files and restarts any affected services.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.