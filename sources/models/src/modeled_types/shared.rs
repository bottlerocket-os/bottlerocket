use lazy_static::lazy_static;
use regex::Regex;
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

/// ValidLinuxHostname represents a string that contains a valid Linux hostname as defined by
/// https://man7.org/linux/man-pages/man7/hostname.7.html.  It stores the original form and makes
/// it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidLinuxHostname {
    inner: String,
}

lazy_static! {
    // According to the man page above, hostnames must be between 1-253 characters long consisting
    // of characters [0-9a-z.-].
    pub(crate) static ref VALID_LINUX_HOSTNAME: Regex = Regex::new(r"^[0-9a-z.-]{1,253}$").unwrap();
}

impl TryFrom<&str> for ValidLinuxHostname {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            VALID_LINUX_HOSTNAME.is_match(input),
            error::InvalidLinuxHostname {
                input,
                msg: "must only be [0-9a-z.-], and 1-253 chars long"
            }
        );

        // Though the man page doesn't explicitly disallow hostnames that start with dots, dots are
        // used as separators so starting with a separator would imply an empty domain, which isn't
        // allowed (must be at least one character).
        ensure!(
            !input.starts_with("-") && !input.starts_with("."),
            error::InvalidLinuxHostname {
                input,
                msg: "must not start with '-' or '.'"
            }
        );

        // Each segment must be from 1-63 chars long and shouldn't start with "-"
        ensure!(
            input
                .split(".")
                .all(|x| x.len() >= 1 && x.len() <= 63 && !x.starts_with("-")),
            error::InvalidLinuxHostname {
                input,
                msg: "segment is less than 1 or greater than 63 chars"
            }
        );

        Ok(Self {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ValidLinuxHostname, "ValidLinuxHostname");

#[cfg(test)]
mod test_valid_linux_hostname {
    use super::ValidLinuxHostname;
    use std::convert::TryFrom;

    #[test]
    fn valid_linux_hostname() {
        assert!(ValidLinuxHostname::try_from("hello").is_ok());
        assert!(ValidLinuxHostname::try_from("hello1234567890").is_ok());

        let segment_limit = std::iter::repeat("a").take(63).collect::<String>();
        assert!(ValidLinuxHostname::try_from(segment_limit.clone()).is_ok());

        let segment = std::iter::repeat("a").take(61).collect::<String>();
        let long_name = format!(
            "{}.{}.{}.{}",
            &segment_limit, &segment_limit, &segment_limit, &segment
        );
        assert!(ValidLinuxHostname::try_from(long_name).is_ok());
    }

    #[test]
    fn invalid_linux_hostname() {
        assert!(ValidLinuxHostname::try_from(" ").is_err());
        assert!(ValidLinuxHostname::try_from("-a").is_err());
        assert!(ValidLinuxHostname::try_from(".a").is_err());
        assert!(ValidLinuxHostname::try_from("@a").is_err());
        assert!(ValidLinuxHostname::try_from("a..a").is_err());
        assert!(ValidLinuxHostname::try_from("a.a.-a.a1234").is_err());

        let long_segment = std::iter::repeat("a").take(64).collect::<String>();
        assert!(ValidLinuxHostname::try_from(long_segment.clone()).is_err());

        let long_name = format!(
            "{}.{}.{}.{}",
            &long_segment, &long_segment, &long_segment, &long_segment
        );
        assert!(ValidLinuxHostname::try_from(long_name).is_err());
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

const CONTAINERD_ID_LENGTH: usize = 76;

impl TryFrom<&str> for Identifier {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let valid_identifier = input
            .chars()
            .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-')
            && input.len() <= CONTAINERD_ID_LENGTH;
        ensure!(valid_identifier, error::InvalidIdentifier { input });
        Ok(Identifier {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(Identifier, "Identifier");

#[cfg(test)]
mod test_valid_identifier {
    use super::{Identifier, CONTAINERD_ID_LENGTH};
    use std::convert::TryFrom;

    #[test]
    fn valid_identifier() {
        assert!(Identifier::try_from("hello-world").is_ok());
        assert!(Identifier::try_from("helloworld").is_ok());
        assert!(Identifier::try_from("123321hello").is_ok());
        assert!(Identifier::try_from("hello-1234").is_ok());
        assert!(Identifier::try_from("--------").is_ok());
        assert!(Identifier::try_from("11111111").is_ok());
        assert!(Identifier::try_from(vec!["X"; CONTAINERD_ID_LENGTH].join("")).is_ok());
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
        assert!(Identifier::try_from(vec!["X"; CONTAINERD_ID_LENGTH + 1].join("")).is_err());
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
            "localhost:8080",
            ".internal",
            ".cluster.local",
        ] {
            Url::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_urls() {
        for err in &["how are you", "weird@"] {
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
    type Error = semver::Error;

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
    use semver::Version;
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
        for err in &[
            "hi",
            "1.0",
            "1",
            "v",
            "v1",
            "v1.0",
            "vv1.1.0",
            "1.0.3-beta.1.01",
            "v1.0.3-beta.1.01",
        ] {
            FriendlyVersion::try_from(*err).unwrap_err();
            let res: Result<Version, semver::Error> = Version::try_from(FriendlyVersion {
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

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// SysctlKey represents a string that is a valid Linux sysctl key; keys must be representable as
/// filesystem paths, and are generally kept to lowercase_underscored_names separated with '.' or
/// '/'.  SysctlKey stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SysctlKey {
    inner: String,
}

lazy_static! {
    /// Pattern matching the name of a sysctl key.  Must be representable as a path; we'll go a bit
    /// further and enforce a basic pattern that would match all known keys, plus some leeway.
    pub(crate) static ref SYSCTL_KEY: Regex = Regex::new(r"^[a-zA-Z0-9./_-]{1,128}$").unwrap();
}

impl TryFrom<&str> for SysctlKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        // Basic directory traversal checks; corndog also checks
        ensure!(
            !input.contains(".."),
            error::InvalidSysctlKey {
                input,
                msg: format!("must not contain '..'"),
            }
        );
        ensure!(
            !input.starts_with('.') && !input.starts_with('/'),
            error::InvalidSysctlKey {
                input,
                msg: format!("must not start with '.' or '/'"),
            }
        );
        ensure!(
            SYSCTL_KEY.is_match(input),
            error::InvalidSysctlKey {
                input,
                msg: format!("must match pattern {}", *SYSCTL_KEY),
            }
        );
        Ok(SysctlKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(SysctlKey, "SysctlKey");

#[cfg(test)]
mod test_sysctl_key {
    use super::SysctlKey;
    use std::convert::TryFrom;

    #[test]
    fn valid_sysctl_key() {
        for ok in &[
            // Longest real one
            "net/ipv4/conf/enp0s42f3/igmpv3_unsolicited_report_interval",
            // Dots or slashes OK
            "net.ipv4.conf.enp0s42f3.igmpv3_unsolicited_report_interval",
            // Mixed dots/slashes isn't supported by sysctl(8), but it's unambiguous
            "net/ipv4.conf.enp0s42f3/igmpv3_unsolicited_report_interval",
            // Shortest real one
            "fs/aio-nr",
            // Shortest allowed
            "a",
            // Longest allowed
            &"a".repeat(128),
            // All allowed characters
            "-./0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ] {
            SysctlKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn invalid_sysctl_key() {
        for err in &[
            // Too long
            &"a".repeat(129),
            // Too short,
            "",
            // Sneaky sneaky
            "hi/../../there",
            "../hi",
            "/../hi",
            // Invalid characters
            "!",
            "@",
            "#",
            "$",
            "%",
            "^",
            "&",
            "*",
            "(",
            ")",
            "\"",
            "'",
            "\\",
            "|",
            "~",
            "`",
        ] {
            SysctlKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Lockdown represents a string that is a valid Linux kernel lockdown mode name.  It stores the
/// original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Lockdown {
    inner: String,
}

impl TryFrom<&str> for Lockdown {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        ensure!(
            matches!(input, "none" | "integrity" | "confidentiality"),
            error::InvalidLockdown { input }
        );
        Ok(Lockdown {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(Lockdown, "Lockdown");

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BootstrapContainerMode {
    inner: String,
}

impl TryFrom<&str> for BootstrapContainerMode {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        ensure!(
            matches!(input, "off" | "once" | "always"),
            error::InvalidBootstrapContainerMode { input }
        );
        Ok(BootstrapContainerMode {
            inner: input.to_string(),
        })
    }
}

impl Default for BootstrapContainerMode {
    fn default() -> Self {
        BootstrapContainerMode {
            inner: "off".to_string(),
        }
    }
}

string_impls_for!(BootstrapContainerMode, "BootstrapContainerMode");

#[cfg(test)]
mod test_valid_container_mode {
    use super::BootstrapContainerMode;
    use std::convert::TryFrom;

    #[test]
    fn valid_container_mode() {
        for ok in &["off", "once", "always"] {
            assert!(BootstrapContainerMode::try_from(*ok).is_ok());
        }
    }

    #[test]
    fn invalid_container_mode() {
        assert!(BootstrapContainerMode::try_from("invalid").is_err());
    }
}
