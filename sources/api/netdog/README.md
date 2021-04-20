# netdog

Current version: 0.1.0

## Introduction

netdog is a small helper program for wicked, to apply network settings received from DHCP.  It also
contains a subcommand `node-ip` that returns the node's current IP address in JSON format; this
subcommand is intended for use as a settings generator.

It generates `/etc/resolv.conf`, sets the hostname, and persists the current IP to file.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.