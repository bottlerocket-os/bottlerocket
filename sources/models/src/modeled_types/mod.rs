//! This module contains data types that can be used in the model when special input/output
//! (ser/de) behavior is desired.  For example, the ValidBase64 type can be used for a model field
//! when we don't even want to accept an API call with invalid base64 data.

// The pattern in this module is to make a struct and implement TryFrom<&str> with code that does
// necessary checks and returns the struct.  Other traits that treat the struct like a string can
// be implemented for you with the string_impls_for macro.

pub mod error {
    use regex::Regex;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Can't create SingleLineString containing line terminator"))]
        StringContainsLineTerminator,

        #[snafu(display("Invalid base64 input: {}", source))]
        InvalidBase64 { source: base64::DecodeError },

        #[snafu(display(
            "Identifiers may only contain ASCII alphanumerics plus hyphens, received '{}'",
            input
        ))]
        InvalidIdentifier { input: String },

        #[snafu(display("Given invalid URL '{}'", input))]
        InvalidUrl { input: String },

        #[snafu(display("Invalid version string '{}'", input))]
        InvalidVersion { input: String },

        #[snafu(display("{} must match '{}', given: {}", thing, pattern, input))]
        Pattern {
            thing: String,
            pattern: Regex,
            input: String,
        },

        // Some regexes are too big to usefully display in an error.
        #[snafu(display("{} given invalid input: {}", thing, input))]
        BigPattern { thing: String, input: String },

        #[snafu(display("Given invalid cluster name '{}': {}", name, msg))]
        InvalidClusterName { name: String, msg: String },
    }
}

/// Helper macro for implementing the common string-like traits for a modeled type.
/// Pass the name of the type, and the name of the type in quotes (to be used in string error
/// messages, etc.).
macro_rules! string_impls_for {
    ($for:ident, $for_str:expr) => {
        impl TryFrom<String> for $for {
            type Error = $crate::modeled_types::error::Error;

            fn try_from(input: String) -> Result<Self, Self::Error> {
                Self::try_from(input.as_ref())
            }
        }

        impl<'de> Deserialize<'de> for $for {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let original = String::deserialize(deserializer)?;
                Self::try_from(original).map_err(|e| {
                    D::Error::custom(format!("Unable to deserialize into {}: {}", $for_str, e))
                })
            }
        }

        /// We want to serialize the original string back out, not our structure, which is just there to
        /// force validation.
        impl Serialize for $for {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.inner)
            }
        }

        impl Deref for $for {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl Borrow<String> for $for {
            fn borrow(&self) -> &String {
                &self.inner
            }
        }

        impl Borrow<str> for $for {
            fn borrow(&self) -> &str {
                &self.inner
            }
        }

        impl AsRef<str> for $for {
            fn as_ref(&self) -> &str {
                &self.inner
            }
        }

        impl fmt::Display for $for {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.inner)
            }
        }

        impl From<$for> for String {
            fn from(x: $for) -> Self {
                x.inner
            }
        }
    };
}

// Must be after macro definition
mod kubernetes;
mod shared;

pub use kubernetes::*;
pub use shared::*;
