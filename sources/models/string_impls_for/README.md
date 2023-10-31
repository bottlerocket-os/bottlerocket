# string_impls_for

Current version: 0.1.0


A convenience macro for Bottlerocket model types.

### Description

The `string_impls_for` is meant to assist in representing string-like types that require additional
validation when deserialized. These types should be modeled as a struct with a single string field
called `inner`.

The user of the macro implements `TryFrom<&str>`, and the macro will derive implementations for
`TryFrom<String>`, `serde::Deserialize`, `serde::Serialize`, `Deref`, `Borrow<String>`,
`Borrow<str>`, `AsRef<str>`, `Display`, `Into<String>`, `PartialEq<str>`, `PartialEq<&str>`, and
`PartialEq<String>`.

### Example

Consider a model which we want to contain a string name for a vegetable, you could implement
something like so:

```rust
use string_impls_for::string_impls_for;

#[derive(Debug, PartialEq, Eq)]
struct Vegetable {
    inner: String,
}

impl TryFrom<&str> for Vegetable {
    type Error = &'static str;

    fn try_from(input: &str) -> std::result::Result<Self, Self::Error> {
        if !["cucumber", "radish", "leek"].contains(&input) {
            return Err("Vegetable name must be one of cucumber, radish, or leek");
        }
        Ok(Vegetable { inner: input.to_string() })
    }
}

string_impls_for!(Vegetable, "Vegetable");


let cucumber = Vegetable::try_from("cucumber").unwrap();
assert_eq!(cucumber.to_string(), "cucumber");
```
!

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
