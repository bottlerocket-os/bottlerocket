//! This module contains data types that can be used in the model when special input/output
//! (ser/de) behavior is desired.  For example, the ValidBase64 type can be used for a model field
//! when we don't even want to accept an API call with invalid base64 data.

// The pattern in this file is to make a struct and implement TryFrom<&str> with code that does
// necessary checks and returns the struct.  Other traits that treat the struct like a string can
// be implemented for you with the string_impls_for macro.

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use serde::de::Error as _;
use snafu::{ensure, ResultExt};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;

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
            type Error = error::Error;

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

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ValidBase64 can only be created by deserializing from valid base64 text.  It stores the
/// original text, not the decoded form.  Its purpose is input validation, namely being used as a
/// field in a model structure so that you don't even accept a request with a field that has
/// invalid base64.
// Note: we use the default base64::STANDARD config which uses/allows "=" padding.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidBase64 {
    inner: String,
}

/// Validate base64 format before we accept the input.
impl TryFrom<&str> for ValidBase64 {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        base64::decode(&input).context(error::InvalidBase64)?;
        Ok(ValidBase64 {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ValidBase64, "ValidBase64");

#[cfg(test)]
mod test_valid_base64 {
    use super::ValidBase64;
    use std::convert::TryFrom;

    #[test]
    fn valid_base64() {
        let v = ValidBase64::try_from("aGk=").unwrap();
        let decoded_bytes = base64::decode(v.as_ref()).unwrap();
        let decoded = std::str::from_utf8(&decoded_bytes).unwrap();
        assert_eq!(decoded, "hi");
    }

    #[test]
    fn invalid_base64() {
        assert!(ValidBase64::try_from("invalid base64").is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// SingleLineString can only be created by deserializing from a string that contains at most one
/// line.  It stores the original form and makes it accessible through standard traits.  Its
/// purpose is input validation, for example in cases where you want to accept input for a
/// configuration file and want to ensure a user can't create a new line with extra configuration.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SingleLineString {
    inner: String,
}

impl TryFrom<&str> for SingleLineString {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        // Rust does not treat all Unicode line terminators as starting a new line, so we check for
        // specific characters here, rather than just counting from lines().
        // https://en.wikipedia.org/wiki/Newline#Unicode
        let line_terminators = [
            '\n',       // newline (0A)
            '\r',       // carriage return (0D)
            '\u{000B}', // vertical tab
            '\u{000C}', // form feed
            '\u{0085}', // next line
            '\u{2028}', // line separator
            '\u{2029}', // paragraph separator
        ];

        ensure!(
            !input.contains(&line_terminators[..]),
            error::StringContainsLineTerminator
        );

        Ok(Self {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(SingleLineString, "SingleLineString");

#[cfg(test)]
mod test_single_line_string {
    use super::SingleLineString;
    use std::convert::TryFrom;

    #[test]
    fn valid_single_line_string() {
        assert!(SingleLineString::try_from("").is_ok());
        assert!(SingleLineString::try_from("hi").is_ok());
        let long_string = std::iter::repeat(" ").take(9999).collect::<String>();
        let json_long_string = format!("{}", &long_string);
        assert!(SingleLineString::try_from(json_long_string).is_ok());
    }

    #[test]
    fn invalid_single_line_string() {
        assert!(SingleLineString::try_from("Hello\nWorld").is_err());

        assert!(SingleLineString::try_from("\n").is_err());
        assert!(SingleLineString::try_from("\r").is_err());
        assert!(SingleLineString::try_from("\r\n").is_err());

        assert!(SingleLineString::try_from("\u{000B}").is_err()); // vertical tab
        assert!(SingleLineString::try_from("\u{000C}").is_err()); // form feed
        assert!(SingleLineString::try_from("\u{0085}").is_err()); // next line
        assert!(SingleLineString::try_from("\u{2028}").is_err()); // line separator
        assert!(SingleLineString::try_from("\u{2029}").is_err());
        // paragraph separator
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Identifier can only be created by deserializing from a string that contains
/// ASCII alphanumeric characters, plus hyphens, which we use as our standard word separator
/// character in user-facing identifiers. It stores the original form and makes it accessible
/// through standard traits. Its purpose is to validate input for identifiers like container names
/// that might be used to create files/directories.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Identifier {
    inner: String,
}

impl TryFrom<&str> for Identifier {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            input
                .chars()
                .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-'),
            error::InvalidIdentifier { input }
        );
        Ok(Identifier {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(Identifier, "Identifier");

#[cfg(test)]
mod test_valid_identifier {
    use super::Identifier;
    use std::convert::TryFrom;

    #[test]
    fn valid_identifier() {
        assert!(Identifier::try_from("hello-world").is_ok());
        assert!(Identifier::try_from("helloworld").is_ok());
        assert!(Identifier::try_from("123321hello").is_ok());
        assert!(Identifier::try_from("hello-1234").is_ok());
        assert!(Identifier::try_from("--------").is_ok());
        assert!(Identifier::try_from("11111111").is_ok());
    }

    #[test]
    fn invalid_identifier() {
        assert!(Identifier::try_from("../").is_err());
        assert!(Identifier::try_from("{}").is_err());
        assert!(Identifier::try_from("hello|World").is_err());
        assert!(Identifier::try_from("hello\nWorld").is_err());
        assert!(Identifier::try_from("hello_world").is_err());
        assert!(Identifier::try_from("タール").is_err());
        assert!(Identifier::try_from("💝").is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Url represents a string that contains a valid URL, according to url::Url, though it also
/// allows URLs without a scheme (e.g. without "http://") because it's common.  It stores the
/// original string and makes it accessible through standard traits. Its purpose is to validate
/// input for any field containing a network address.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Url {
    inner: String,
}

impl TryFrom<&str> for Url {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        if let Ok(_) = input.parse::<url::Url>() {
            return Ok(Url {
                inner: input.to_string(),
            });
        } else {
            // It's very common to specify URLs without a scheme, so we add one and see if that
            // fixes parsing.
            let prefixed = format!("http://{}", input);
            if let Ok(_) = prefixed.parse::<url::Url>() {
                return Ok(Url {
                    inner: input.to_string(),
                });
            }
        }
        error::InvalidUrl { input }.fail()
    }
}

string_impls_for!(Url, "Url");

#[cfg(test)]
mod test_url {
    use super::Url;
    use std::convert::TryFrom;

    #[test]
    fn good_urls() {
        for ok in &[
            "https://example.com/path",
            "https://example.com",
            "example.com/path",
            "example.com",
            "ntp://127.0.0.1/path",
            "ntp://127.0.0.1",
            "127.0.0.1/path",
            "127.0.0.1",
            "http://localhost/path",
            "http://localhost",
            "localhost/path",
            "localhost",
        ] {
            Url::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_urls() {
        for err in &[
            "how are you",
            "weird@",
        ] {
            Url::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesName represents a string that contains a valid Kubernetes resource name.  It stores
/// the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#names
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesName {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_NAME: Regex = Regex::new(r"^[0-9a-z.-]{1,253}$").unwrap();
}

impl TryFrom<&str> for KubernetesName {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_NAME.is_match(input),
            error::Pattern {
                thing: "Kubernetes name",
                pattern: KUBERNETES_NAME.clone(),
                input
            }
        );
        Ok(KubernetesName {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesName, "KubernetesName");

#[cfg(test)]
mod test_kubernetes_name {
    use super::KubernetesName;
    use std::convert::TryFrom;

    #[test]
    fn good_names() {
        for ok in &["howdy", "42", "18-eighteen."] {
            KubernetesName::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &["", "HOWDY", "@", "hi/there", &"a".repeat(254)] {
            KubernetesName::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesLabelKey represents a string that contains a valid Kubernetes label key.  It stores
/// the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesLabelKey {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_LABEL_KEY: Regex = Regex::new(
        r"(?x)^
       (  # optional prefix
           [[:alnum:].-]{1,253}/  # DNS label characters followed by slash
       )?
       [[:alnum:]]  # at least one alphanumeric
       (
           ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
           [[:alnum:]]  # have to end with alphanumeric
       )?
   $"
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesLabelKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_LABEL_KEY.is_match(input),
            error::BigPattern {
                thing: "Kubernetes label key",
                input
            }
        );
        Ok(KubernetesLabelKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesLabelKey, "KubernetesLabelKey");

#[cfg(test)]
mod test_kubernetes_label_key {
    use super::KubernetesLabelKey;
    use std::convert::TryFrom;

    #[test]
    fn good_keys() {
        for ok in &[
            "no-prefix",
            "have.a/prefix",
            "more-chars_here.now",
            &"a".repeat(63),
            &format!("{}/{}", "a".repeat(253), "name"),
        ] {
            KubernetesLabelKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_keys() {
        for err in &[
            ".bad",
            "bad.",
            &"a".repeat(64),
            &format!("{}/{}", "a".repeat(254), "name"),
        ] {
            KubernetesLabelKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesLabelValue represents a string that contains a valid Kubernetes label value.  It
/// stores the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesLabelValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_LABEL_VALUE: Regex = Regex::new(
        r"(?x)
        ^$ |  # may be empty, or:
        ^
           [[:alnum:]]  # at least one alphanumeric
           (
               ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
               [[:alnum:]]  # have to end with alphanumeric
           )?
        $
   "
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesLabelValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_LABEL_VALUE.is_match(input),
            error::BigPattern {
                thing: "Kubernetes label value",
                input
            }
        );
        Ok(KubernetesLabelValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesLabelValue, "KubernetesLabelValue");

#[cfg(test)]
mod test_kubernetes_label_value {
    use super::KubernetesLabelValue;
    use std::convert::TryFrom;

    #[test]
    fn good_values() {
        for ok in &["", "more-chars_here.now", &"a".repeat(63)] {
            KubernetesLabelValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &[".bad", "bad.", &"a".repeat(64)] {
            KubernetesLabelValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesTaintValue represents a string that contains a valid Kubernetes taint value, which is
/// like a label value, plus a colon, plus an "effect".  It stores the original string and makes it
/// accessible through standard traits.
///
/// Note: Kubelet won't launch if you specify an effect it doesn't know about, but we don't want to
/// gatekeep all possible values, so be careful.
// Note: couldn't find an exact spec for this.  Cobbling things together, and guessing a bit as to
// the syntax of the effect.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set
// https://kubernetes.io/docs/concepts/configuration/taint-and-toleration/
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesTaintValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_TAINT_VALUE: Regex = Regex::new(
        r"(?x)^
       [[:alnum:]]  # at least one alphanumeric
       (
           ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
           [[:alnum:]]  # have to end with alphanumeric
       )?
       :  # separate the label value from the effect
       [[:alnum:]]{1,253}  # effect
   $"
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesTaintValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_TAINT_VALUE.is_match(input),
            error::BigPattern {
                thing: "Kubernetes taint value",
                input
            }
        );
        Ok(KubernetesTaintValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesTaintValue, "KubernetesTaintValue");

#[cfg(test)]
mod test_kubernetes_taint_value {
    use super::KubernetesTaintValue;
    use std::convert::TryFrom;

    #[test]
    fn good_values() {
        // All the examples from the docs linked above
        for ok in &[
            "value:NoSchedule",
            "value:PreferNoSchedule",
            "value:NoExecute",
        ] {
            KubernetesTaintValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &[".bad", "bad.", &"a".repeat(254), "value:", ":effect"] {
            KubernetesTaintValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesClusterName represents a string that contains a valid Kubernetes cluster name.  It
/// stores the original string and makes it accessible through standard traits.
// Note: I was unable to find the rules for cluster naming.  We know they have to fit into label
// values, because of the common cluster-name label, but they also can't be empty.  This combines
// those two characteristics into a new type, until we find an explicit syntax.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesClusterName {
    inner: String,
}

impl TryFrom<&str> for KubernetesClusterName {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            !input.is_empty(),
            error::InvalidClusterName {
                name: input,
                msg: "must not be empty"
            }
        );
        ensure!(
            KubernetesLabelValue::try_from(input).is_ok(),
            error::InvalidClusterName {
                name: input,
                msg: "cluster names must be valid Kubernetes label values"
            }
        );

        Ok(KubernetesClusterName {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesClusterName, "KubernetesClusterName");

#[cfg(test)]
mod test_kubernetes_cluster_name {
    use super::KubernetesClusterName;
    use std::convert::TryFrom;

    #[test]
    fn good_cluster_names() {
        for ok in &["more-chars_here.now", &"a".repeat(63)] {
            KubernetesClusterName::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_alues() {
        for err in &["", ".bad", "bad.", &"a".repeat(64)] {
            KubernetesClusterName::try_from(*err).unwrap_err();
        }
    }
}
