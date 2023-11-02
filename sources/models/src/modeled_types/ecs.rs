use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
// Just need serde's Error in scope to get its trait methods
use super::error::{self, big_pattern_error};
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;
use snafu::ensure;
use std::convert::TryFrom;
use string_impls_for::string_impls_for;

/// ECSAttributeKey represents a string that contains a valid ECS attribute key.  It stores
/// the original string and makes it accessible through standard traits.
// https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_Attribute.html
#[derive(Debug, Clone, Eq, PartialEq, Hash, Scalar)]
pub struct ECSAttributeKey {
    inner: String,
}

// The name of the attribute. The name must contain between 1 and 128
// characters and name may contain letters (uppercase and lowercase), numbers,
// hyphens, underscores, forward slashes, back slashes, or periods.
lazy_static! {
    pub(crate) static ref ECS_ATTRIBUTE_KEY: Regex = Regex::new(
        r"(?x)^
          [a-zA-Z0-9._/-]{1,128}
          $"
    )
    .unwrap();
}

impl Validate for ECSAttributeKey {
    fn validate<S: Into<String>>(input: S) -> std::result::Result<Self, ValidationError> {
        let input = input.into();
        require!(
            ECS_ATTRIBUTE_KEY.is_match(&input),
            big_pattern_error("ECS attribute key", &input)
        );
        Ok(ECSAttributeKey { inner: input })
    }
}

#[cfg(test)]
mod test_ecs_attribute_key {
    use super::ECSAttributeKey;
    use std::convert::TryFrom;

    #[test]
    fn good_keys() {
        for key in &[
            "a",
            "alphabetical",
            "1234567890",
            "with-dash",
            "have.period/slash",
            "have_underscore_too",
            &"a".repeat(128),
            ".leadingperiod",
            "trailingperiod.",
        ] {
            ECSAttributeKey::try_from(*key).unwrap();
        }
    }

    #[test]
    fn bad_keys() {
        for key in &[
            "",
            &"a".repeat(129),
            "@",
            "$",
            "%",
            ":",
            "no spaces allowed",
        ] {
            ECSAttributeKey::try_from(*key).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ECSAttributeValue represents a string that contains a valid ECS attribute value.  It stores
/// the original string and makes it accessible through standard traits.
// https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_Attribute.html
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ECSAttributeValue {
    inner: String,
}

// The value of the attribute. The value must contain between 1 and 128
// characters and may contain letters (uppercase and lowercase), numbers,
// hyphens, underscores, periods, at signs (@), forward slashes, back slashes,
// colons, or spaces. The value cannot contain any leading or trailing
// whitespace.
lazy_static! {
    pub(crate) static ref ECS_ATTRIBUTE_VALUE: Regex = Regex::new(
        r"(?x)^
          [a-zA-Z0-9.@:_/\\-] # at least one non-space
          (
            ([a-zA-Z0-9.@:\ _/\\-]{0,126})? # spaces allowed
            [a-zA-Z0-9.@:_/\\-] # end with non-space
          )?
          $"
    )
    .unwrap();
}

impl TryFrom<&str> for ECSAttributeValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            ECS_ATTRIBUTE_VALUE.is_match(input),
            error::BigPatternSnafu {
                thing: "ECS attribute value",
                input
            }
        );
        Ok(ECSAttributeValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ECSAttributeValue, "ECSAttributeValue");

#[cfg(test)]
mod test_ecs_attribute_value {
    use super::ECSAttributeValue;
    use std::convert::TryFrom;

    #[test]
    fn good_vals() {
        for val in &[
            "a",
            "alphabetical",
            "1234567890",
            "with-dash",
            "have.period/slash",
            "have/slash\\backslash",
            "have_underscore_too",
            "with spaces in between",
            &"a".repeat(128),
            ".leadingperiod",
            "trailingperiod.",
            "@ and : allowed too",
            "\\",
            "\\ \\",
        ] {
            ECSAttributeValue::try_from(*val).unwrap();
        }
    }

    #[test]
    fn bad_vals() {
        for val in &[
            "",
            &"a".repeat(129),
            "$",
            "%",
            " leading space",
            "trailing space ",
        ] {
            ECSAttributeValue::try_from(*val).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ECSAgentLogLevel represents a string that contains a valid ECS log level for the ECS agent.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Scalar)]
#[serde(rename_all = "lowercase")]
pub enum ECSAgentLogLevel {
    Debug,
    Info,
    Warn,
    // Rename needed due to #[deny(ambiguous_associated_items)]
    // see https://github.com/rust-lang/rust/issues/57644
    #[serde(rename = "error")]
    ErrorLevel,
    Crit,
}

#[cfg(test)]
mod test_ecs_agent_log_level {
    use super::ECSAgentLogLevel;
    use std::convert::TryFrom;

    #[test]
    fn good_vals() {
        for val in &["debug", "info", "warn"] {
            ECSAgentLogLevel::try_from(*val).unwrap();
        }
    }

    #[test]
    fn bad_vals() {
        for val in &["", "warning", "errors", " "] {
            ECSAgentLogLevel::try_from(*val).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ECSAgentImagePullBehavior represents a valid ECS Image Pull Behavior for the ECS agent.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Scalar)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum ECSAgentImagePullBehavior {
    Default = 0,
    Always,
    Once,
    PreferCached,
}

impl ECSAgentImagePullBehavior {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

#[cfg(test)]
mod test_ecs_agent_image_pull_behavior {
    use super::ECSAgentImagePullBehavior;
    use std::convert::TryFrom;

    #[test]
    fn good_vals() {
        for val in &["default", "always", "once", "prefer-cached"] {
            ECSAgentImagePullBehavior::try_from(*val).unwrap();
        }
    }

    #[test]
    fn bad_vals() {
        for val in &["", "tomorrow", "never", " "] {
            ECSAgentImagePullBehavior::try_from(*val).unwrap_err();
        }
    }
}

/// ECSDurationValue represents a string that contains a valid ECS duration value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ECSDurationValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref ECS_DURATION_VALUE: Regex =
        Regex::new(r"^(([0-9]+\.)?[0-9]+h)?(([0-9]+\.)?[0-9]+m)?(([0-9]+\.)?[0-9]+s)?(([0-9]+\.)?[0-9]+ms)?(([0-9]+\.)?[0-9]+(u|µ)s)?(([0-9]+\.)?[0-9]+ns)?$").unwrap();
}

impl TryFrom<&str> for ECSDurationValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            !input.is_empty() && ECS_DURATION_VALUE.is_match(input),
            error::InvalidECSDurationValueSnafu { input }
        );
        Ok(ECSDurationValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ECSDurationValue, "ECSDurationValue");

#[cfg(test)]
mod test_ecs_duration_value {
    use super::ECSDurationValue;
    use std::convert::TryFrom;

    #[test]
    fn valid_values() {
        for ok in &[
            "99s",
            "20m",
            "1h",
            "1h2m3s",
            "4m5s",
            "2h3s",
            "1.5h3.5m",
            "1ms1us1ns",
            "1s1µs1ns",
        ] {
            ECSDurationValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn invalid_values() {
        for err in &[
            "",
            "100",
            "...3ms",
            "1..5s",
            "ten second",
            "1m2h",
            "1y2w",
            &"a".repeat(23),
        ] {
            ECSDurationValue::try_from(*err).unwrap_err();
        }
    }
}
