use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use super::error;
use semver::Version;
use serde::de::Error as _;
use snafu::{ensure, ResultExt};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use url::Host;

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

/// FriendlyVersion represents a version string that can optionally be prefixed with 'v'.
/// It can also be set to 'latest' to represent the latest version. It stores the original string
/// and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FriendlyVersion {
    inner: String,
}

impl TryFrom<&str> for FriendlyVersion {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        if input == "latest" {
            return Ok(FriendlyVersion {
                inner: input.to_string(),
            });
        }
        // If the string begins with a 'v', skip it before checking if it is valid semver.
        let version = if input.starts_with('v') {
            &input[1..]
        } else {
            input
        };

        if version.parse::<semver::Version>().is_ok() {
            return Ok(FriendlyVersion {
                inner: input.to_string(),
            });
        }
        error::InvalidVersion { input }.fail()
    }
}

impl TryFrom<FriendlyVersion> for semver::Version {
    type Error = semver::SemVerError;

    fn try_from(input: FriendlyVersion) -> Result<semver::Version, Self::Error> {
        // If the string begins with a 'v', skip it before conversion
        let version = if input.inner.starts_with('v') {
            &input.inner[1..]
        } else {
            &input.inner
        };
        Version::from_str(version)
    }
}

string_impls_for!(FriendlyVersion, "FriendlyVersion");

#[cfg(test)]
mod test_version {
    use super::FriendlyVersion;
    use semver::{SemVerError, Version};
    use std::convert::TryFrom;
    use std::convert::TryInto;

    #[test]
    fn good_version_strings() {
        for ok in &[
            "1.0.0",
            "v1.0.0",
            "1.0.1-alpha",
            "v1.0.1-alpha",
            "1.0.2-alpha+1.0",
            "v1.0.2-alpha+1.0",
            "1.0.3-beta.1.01",
            "v1.0.3-beta.1.01",
            "1.1.0-rc.1.1",
            "v1.1.0-rc.1.1",
            "latest",
        ] {
            FriendlyVersion::try_from(*ok).unwrap();
            // Test conversion to semver::Version
            if *ok != "latest" {
                let _: Version = FriendlyVersion {
                    inner: ok.to_string(),
                }
                .try_into()
                .unwrap();
            }
        }
    }

    #[test]
    fn bad_version_string() {
        for err in &["hi", "1.0", "1", "v", "v1", "v1.0", "vv1.1.0"] {
            FriendlyVersion::try_from(*err).unwrap_err();
            let res: Result<Version, SemVerError> = Version::try_from(FriendlyVersion {
                inner: err.to_string(),
            });
            res.unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// DNSDomain represents a string that is a valid DNS domain. It stores the
/// original string and makes it accessible through standard traits. Its purpose
/// is input validation, for example validating the kubelet's "clusterDomain"
/// config value.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DNSDomain {
    inner: String,
}

impl TryFrom<&str> for DNSDomain {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        ensure!(
            !input.starts_with('.'),
            error::InvalidDomainName {
                input: input,
                msg: "must not start with '.'",
            }
        );

        let host = Host::parse(input).or_else(|e| {
            error::InvalidDomainName {
                input: input,
                msg: e.to_string(),
            }
            .fail()
        })?;
        match host {
            Host::Ipv4(_) | Host::Ipv6(_) => error::InvalidDomainName {
                input: input,
                msg: "IP address is not a valid domain name",
            }
            .fail(),
            Host::Domain(_) => Ok(Self {
                inner: input.to_string(),
            }),
        }
    }
}

string_impls_for!(DNSDomain, "DNSDomain");

#[cfg(test)]
mod test_dns_domain {
    use super::DNSDomain;
    use std::convert::TryFrom;

    #[test]
    fn valid_dns_domain() {
        for ok in &["cluster.local", "dev.eks", "stage.eks", "prod.eks"] {
            assert!(DNSDomain::try_from(*ok).is_ok());
        }
    }

    #[test]
    fn invalid_dns_domain() {
        for err in &[
            "foo/com",
            ".a",
            "123.123.123.123",
            "[2001:db8::ff00:42:8329]",
        ] {
            assert!(DNSDomain::try_from(*err).is_err());
        }
    }
}
