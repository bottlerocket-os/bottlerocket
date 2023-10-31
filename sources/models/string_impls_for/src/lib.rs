/*!

A convenience macro for Bottlerocket model types.

## Description

The `string_impls_for` is meant to assist in representing string-like types that require additional
validation when deserialized. These types should be modeled as a struct with a single string field
called `inner`.

The user of the macro implements `TryFrom<&str>`, and the macro will derive implementations for
`TryFrom<String>`, `serde::Deserialize`, `serde::Serialize`, `Deref`, `Borrow<String>`,
`Borrow<str>`, `AsRef<str>`, `Display`, `Into<String>`, `PartialEq<str>`, `PartialEq<&str>`, and
`PartialEq<String>`.

## Example

Consider a model which we want to contain a string name for a vegetable, you could implement
something like so:

```
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
!*/
#[macro_export]
/// Helper macro for implementing the common string-like traits for a modeled type.
/// Pass the name of the type, and the name of the type in quotes (to be used in string error
/// messages, etc.).
macro_rules! string_impls_for {
    ($for:ident, $for_str:expr) => {
        impl TryFrom<String> for $for {
            type Error = <Self as TryFrom<&'static str>>::Error;

            fn try_from(input: String) -> std::result::Result<Self, Self::Error> {
                Self::try_from(input.as_ref())
            }
        }

        impl<'de> serde::Deserialize<'de> for $for {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let original = String::deserialize(deserializer)?;
                Self::try_from(original).map_err(|e| {
                    <D::Error as serde::de::Error>::custom(format!(
                        "Unable to deserialize into {}: {}",
                        $for_str, e
                    ))
                })
            }
        }

        /// We want to serialize the original string back out, not our structure, which is just there to
        /// force validation.
        impl serde::Serialize for $for {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.inner)
            }
        }

        impl std::ops::Deref for $for {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::borrow::Borrow<String> for $for {
            fn borrow(&self) -> &String {
                &self.inner
            }
        }

        impl std::borrow::Borrow<str> for $for {
            fn borrow(&self) -> &str {
                &self.inner
            }
        }

        impl AsRef<str> for $for {
            fn as_ref(&self) -> &str {
                &self.inner
            }
        }

        impl std::fmt::Display for $for {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.inner)
            }
        }

        impl From<$for> for String {
            fn from(x: $for) -> Self {
                x.inner
            }
        }

        impl PartialEq<str> for $for {
            fn eq(&self, other: &str) -> bool {
                &self.inner == other
            }
        }

        impl PartialEq<String> for $for {
            fn eq(&self, other: &String) -> bool {
                &self.inner == other
            }
        }

        impl PartialEq<&str> for $for {
            fn eq(&self, other: &&str) -> bool {
                &self.inner == other
            }
        }
    };
}
