//! The interface_id module contains the definition of a valid network interface name and MAC
//! address and the code to support creation of either structure from string.
//!
//! A valid network interface name is defined by the criteria in the linux kernel:
//! https://elixir.bootlin.com/linux/v5.10.102/source/net/core/dev.c#L1138
use serde::{Deserialize, Serialize, Serializer};
use snafu::{ensure, ResultExt};
use std::convert::TryFrom;
use std::fmt::Display;
use std::ops::Deref;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub(crate) enum InterfaceId {
    Name(InterfaceName),
    MacAddress(MacAddress),
}

impl From<InterfaceName> for InterfaceId {
    fn from(name: InterfaceName) -> Self {
        InterfaceId::Name(name)
    }
}

impl From<MacAddress> for InterfaceId {
    fn from(mac: MacAddress) -> Self {
        InterfaceId::MacAddress(mac)
    }
}

#[allow(clippy::to_string_in_format_args)]
impl Display for InterfaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceId::Name(name) => write!(f, "{}", name.to_string()),
            InterfaceId::MacAddress(mac) => write!(f, "{}", mac.to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct MacAddress {
    inner: String,
}

impl TryFrom<String> for MacAddress {
    type Error = error::Error;

    fn try_from(input: String) -> Result<Self> {
        let mut octets = 0;

        for octet in input.split(|b| b == '-' || b == ':') {
            // If we've gotten to 6 and are still iterating, the MAC is too long
            ensure!(
                octets != 6 && octet.len() == 2,
                error::InvalidMacAddressSnafu {
                    input,
                    msg: "must have 6 octets of 2 chars/digits"
                }
            );

            // Validate the characters in the MAC
            u8::from_str_radix(octet, 16).context(error::InvalidMacAddressCharSnafu {
                input: input.to_string(),
                msg: "invalid character/digit",
            })?;

            octets += 1;
        }

        ensure!(
            octets == 6,
            error::InvalidMacAddressSnafu {
                input,
                msg: "must have 6 octets"
            }
        );

        // Store the MAC internally as lowercase and colon-separated for ease of use later
        Ok(MacAddress {
            inner: input.to_lowercase().replace('-', ":"),
        })
    }
}

impl TryFrom<&str> for MacAddress {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self> {
        Self::try_from(input.to_string())
    }
}

impl Deref for MacAddress {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Serialize for MacAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// InterfaceName can only be created from a string that contains a valid network interface name.
/// Validation is handled in the `TryFrom` implementation below.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct InterfaceName {
    inner: String,
}

impl TryFrom<String> for InterfaceName {
    type Error = error::Error;

    fn try_from(input: String) -> Result<Self> {
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
            error::InvalidNetworkDeviceNameSnafu {
                input,
                msg: "contains line terminators"
            }
        );

        // The length for an interface name is defined here:
        // https://elixir.bootlin.com/linux/v5.10.102/source/include/uapi/linux/if.h#L33
        // The constant definition (16) is a little misleading as the check for it ensures that the
        // name is NOT equal to 16.  A name must be 1-15 characters.
        ensure!(
            !input.is_empty() && input.len() <= 15,
            error::InvalidNetworkDeviceNameSnafu {
                input,
                msg: "invalid length, must be 1 to 15 characters long"
            }
        );

        ensure!(
            !input.contains('.') && !input.contains('/') && !input.contains(char::is_whitespace),
            error::InvalidNetworkDeviceNameSnafu {
                input,
                msg: "contains invalid characters"
            }
        );

        Ok(Self { inner: input })
    }
}

impl TryFrom<&str> for InterfaceName {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self> {
        Self::try_from(input.to_string())
    }
}

impl Deref for InterfaceName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Serialize for InterfaceName {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

impl Display for InterfaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

mod error {
    use snafu::Snafu;
    use std::num::ParseIntError;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    #[allow(clippy::enum_variant_names)]
    pub(crate) enum Error {
        #[snafu(display("Invalid network device name '{}': {}", input, msg))]
        InvalidNetworkDeviceName { input: String, msg: String },

        #[snafu(display("Invalid MAC address '{}': {}", input, msg))]
        InvalidMacAddress { input: String, msg: String },

        #[snafu(display("Invalid MAC address '{}': {}: {}", input, msg, source))]
        InvalidMacAddressChar {
            input: String,
            msg: String,
            source: ParseIntError,
        },
    }
}

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_interface_name() {
        let bad_str = [
            &"a".repeat(16),
            "",
            ".",
            "..",
            "f/eno1",
            "eno 1",
            "eno\n1",
            "\n",
            "\r",
            "\u{000B}",
            "\u{000C}",
            "\u{0085}",
            "\u{2028}",
            "\u{2029}",
        ];
        for bad in bad_str {
            assert!(InterfaceName::try_from(bad).is_err())
        }
    }

    #[test]
    fn valid_interface_name() {
        let ok_str = [&"a".repeat(15), "eno1", "eth0", "enp5s0", "enx0eb36944b633"];
        for ok in ok_str {
            assert!(InterfaceName::try_from(ok).is_ok())
        }
    }

    #[test]
    fn valid_mac_address() {
        let ok_str = [
            "52:54:00:79:99:c6",
            "52-54-00-79-99-c6",
            "F8:75:A4:D5:32:64",
            "F8-75-A4-D5-32-64",
        ];
        for ok in ok_str {
            assert!(MacAddress::try_from(ok).is_ok())
        }
    }

    #[test]
    fn invalid_mac_address() {
        let bad_str = [
            "",
            ":",
            "52:",
            "52:54:00:79:99:c",
            "52:54:00:79:99:c6:c7",
            "52:54:00:79:99:z6",
        ];
        for bad in bad_str {
            assert!(MacAddress::try_from(bad).is_err())
        }
    }
}
