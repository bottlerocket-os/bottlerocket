# prairiedog

Current version: 0.1.0

  prairiedog is a tool to provide kdump support in Bottlerocket. It performs three operations:

  - _digs_ to find the active boot partition and mounts it in /boot
  - loads the crash kernel from /boot
  - creates memory dumps when the kernel panics

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.