# imdsclient

Current version: 0.1.0

The imdsclient library provides high-level methods to interact with the AWS Instance Metadata Service.
The high-level methods provided are [`fetch_dynamic`], [`fetch_metadata`], and [`fetch_userdata`].

For more control, and to query IMDS without high-level wrappers, there is also a [`fetch_imds`] method.
This method is useful for specifying things like a pinned date for the IMDS schema version.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.