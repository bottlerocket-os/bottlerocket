# scalar-derive

Current version: 0.1.0


A convenience macro for Bottlerocket model types.

### Description

The `Scalar` macro can be used for strings or numbers that need to be validated in the Bottlerocket
model. It uses a trait, also named `Scalar`, to treat a `struct` as a thin wrapper around an
internal scalar type, or to treat an `enum` as "string-like".

For structs, the macro expects your inner scalar type to implement `Display`, `PartialEq`,
`Serialize` and `Deserialize`. It then implements these traits on the wrapper type by passing them
through to the inner type.

You are also expected to implement the `Validate` trait on your `Scalar` struct types (the wrapper,
not the inner type). This macro will call `<YourType as Validate>::validate(some_value)` when
implementing `YourType::new`.

Enums do not require a wrapping struct since it is assumed that the deserializtion of the enum
serves as validation. When using the `Scalar` macro on an enum it expects the enum to implement
`Serialize` and `Deserialize`. It also expects that your enum doesn't not contain any structures.
That is, your enum should be representable with a simple string and compatible with `serde_plain`.
The `Scalar` uses `serde_plain`, to implement `Display`, `FromStr` and `String` conversions for your
enum.

### Parameters

The macro can take the following input parameters when used with wrapper structs (in most cases you
will not need to use these; the defaults will "just work"):
- `as_ref_str: bool`: Set to `true` if need the macro to treat your inner type as a `String`.
   This will happen automatically if your inner type is named `String`.
- `inner`: The name of the field that holds your `inner` type. Defaults to `inner`.

## Examples

### Simple Usage

This is an example of a very common use-case in Bottlerocket. We have a string, but we want to
validate it. In this example we want to return an error if the string is "pineapple".

```rust
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;

// We create a struct with an inner type in a field named `inner`. We derive `Scalar`.
#[derive(Debug, PartialEq, Scalar)]
struct Pizza {
    inner: String
}

// We must implement the `Validate` trait for our type.
impl Validate for Pizza {
    fn validate<S: Into<String>>(input: S) -> Result<Pizza, ValidationError> {
        let input: String = input.into();
        if input == "pineapple" {
            Err(ValidationError::new("pineapple is gross on pizza"))
        } else {
            Ok(Self{ inner: input })
        }
    }
}

// The `Scalar` derive macro has made it so that we can use `Pizza` as if it were a `String`,
// but we know that the value has been validated.

let pizza = Pizza::new("pepperoni").unwrap();
// `pizza` behaves like a string!
assert!("pepperoni" == pizza);

let err = Pizza::new("pineapple");
// no that's gross!
assert!(err.is_err());
```

### Validating a Number

Here we use the Scalar macro with a numeric inner type. The inner value is constrained to be less
than 4.

```rust
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;

#[derive(Debug, PartialEq, Scalar)]
struct CatQuantity {
    inner: i32
}

impl Validate for CatQuantity {
    fn validate<I: Into<i32>>(input: I) -> Result<CatQuantity, ValidationError> {
        let input: i32 = input.into();
        if input > 4 {
            Err(ValidationError::new("that's too many cats"))
        } else {
            Ok(Self{ inner: input })
        }
    }
}

let cat_quantity = CatQuantity::new(2).unwrap();
// `cat_quantity` can be compared to a i32
assert!(2 == cat_quantity);

let err = CatQuantity::new(23);
// no that's too many!
assert!(err.is_err());
```

### Using Macro Parameters

In this example we will show how to use the parameters `as_ref_str` and `inner`. This example is
contrived, but demonstrates how to pass parameters to the derive macro.

```rust
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct WeirdType;

// We need `AsRef<str>` to treat it like a string with `as_ref_str`.
impl AsRef<str> for WeirdType {
    fn as_ref(&self) -> &str {
        "i'm a weird type"
    }
}

// We also need `From<&str>` to treat it like a string with `as_ref_str`.
impl From<&str> for WeirdType {
    fn from(_: &str) -> WeirdType {
        WeirdType
    }
}

// We also need `Deref` to treat it like a string with `as_ref_str`.
impl std::ops::Deref for WeirdType {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

// We also need `Borrow` to treat it like a string with `as_ref_str`.
impl core::borrow::Borrow<str> for WeirdType {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

// We also need `Display` to work.
impl std::fmt::Display for WeirdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

// Here we create a struct that doesn't use the default name for the inner field. We also tell
// the `Scalar` macro to treat the inner type like a string.
#[derive(Debug, PartialEq, Scalar)]
#[scalar(as_ref_str = true, inner = "some_field")]
struct MyWrapper {
    some_field: WeirdType,
}

impl Validate for MyWrapper {
    fn validate<T: Into<WeirdType>>(input: T) -> Result<MyWrapper, ValidationError> {
        Ok(Self{ some_field: input.into() })
    }
}

let value = MyWrapper::new(WeirdType).unwrap();
// This type can be compared with &str because we specified `as_ref_str = true`.
assert!("i'm a weird type" == value);
```

### Enums

When used with an enum, `Scalar` implements a few `String` conversions such as `Display` and
`FromStr`.

```rust
use scalar_derive::Scalar;
use serde::{Serialize, Deserialize};
use std::convert::TryInto;

#[derive(Debug, PartialEq, Serialize, Deserialize, Scalar)]
#[serde(rename_all = "snake_case")]
enum Color {
    Red,
    Green,
    Blue,
}


let value = Color::Blue;
let to_string_val = value.to_string();
assert_eq!(to_string_val, "blue");
let from_str_val: Color = "blue".parse().unwrap();
assert_eq!(value, from_str_val);
let into_string_val: String = value.into();
assert_eq!(into_string_val, "blue");
let try_from_value: Color = "blue".try_into().unwrap();
assert_eq!(Color::Blue, try_from_value);
```


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
