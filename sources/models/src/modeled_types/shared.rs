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
use std::net::IpAddr;
use std::ops::Deref;
use std::str::FromStr;
use url::Host;
use x509_parser;

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
impl FromStr for ValidBase64 {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        base64::decode(&input).context(error::InvalidBase64Snafu)?;
        Ok(ValidBase64 {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ValidBase64, "ValidBase64");

#[cfg(test)]
mod test_valid_base64 {
    use super::ValidBase64;

    #[test]
    fn valid_base64() {
        let v = ("aGk=").parse::<ValidBase64>().unwrap();
        let decoded_bytes = base64::decode(v.as_ref()).unwrap();
        let decoded = std::str::from_utf8(&decoded_bytes).unwrap();
        assert_eq!(decoded, "hi");
    }

    #[test]
    fn invalid_base64() {
        assert!(("invalid base64").parse::<ValidBase64>().is_err());
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

impl FromStr for SingleLineString {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
            error::StringContainsLineTerminatorSnafu
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

    #[test]
    fn valid_single_line_string() {
        assert!(("").parse::<SingleLineString>().is_ok());
        assert!(("hi").parse::<SingleLineString>().is_ok());
        let long_string = std::iter::repeat(" ").take(9999).collect::<String>();
        let json_long_string = format!("{}", &long_string);
        assert!((&json_long_string).parse::<SingleLineString>().is_ok());
    }

    #[test]
    fn invalid_single_line_string() {
        assert!(("Hello\nWorld").parse::<SingleLineString>().is_err());

        assert!(("\n").parse::<SingleLineString>().is_err());
        assert!(("\r").parse::<SingleLineString>().is_err());
        assert!(("\r\n").parse::<SingleLineString>().is_err());

        assert!(("\u{000B}").parse::<SingleLineString>().is_err()); // vertical tab
        assert!(("\u{000C}").parse::<SingleLineString>().is_err()); // form feed
        assert!(("\u{0085}").parse::<SingleLineString>().is_err()); // next line
        assert!(("\u{2028}").parse::<SingleLineString>().is_err()); // line separator
        assert!(("\u{2029}").parse::<SingleLineString>().is_err());
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

impl FromStr for ValidLinuxHostname {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            VALID_LINUX_HOSTNAME.is_match(input),
            error::InvalidLinuxHostnameSnafu {
                input,
                msg: "must only be [0-9a-z.-], and 1-253 chars long"
            }
        );

        // Though the man page doesn't explicitly disallow hostnames that start with dots, dots are
        // used as separators so starting with a separator would imply an empty domain, which isn't
        // allowed (must be at least one character).
        ensure!(
            !input.starts_with("-") && !input.starts_with("."),
            error::InvalidLinuxHostnameSnafu {
                input,
                msg: "must not start with '-' or '.'"
            }
        );

        // Each segment must be from 1-63 chars long and shouldn't start with "-"
        ensure!(
            input
                .split(".")
                .all(|x| x.len() >= 1 && x.len() <= 63 && !x.starts_with("-")),
            error::InvalidLinuxHostnameSnafu {
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

    #[test]
    fn valid_linux_hostname() {
        assert!(("hello").parse::<ValidLinuxHostname>().is_ok());
        assert!(("hello1234567890").parse::<ValidLinuxHostname>().is_ok());

        let segment_limit = std::iter::repeat("a").take(63).collect::<String>();
        assert!(segment_limit.parse::<ValidLinuxHostname>().is_ok());

        let segment = std::iter::repeat("a").take(61).collect::<String>();
        let long_name = format!(
            "{}.{}.{}.{}",
            &segment_limit, &segment_limit, &segment_limit, &segment
        );
        assert!((&long_name).parse::<ValidLinuxHostname>().is_ok());
    }

    #[test]
    fn invalid_linux_hostname() {
        assert!((" ").parse::<ValidLinuxHostname>().is_err());
        assert!(("-a").parse::<ValidLinuxHostname>().is_err());
        assert!((".a").parse::<ValidLinuxHostname>().is_err());
        assert!(("@a").parse::<ValidLinuxHostname>().is_err());
        assert!(("a..a").parse::<ValidLinuxHostname>().is_err());
        assert!(("a.a.-a.a1234").parse::<ValidLinuxHostname>().is_err());

        let long_segment = std::iter::repeat("a").take(64).collect::<String>();
        assert!(long_segment.parse::<ValidLinuxHostname>().is_err());

        let long_name = format!(
            "{}.{}.{}.{}",
            &long_segment, &long_segment, &long_segment, &long_segment
        );
        assert!((&long_name).parse::<ValidLinuxHostname>().is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// EtcHostsEntries represents a mapping of IP Address to hostname aliases that can apply to those
/// addresses.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EtcHostsEntries(
    // Ordering matters in /etc/hosts, and this setting directly maps to that file and its behavior in glibc.
    // Repeated IP Addresses have their host aliases merged to a single line, respecting the order as they appear in this list.
    Vec<(IpAddr, Vec<ValidLinuxHostname>)>,
);

impl EtcHostsEntries {
    pub fn iter_merged(&self) -> impl Iterator<Item = (IpAddr, Vec<ValidLinuxHostname>)> {
        let mut merged = indexmap::IndexMap::with_capacity(self.0.len());

        for (ip_address, aliases) in &self.0 {
            merged
                .entry(ip_address.clone())
                .or_insert(vec![])
                .append(&mut (aliases.clone()));
        }

        merged.into_iter()
    }
}

#[cfg(test)]
mod test_etc_hosts_entries {
    use super::{EtcHostsEntries, ValidLinuxHostname};
    use std::net::IpAddr;

    #[test]
    fn test_valid_etc_hosts_entries() {
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[
            ["127.0.0.1", ["localhost", "localhost4"]],
            ["::1", ["localhost", "localhost6"]]
        ]"#
        )
        .is_ok());
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[
            ["127.0.0.1", ["localhost", "localhost4"]],
            ["::1", ["localhost", "localhost6"]],
            ["127.0.0.1", ["test.example.com"]]
        ]"#
        )
        .is_ok());
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[
            ["::1", ["localhost", "localhost6"]],
            ["0000:0000:0000:0000:0000:0000:0000:0001", ["test6.example.com"]]
        ]"#
        )
        .is_ok());
        assert!(serde_json::from_str::<EtcHostsEntries>(r#"[]"#).is_ok());
    }

    #[test]
    fn test_invalid_etc_hosts_entries() {
        assert!(
            serde_json::from_str::<EtcHostsEntries>(r#"[["127.0.0.0/24", ["localhost"]]"#).is_err()
        );
        assert!(
            serde_json::from_str::<EtcHostsEntries>(r#"[["not_an_ip", ["localhost"]]"#).is_err()
        );
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[["127.0.0.1", ["not_a_valid_hostname!"]]"#
        )
        .is_err());
    }

    #[test]
    fn test_iter_merged() {
        assert_eq!(
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4"]],
                    ["127.0.0.1", ["test.example.com"]]
                ]"#,
            )
            .unwrap()
            .iter_merged()
            .collect::<Vec<(IpAddr, Vec<ValidLinuxHostname>)>>(),
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4", "test.example.com"]]
                ]"#,
            )
            .unwrap()
            .0
        );

        assert_eq!(
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4"]],
                    ["127.0.0.3", ["test.example.com"]],
                    ["127.0.0.2", ["test.example.com"]],
                    ["127.0.0.1", ["test.example.com"]]
                ]"#,
            )
            .unwrap()
            .iter_merged()
            .collect::<Vec<(IpAddr, Vec<ValidLinuxHostname>)>>(),
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4", "test.example.com"]],
                    ["127.0.0.3", ["test.example.com"]],
                    ["127.0.0.2", ["test.example.com"]]
                ]"#,
            )
            .unwrap()
            .0
        );

        assert_eq!(
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4"]],
                    ["::1", ["localhost", "localhost6"]],
                    ["127.0.0.1", ["test.example.com"]],
                    ["0000:0000:0000:0000:0000:0000:0000:0001", ["test6.example.com"]],
                    ["10.0.0.1", ["example.bottlerocket.aws"]]
                ]"#,
            )
            .unwrap()
            .iter_merged()
            .collect::<Vec<(IpAddr, Vec<ValidLinuxHostname>)>>(),
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4", "test.example.com"]],
                    ["::1", ["localhost", "localhost6", "test6.example.com"]],
                    ["10.0.0.1", ["example.bottlerocket.aws"]]
                ]"#,
            )
            .unwrap()
            .0
        );
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

impl FromStr for Identifier {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let valid_identifier = input
            .chars()
            .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-')
            && input.len() <= CONTAINERD_ID_LENGTH;
        ensure!(valid_identifier, error::InvalidIdentifierSnafu { input });
        Ok(Identifier {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(Identifier, "Identifier");

#[cfg(test)]
mod test_valid_identifier {
    use super::{Identifier, CONTAINERD_ID_LENGTH};

    #[test]
    fn valid_identifier() {
        assert!(("hello-world").parse::<Identifier>().is_ok());
        assert!(("helloworld").parse::<Identifier>().is_ok());
        assert!(("123321hello").parse::<Identifier>().is_ok());
        assert!(("hello-1234").parse::<Identifier>().is_ok());
        assert!(("--------").parse::<Identifier>().is_ok());
        assert!(("11111111").parse::<Identifier>().is_ok());
        assert!((&vec!["X"; CONTAINERD_ID_LENGTH].join(""))
            .parse::<Identifier>()
            .is_ok());
    }

    #[test]
    fn invalid_identifier() {
        assert!(("../").parse::<Identifier>().is_err());
        assert!(("{}").parse::<Identifier>().is_err());
        assert!(("hello|World").parse::<Identifier>().is_err());
        assert!(("hello\nWorld").parse::<Identifier>().is_err());
        assert!(("hello_world").parse::<Identifier>().is_err());
        assert!(("タール").parse::<Identifier>().is_err());
        assert!(("💝").parse::<Identifier>().is_err());
        assert!((&vec!["X"; CONTAINERD_ID_LENGTH + 1].join(""))
            .parse::<Identifier>()
            .is_err());
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

impl FromStr for Url {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
        error::InvalidUrlSnafu { input }.fail()
    }
}

string_impls_for!(Url, "Url");

#[cfg(test)]
mod test_url {
    use super::Url;

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
            ok.parse::<Url>().unwrap();
        }
    }

    #[test]
    fn bad_urls() {
        for err in &["how are you", "weird@"] {
            err.parse::<Url>().unwrap_err();
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

impl FromStr for FriendlyVersion {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
        error::InvalidVersionSnafu { input }.fail()
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
        version.parse::<Version>()
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
            ok.parse::<FriendlyVersion>().unwrap();
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
            err.parse::<FriendlyVersion>().unwrap_err();
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

impl FromStr for DNSDomain {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            !input.starts_with('.'),
            error::InvalidDomainNameSnafu {
                input: input,
                msg: "must not start with '.'",
            }
        );

        let host = Host::parse(input).or_else(|e| {
            error::InvalidDomainNameSnafu {
                input: input,
                msg: e.to_string(),
            }
            .fail()
        })?;
        match host {
            Host::Ipv4(_) | Host::Ipv6(_) => error::InvalidDomainNameSnafu {
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

    #[test]
    fn valid_dns_domain() {
        for ok in &["cluster.local", "dev.eks", "stage.eks", "prod.eks"] {
            assert!(ok.parse::<DNSDomain>().is_ok());
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
            assert!(err.parse::<DNSDomain>().is_err());
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

impl FromStr for SysctlKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Basic directory traversal checks; corndog also checks
        ensure!(
            !input.contains(".."),
            error::InvalidSysctlKeySnafu {
                input,
                msg: format!("must not contain '..'"),
            }
        );
        ensure!(
            !input.starts_with('.') && !input.starts_with('/'),
            error::InvalidSysctlKeySnafu {
                input,
                msg: format!("must not start with '.' or '/'"),
            }
        );
        ensure!(
            SYSCTL_KEY.is_match(input),
            error::InvalidSysctlKeySnafu {
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
            ok.parse::<SysctlKey>().unwrap();
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
            err.parse::<SysctlKey>().unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// BootConfigKey represents a string that is a valid Kernel boot config key; each key word must
/// contain only alphabets, numbers, dash (-) or underscore (_).
/// BootConfigKey stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BootConfigKey {
    inner: String,
}

impl FromStr for BootConfigKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Each individual keyword must be valid
        let valid_key = input.split('.').all(|keyword| {
            !keyword.is_empty()
                && keyword
                    .chars()
                    .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-' || c == '_')
        });
        ensure!(valid_key, error::InvalidBootconfigKeySnafu { input });
        Ok(BootConfigKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(BootConfigKey, "BootConfigKey");

#[cfg(test)]
mod test_bootconfig_key {
    use super::BootConfigKey;

    #[test]
    fn valid_bootconfig_key() {
        for ok in &[
            "keyword1.keyword2",
            "-keyword1.keyword2",
            "_keyword.1.2.3",
            "key_word",
            "key-word",
            "keyword1",
            "keyword1-",
            "keyword2_",
        ] {
            ok.parse::<BootConfigKey>().unwrap();
        }
    }

    #[test]
    fn invalid_bootconfig_key() {
        for err in &[
            "", "①", ".", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "\"", "'", "\\", "|",
            "~", "`",
        ] {
            err.parse::<BootConfigKey>().unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// BootConfigValue represents a string that is a valid Kernel boot config value; each value only
/// contains printable characters or spaces except for delimiters such as semicolon, newline, comma,
/// hash, and closing brace. These delimiters are only usable if the value itself is quoted with
/// single-quotes or double-quotes. Here we treat the value as if they're always quoted in the context
/// of Bottlerocket settings. This means the value just has to be printable ASCII.
/// BootConfigValue stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BootConfigValue {
    inner: String,
}

impl FromStr for BootConfigValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            input.chars().all(|c| c.is_ascii() && !c.is_ascii_control())
            // Values containing both single quotes and double quotes are inherently invalid since quotes
            // cannot be escaped.
                && !(input.contains('"') && input.contains("'")),
            error::InvalidBootconfigValueSnafu { input }
        );
        Ok(BootConfigValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(BootConfigValue, "BootConfigValue");

#[cfg(test)]
mod test_bootconfig_value {
    use super::BootConfigValue;

    #[test]
    fn valid_bootconfig_value() {
        for ok in &[
            "plain",
            "@yogurt@",
            "\"abc",
            " !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}",
            "1",
            "value1",
            "hello.goodbye",
            "",
        ] {
            ok.parse::<BootConfigValue>().unwrap();
        }
    }

    #[test]
    fn invalid_bootconfig_value() {
        for err in &["'\"", "bottlerocket①", "💝", "Ï", "—"] {
            err.parse::<BootConfigValue>().unwrap_err();
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

impl FromStr for Lockdown {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            matches!(input, "none" | "integrity" | "confidentiality"),
            error::InvalidLockdownSnafu { input }
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

impl FromStr for BootstrapContainerMode {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            matches!(input, "off" | "once" | "always"),
            error::InvalidBootstrapContainerModeSnafu { input }
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

    #[test]
    fn valid_container_mode() {
        for ok in &["off", "once", "always"] {
            assert!(ok.parse::<BootstrapContainerMode>().is_ok());
        }
    }

    #[test]
    fn invalid_container_mode() {
        assert!(("invalid").parse::<BootstrapContainerMode>().is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PemCertificateString {
    inner: String,
}

impl FromStr for PemCertificateString {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Empty strings are valid to allow deleting bundles
        if input.trim().len() == 0 {
            return Ok(PemCertificateString {
                inner: input.to_string(),
            });
        }
        let decoded_bytes = base64::decode(input).context(error::InvalidBase64Snafu)?;
        // Flag to check if the bundle doesn't contain any valid certificate
        let mut certs_found = false;
        // Validate each certificate in the bundle
        for (_, pem) in x509_parser::pem::Pem::iter_from_buffer(&decoded_bytes).enumerate() {
            // Parse buffer into a PEM object, then to a x509 certificate
            let pem = pem.context(error::InvalidPEMSnafu)?;
            pem.parse_x509()
                .context(error::InvalidX509CertificateSnafu)?;
            certs_found = true;
        }

        // No valid certificate found
        if !certs_found {
            return error::NoCertificatesFoundSnafu {}.fail();
        }

        Ok(PemCertificateString {
            inner: input.to_string(),
        })
    }
}

impl Default for PemCertificateString {
    fn default() -> Self {
        PemCertificateString {
            inner: "".to_string(),
        }
    }
}

string_impls_for!(PemCertificateString, "PemCertificateString");

#[cfg(test)]
mod test_valid_pem_certificate_string {
    use super::PemCertificateString;

    static TEST_PEM: &str = include_str!("../../tests/data/test-pem");
    static TEST_INCOMPLETE_PEM: &str = include_str!("../../tests/data/test-incomplete-pem");

    #[test]
    fn valid_pem_certificate() {
        assert!((TEST_PEM).parse::<PemCertificateString>().is_ok());
        assert!(("").parse::<PemCertificateString>().is_ok());
    }

    #[test]
    fn invalid_pem_certificate() {
        // PEM with valid markers but with invalid content
        assert!(
            ("LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tIGJhZCAtLS0tLUVORCBDRVJUSUZJQ0FURS0tLS0tCg==")
                .parse::<PemCertificateString>()
                .is_err()
        );
        // PEM with valid content but without footer marker
        assert!((TEST_INCOMPLETE_PEM)
            .parse::<PemCertificateString>()
            .is_err());

        // PEM without any valid certificate
        assert!((
            "77yc44Kz77ya44OfIOOBj+OCszrlvaEg77yc44Kz77ya44OfIOOBj+OCszrlvaEg77yc44Kz77ya44OfCg=="
        ).parse::<PemCertificateString>()
        .is_err())
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KmodKey can only be created by deserializing from a string that contains ASCII
/// alphanumeric characters, plus hyphens, plus underscores. It stores the original
/// form and makes it accessible through standard traits. Its purpose is to validate
/// input that will be treated as a potential kernel module name.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KmodKey {
    inner: String,
}

// This limit is based on the kernel definition, and assumes a 64-bit host.
//   include/linux/module.h
//     #define MODULE_NAME_LEN MAX_PARAM_PREFIX_LEN
//   include/linux/moduleparam.h
//     #define MAX_PARAM_PREFIX_LEN (64 - sizeof(unsigned long))
const KMOD_KEY_LENGTH: usize = 56;

impl FromStr for KmodKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // The kernel allows modules to have any name that's a valid filename,
        // but real module names seem to be limited to this character set.
        let valid_key = input
            .chars()
            .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-' || c == '_')
            && input.len() <= KMOD_KEY_LENGTH;
        ensure!(valid_key, error::InvalidKmodKeySnafu { input });
        Ok(KmodKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KmodKey, "KmodKey");

#[cfg(test)]
mod test_valid_kmod_key {
    use super::{KmodKey, KMOD_KEY_LENGTH};

    #[test]
    fn valid_kmod_key() {
        assert!(("kmod").parse::<KmodKey>().is_ok());
        assert!(("i8042").parse::<KmodKey>().is_ok());
        assert!(("xt_XT").parse::<KmodKey>().is_ok());
        assert!(("dm-block").parse::<KmodKey>().is_ok());
        assert!(("blowfish-x86_64").parse::<KmodKey>().is_ok());
        assert!((&vec!["a"; KMOD_KEY_LENGTH].join(""))
            .parse::<KmodKey>()
            .is_ok());
    }

    #[test]
    fn invalid_kmod_key() {
        assert!(("../").parse::<KmodKey>().is_err());
        assert!(("{}").parse::<KmodKey>().is_err());
        assert!(("kernel|Module").parse::<KmodKey>().is_err());
        assert!(("kernel\nModule").parse::<KmodKey>().is_err());
        assert!(("🐡").parse::<KmodKey>().is_err());
        assert!((&vec!["z"; KMOD_KEY_LENGTH + 1].join(""))
            .parse::<KmodKey>()
            .is_err());
    }
}
