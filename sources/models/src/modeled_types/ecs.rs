use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use super::error;
use serde::de::Error as _;
use snafu::{ensure, ResultExt};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// ECSAttributeKey represents a string that contains a valid ECS attribute key.  It stores
/// the original string and makes it accessible through standard traits.
// https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_Attribute.html
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

impl FromStr for ECSAttributeKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            ECS_ATTRIBUTE_KEY.is_match(input),
            error::BigPatternSnafu {
                thing: "ECS attribute key",
                input
            }
        );
        Ok(ECSAttributeKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ECSAttributeKey, "ECSAttributeKey");

#[cfg(test)]
mod test_ecs_attribute_key {
    use super::ECSAttributeKey;

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
            key.parse::<ECSAttributeKey>().unwrap();
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
            key.parse::<ECSAttributeKey>().unwrap_err();
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

impl FromStr for ECSAttributeValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
            val.parse::<ECSAttributeValue>().unwrap();
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
            val.parse::<ECSAttributeValue>().unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ECSAgentLogLevel represents a string that contains a valid ECS log level for the ECS agent.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ECSAgentLogLevel {
    inner: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ECSLogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Crit,
}

string_impls_for!(ECSAgentLogLevel, "ECSAgentLogLevel");

impl FromStr for ECSAgentLogLevel {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        serde_plain::from_str::<ECSLogLevel>(input).context(error::InvalidPlainValueSnafu {
            field: "ecs.loglevel",
        })?;
        Ok(ECSAgentLogLevel {
            inner: input.to_string(),
        })
    }
}

#[cfg(test)]
mod test_ecs_agent_log_level {
    use super::ECSAgentLogLevel;

    #[test]
    fn good_vals() {
        for val in &["debug", "info", "warn"] {
            val.parse::<ECSAgentLogLevel>().unwrap();
        }
    }

    #[test]
    fn bad_vals() {
        for val in &["", "warning", "errors", " "] {
            val.parse::<ECSAgentLogLevel>().unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ECSAgentImagePullBehavior represents a string that contains a valid ECS Image Pull Behavior for the ECS agent.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ECSAgentImagePullBehavior {
    inner: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ECSImagePullBehavior {
    Default = 0,
    Always,
    Once,
    PreferCached,
}

impl FromStr for ECSImagePullBehavior {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let image_pull_behavior = serde_plain::from_str::<ECSImagePullBehavior>(input).context(
            error::InvalidPlainValueSnafu {
                field: "ecs.image_pull_behavior",
            },
        )?;
        Ok(image_pull_behavior)
    }
}

string_impls_for!(ECSAgentImagePullBehavior, "ECSAgentImagePullBehavior");

impl FromStr for ECSAgentImagePullBehavior {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input.parse::<ECSImagePullBehavior>()?;
        Ok(ECSAgentImagePullBehavior {
            inner: input.to_string(),
        })
    }
}

#[cfg(test)]
mod test_ecs_agent_image_pull_behavior {
    use super::ECSAgentImagePullBehavior;

    #[test]
    fn good_vals() {
        for val in &["default", "always", "once", "prefer-cached"] {
            val.parse::<ECSAgentImagePullBehavior>().unwrap();
        }
    }

    #[test]
    fn bad_vals() {
        for val in &["", "tomorrow", "never", " "] {
            val.parse::<ECSAgentImagePullBehavior>().unwrap_err();
        }
    }
}
