# retry-read

Current version: 0.1.0

This library provides a `RetryRead` trait with a `retry_read` function that's available for any
`Read` type.  `retry_read` retries after standard interruptions (unlike `read`) but also
returns the number of bytes read (unlike `read_exact`), and without needing to read to the end
of the input (unlike `read_to_end` and `read_to_string`).

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
