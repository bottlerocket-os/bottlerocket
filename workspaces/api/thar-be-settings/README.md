# thar-be-settings

Current version: 0.1.0

## Background

thar-be-settings is a simple configuration applier.

In the normal ("specific keys") mode, it's intended to be called by the Thar API server after a
commit.  It's told the keys that changed, and then queries the API to determine which services and
configuration files are affected by that change.  It then renders and rewrites the affected
configuration files and restarts any affected services.

In the standalone ("all keys") mode, it queries the API for all services and configuration files,
then renders and rewrites all configuration files and restarts all services.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.