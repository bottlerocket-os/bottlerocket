# parse-datetime

Current version: 0.1.0

## Background

This library parses a `DateTime<Utc>` from a string.

The string can be:

* an `RFC3339` formatted date / time
* a string with the form `"[in] <unsigned integer> <unit(s)>"` where 'in' is optional
   * `<unsigned integer>` may be any unsigned integer and
   * `<unit(s)>` may be either the singular or plural form of the following: `hour | hours`, `day | days`, `week | weeks`

Examples:

* `"in 1 hour"`
* `"in 2 hours"`
* `"in 6 days"`
* `"in 2 weeks"`
* `"1 hour"`
* `"7 days"`

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
