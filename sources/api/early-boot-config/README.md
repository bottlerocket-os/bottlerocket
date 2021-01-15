# early-boot-config

Current version: 0.1.0

## Introduction

early-boot-config sends provider-specific platform data to the Bottlerocket API.

For most providers this means configuration from user data and platform metadata, taken from
something like an instance metadata service.

This program is conditionally compiled to include the appropriate data providers for a specific
Bottlerocket platform.  Currently, Amazon EC2 is supported through the IMDSv2 HTTP API.  For
development variants, data will be taken from files in /etc/early-boot-config instead, if
available, for testing purposes.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.