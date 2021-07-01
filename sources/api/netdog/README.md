# netdog

Current version: 0.1.0

## Introduction

netdog is a small helper program for wicked, to apply network settings received from DHCP.  It
generates `/etc/resolv.conf`, generates and sets the hostname, and persists the current IP to file.

It contains two subcommands meant for use as settings generators:
* `node-ip`: returns the node's current IP address in JSON format
* `generate-hostname`: returns the node's hostname in JSON format (it is the resolved IP or the IP
  in format "ip-x-x-x-x" if resolving fails)

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.