# moondog

Current version: 0.1.0

## Introduction

moondog is a minimal user data agent.

It accepts TOML-formatted settings from a user data provider such as an instance metadata service.
These are sent to a known Thar API server endpoint, then committed.

Currently, Amazon EC2 user data support is implemented.
User data can also be retrieved from a file for testing.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.