//! The networkd_status module contains definitions and functions for tracking network status from networkd
//!
//! The primary purpose of this module is to provide structures to read in a `networkctl status` command output
//! and make specific fields available from this output. These structs can then be used to read DNS, IP Addressing
//! and any other networking status data for use in configuration files needed for other networking tools.
use crate::interface_id::InterfaceName;
use crate::NETWORKCTL;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use snafu::{ensure, ResultExt};
use std::convert::TryInto;
use std::net::IpAddr;
use std::process::Command;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct NetworkDInterfaceStatus {
    pub(crate) name: InterfaceName,
    #[serde(rename = "Addresses", deserialize_with = "from_networkctl_addresses")]
    pub(crate) addresses: Vec<IpAddr>,
}

// get an IpAddr from a Vec<u8> (could be 4 or 16 length)
fn ipaddr_from_vec(address_vec: Vec<u8>) -> Result<IpAddr> {
    match address_vec.len() {
        // Already checked that its exactly 4 u8 long
        4 => Ok(IpAddr::from(
            TryInto::<[u8; 4]>::try_into(address_vec).expect("4 bytes"),
        )),
        // Already checked that its exactly 16 u8 long
        16 => Ok(IpAddr::from(
            TryInto::<[u8; 16]>::try_into(address_vec).expect("16 bytes"),
        )),
        _ => error::BadIpAddressSnafu {
            input: address_vec,
            msg: "invalid length, must be 4 or 16 octets".to_string(),
        }
        .fail(),
    }
}

fn from_networkctl_addresses<'de, D>(deserializer: D) -> std::result::Result<Vec<IpAddr>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct NetworkctlAddress {
        address: Vec<u8>,
    }
    let addresses: Vec<NetworkctlAddress> = Deserialize::deserialize(deserializer)?;
    let mut addrs = Vec::new();
    for addr in addresses.iter() {
        addrs.push(ipaddr_from_vec(addr.address.clone()).map_err(D::Error::custom)?);
    }
    Ok(addrs)
}

impl NetworkDInterfaceStatus {
    pub(crate) fn new(link: String) -> Result<Self> {
        let systemd_networkctl_result = Command::new(NETWORKCTL)
            .arg("status")
            .arg("--json=pretty")
            .arg(link)
            .output()
            .context(error::NetworkctlExecutionSnafu)?;
        ensure!(
            systemd_networkctl_result.status.success(),
            error::FailedNetworkctlSnafu {
                stderr: String::from_utf8_lossy(&systemd_networkctl_result.stderr)
            }
        );

        let networkd_status = serde_json::from_slice(&systemd_networkctl_result.stdout)
            .context(error::NetworkctlDeserializeSnafu {})?;

        Ok(networkd_status)
    }

    // Fetches the IP Address for the primary interface. If there are no addresses, this is
    // an error. If there is one, it should be returned. If there is more than one, then find
    // the first IPv4 address and return it, if there are none, return the first IPv6 address.
    pub(crate) fn primary_address(&self) -> Result<IpAddr> {
        // Find IPv4 first
        for addr in self.addresses.iter() {
            if addr.is_ipv4() {
                return Ok(*addr);
            }
        }
        // No IPv4 addresses, then attempt to return the first IPv6, otherwise return error
        match self.addresses.first() {
            Some(addr) => Ok(*addr),
            None => error::NoIpAddressSnafu {
                interface: self.name.clone(),
            }
            .fail(),
        }
    }
}

mod error {
    use crate::interface_id::InterfaceId;
    use snafu::Snafu;
    use std::{io, string::FromUtf8Error};

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to run 'networkctl': {}", source))]
        NetworkctlExecution { source: io::Error },

        #[snafu(display("'networkctl' failed: {}", stderr))]
        FailedNetworkctl { stderr: String },

        #[snafu(display("Failed to parse IP Address: {:?} {}", input, msg))]
        BadIpAddress { input: Vec<u8>, msg: String },

        #[snafu(display("No IP Address for Primary Interface: {:?}", interface))]
        NoIpAddress { interface: InterfaceId },

        #[snafu(display("Failed to parse 'networkctl' output: {}", source))]
        NetworkctlParsing { source: FromUtf8Error },

        #[snafu(display("Failed to deserialize 'networkctl' output: {}", source))]
        NetworkctlDeserialize { source: serde_json::Error },
    }
}

pub(crate) use error::Error as NetworkDStatusError;
type Result<T> = std::result::Result<T, NetworkDStatusError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::net::Ipv4Addr;
    use std::path::Path;
    use std::path::PathBuf;

    fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
    }

    fn networkd_config() -> PathBuf {
        test_data().join("networkd")
    }

    fn read_output_file<P>(path: P) -> String
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        fs::read_to_string(path).unwrap()
    }

    // full deserialize test
    #[test]
    fn no_search_domains_networkd_link_status() {
        let file_name = "no_search_domains_networkctl_output.json";
        let ok = networkd_config().join(file_name);
        let network_status_str = read_output_file(ok);
        let status_output = String::from_utf8(network_status_str.into()).unwrap();

        // Parses correctly
        let network_status_result = serde_json::from_str::<NetworkDInterfaceStatus>(&status_output);
        assert!(network_status_result.is_ok());

        // Primary address is correct for file
        let network_status = network_status_result.unwrap();
        assert_eq!(
            network_status.primary_address().unwrap(),
            Ipv4Addr::new(10, 0, 2, 15)
        );
    }

    #[test]
    fn with_search_domains_networkd_link_status() {
        let file_name = "has_search_domains_networkctl_output.json";
        let ok = networkd_config().join(file_name);
        let network_status_str = read_output_file(ok);
        let status_output: String = String::from_utf8(network_status_str.into()).unwrap();

        // Parses correctly
        let network_status_result = serde_json::from_str::<NetworkDInterfaceStatus>(&status_output);
        assert!(network_status_result.is_ok());

        // Primary address is correct for file
        let network_status = network_status_result.unwrap();
        assert_eq!(
            network_status.primary_address().unwrap(),
            Ipv4Addr::new(172, 31, 28, 92)
        );
    }

    #[test]
    fn valid_ipv4addr_from_vec() {
        let ok_vec: Vec<Vec<u8>> = vec![vec![172, 1, 2, 2], vec![0, 0, 0, 0]];
        for ok in ok_vec {
            assert!(ipaddr_from_vec(ok).is_ok())
        }
    }

    #[test]
    fn valid_ip6addr_from_vec() {
        let ok_vec = vec![
            vec![254, 128, 0, 0, 0, 0, 0, 0, 80, 84, 0, 255, 254, 18, 52, 86],
            vec![254, 128, 0, 0, 0, 0, 0, 0, 4, 8, 13, 255, 254, 137, 48, 197],
        ];
        for ok in ok_vec {
            assert!(ipaddr_from_vec(ok).is_ok())
        }
    }

    #[test]
    fn invalid_ipaddr_from_vec() {
        let bad_vec = vec![
            vec![0],
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        ];
        for bad in bad_vec {
            assert!(ipaddr_from_vec(bad).is_err())
        }
    }
}
