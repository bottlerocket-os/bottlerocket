# moondog

Current version: 0.1.0

## Introduction

moondog is a minimal userdata agent.

It accepts TOML-formatted settings from a userdata provider such as an instance metadata service.
These are sent to a known Thar-API-server endpoint, then committed.

Currently, AWS userdata support is implemented.
Userdata can also be retrieved from a file for testing.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.