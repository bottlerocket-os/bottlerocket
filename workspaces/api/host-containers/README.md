# host-containers

Current version: 0.1.0

## Background

host-containers is a tool that queries the API for the currently enabled host containers and
ensures the relevant systemd service is enabled/started or disabled/stopped for each one depending
on its 'enabled' flag.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.