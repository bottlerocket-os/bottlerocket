# settings-committer

Current version: 0.1.0

## Introduction

settings-committer can be called to commit any pending settings in the API.
It logs any pending settings, then commits them to live.

This is typically run during startup as a pre-exec command by any services that depend on settings
changes from previous services.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.