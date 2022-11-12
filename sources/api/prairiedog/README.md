# prairiedog

Current version: 0.1.0

  prairiedog is a tool for providing kernel boot related support in Bottlerocket.

It does the following:
  - _digs_ to find the active boot partition and mounts it in /boot
  - loads the crash kernel from /boot
  - creates memory dumps when the kernel panics
  - generates kernel boot config from settings
  - generates settings from the existing kernel boot config file


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
