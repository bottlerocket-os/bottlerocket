# settings-committer

Current version: 0.1.0

## Introduction

settings-committer can be called to commit a pending transaction in the API.
It logs any pending settings, then commits them to live.

By default, it commits the 'bottlerocket-launch' transaction, which is used to organize boot-time services - this program is typically run as a pre-exec command by any services that depend on settings changes from previous services.

The `--transaction` argument can be used to specify another transaction.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
