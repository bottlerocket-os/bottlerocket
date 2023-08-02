//! Provides utilities for specifying `ExtensionRequirement`s via the command line.
//! Extension requirements on the command line are passed in the form `extension@version(helper1, helper2, ...)`
use super::{error, Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use schnauzer::template::ExtensionRequirement;
use snafu::OptionExt;
use std::str::FromStr;

/// Newtype wrapper around `ExtensionRequirement` provided to parse CLI arguments.
#[derive(Debug, Clone)]
pub struct CLIExtensionRequirement(ExtensionRequirement);

impl From<CLIExtensionRequirement> for ExtensionRequirement {
    fn from(value: CLIExtensionRequirement) -> Self {
        value.0
    }
}

const EXTENSION_REQUIREMENT_RE: &str = r#"(?x)
    ^\s*                                            # Allow loose whitespace
    (?P<extension>[a-zA-Z0-9_\-]+)                  # Extension name
    @
    (?P<version>[a-zA-Z0-9_\-\.]+)                  # Extension version
    \s*
    (?:\(\s*                                        # Start list of helpers
    (?P<helpers>
    (?:[a-zA-Z0-9\-_]+                              # Helper name
    (?:\s*,\s*)?)*                                  # Delimited by commas and optional whitespace
    )
    \s*\))?                                         # End list of helpers
    \s*$"#;

lazy_static! {
    static ref EXTENSION_REQUIREMENT: Regex = Regex::new(EXTENSION_REQUIREMENT_RE).unwrap();
    static ref COMMA_DELIMITER: Regex = Regex::new(r"\s*,\s*").unwrap();
}

/// Splits comma-delimited string of helpers with optional whitespace.
fn split_helpers(helpers: &str) -> Vec<&str> {
    COMMA_DELIMITER.split(helpers).collect()
}

impl FromStr for CLIExtensionRequirement {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let re_captures =
            EXTENSION_REQUIREMENT
                .captures(s)
                .context(error::RequirementsParseSnafu {
                    requirement: s.to_string(),
                    reason: "Extension requirement regex did not find a match",
                })?;

        let name = re_captures
            .name("extension")
            .context(error::RequirementsParseSnafu {
                requirement: s.to_string(),
                reason: "Did not find extension name",
            })?
            .as_str()
            .to_string();

        let version = re_captures
            .name("version")
            .context(error::RequirementsParseSnafu {
                requirement: s.to_string(),
                reason: "Did not find extension version",
            })?
            .as_str()
            .to_string();

        let helpers = re_captures
            .name("helpers")
            .map(|helpers| {
                split_helpers(helpers.as_str())
                    .into_iter()
                    .filter(|helper| !helper.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        Ok(CLIExtensionRequirement(ExtensionRequirement {
            name,
            version,
            helpers,
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod test {
    use schnauzer::v2::ExtensionRequirement;

    use super::CLIExtensionRequirement;

    #[test]
    fn test_requirements_parse_succeeds() {
        let test_cases = &[
            (
                "extension@version(helper1)",
                ExtensionRequirement {
                    name: "extension".to_string(),
                    version: "version".to_string(),
                    helpers: vec!["helper1".to_string()],
                    ..Default::default()
                },
            ),
            (
                " myextension@v1(    )   ",
                ExtensionRequirement {
                    name: "myextension".to_string(),
                    version: "v1".to_string(),
                    helpers: vec![],
                    ..Default::default()
                },
            ),
            (
                "extension@version",
                ExtensionRequirement {
                    name: "extension".to_string(),
                    version: "version".to_string(),
                    helpers: vec![],
                    ..Default::default()
                },
            ),
            (
                "std@v1   (   base64_decode,    join_array  )  ",
                ExtensionRequirement {
                    name: "std".to_string(),
                    version: "v1".to_string(),
                    helpers: vec!["base64_decode", "join_array"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    ..Default::default()
                },
            ),
            (
                "weird-extension@but_valid1.23(1, 2,   3)",
                ExtensionRequirement {
                    name: "weird-extension".to_string(),
                    version: "but_valid1.23".to_string(),
                    helpers: vec!["1", "2", "3"].into_iter().map(String::from).collect(),
                    ..Default::default()
                },
            ),
        ];

        for (requirement_string, expected) in test_cases.into_iter() {
            let parsed: ExtensionRequirement = requirement_string
                .parse::<CLIExtensionRequirement>()
                .unwrap()
                .into();
            assert_eq!(parsed, *expected);
        }
    }

    #[test]
    fn test_requirements_parse_fails() {
        let test_cases = &[
            "unversioned",
            "no.dots.in.name@v1",
            "name@version(no.dots.in.helpers)",
            "what-helpers@v1(,,,,)",
            "badchar?@v1()",
        ];

        for requirement_string in test_cases.into_iter() {
            assert!(requirement_string
                .parse::<CLIExtensionRequirement>()
                .is_err());
        }
    }
}
