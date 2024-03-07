//! The lease module contains the struct and code needed to parse a wicked DHCP lease file
use crate::LEASE_DIR;
use ipnet::IpNet;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use snafu::ResultExt;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

// Matches wicked's shell-like syntax for DHCP lease variables:
//     FOO='BAR' -> key=FOO, val=BAR
lazy_static! {
    static ref LEASE_PARAM: Regex = Regex::new(r"^(?P<key>[A-Z]+)='(?P<val>.+)'$").unwrap();
}

/// Stores fields extracted from a DHCP lease.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct LeaseInfo {
    // When multiple IP addresses exist for an interface, the second address's key in the lease
    // file will be `IPADDR_1`, `IPADDR_2`, and so on.  Parsing the lease for "ipaddr" means we
    // will always pick up the first configured IP address.
    #[serde(rename = "ipaddr")]
    pub(crate) ip_address: IpNet,
    #[serde(rename = "dnsservers")]
    pub(crate) dns_servers: Option<BTreeSet<IpAddr>>,
    #[serde(rename = "dnsdomain")]
    pub(crate) dns_domain: Option<String>,
    #[serde(rename = "dnssearch")]
    pub(crate) dns_search: Option<Vec<String>>,
}

impl LeaseInfo {
    /// Parse lease data file into a LeaseInfo structure.
    pub(crate) fn from_lease<P>(lease_file: P) -> Result<LeaseInfo>
    where
        P: AsRef<Path>,
    {
        let lease_file = lease_file.as_ref();
        let f = File::open(lease_file).context(error::LeaseReadFailedSnafu { path: lease_file })?;
        let f = BufReader::new(f);

        let mut env = Vec::new();
        for line in f.lines() {
            let line = line.context(error::LeaseReadFailedSnafu { path: lease_file })?;
            // We ignore any line that does not match the regex.
            for cap in LEASE_PARAM.captures_iter(&line) {
                let key = cap.name("key").map(|k| k.as_str());
                let val = cap.name("val").map(|v| v.as_str());
                if let (Some(k), Some(v)) = (key, val) {
                    // If present, replace spaces with commas so Envy deserializes into a list.
                    env.push((k.to_string(), v.replace(' ', ",")))
                }
            }
        }

        // Envy implements a serde `Deserializer` for an iterator of key/value pairs. That lets us
        // feed in the key/value pairs from the lease file and get a `LeaseInfo` struct. If not all
        // expected values are present in the file, it will fail; any extra values are ignored.
        envy::from_iter::<_, LeaseInfo>(env)
            .context(error::LeaseParseFailedSnafu { path: lease_file })
    }
}

/// Return the path to a given interface's DHCP ipv4/ipv6 lease if it exists, favoring ipv4 if both
/// ipv4 and ipv6 exist
pub(crate) fn dhcp_lease_path<S>(interface: S) -> Option<PathBuf>
where
    S: AsRef<str>,
{
    get_lease_path("dhcp", interface)
}

/// Return the path to a given interface's static ipv4/ipv6 lease if it exists, favoring ipv4 if
/// both ipv4 and ipv6 exist
pub(crate) fn static_lease_path<S>(interface: S) -> Option<PathBuf>
where
    S: AsRef<str>,
{
    get_lease_path("static", interface)
}

/// Given a lease type and interface, return the path to the ipv4/6 lease file if it exists,
/// favoring ipv4 if both ipv4 and ipv6 exist
fn get_lease_path<S1, S2>(lease_type: S1, interface: S2) -> Option<PathBuf>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let lease_type = lease_type.as_ref();
    let interface = interface.as_ref();

    let ipv4 = Path::new(LEASE_DIR).join(format!("leaseinfo.{}.{}.ipv4", interface, lease_type));
    let ipv6 = Path::new(LEASE_DIR).join(format!("leaseinfo.{}.{}.ipv6", interface, lease_type));

    // If both ipv4 and ipv6 leases exist, use the ipv4 lease for DNS settings
    let ipv4_exists = Path::exists(&ipv4);
    let ipv6_exists = Path::exists(&ipv6);
    match (ipv4_exists, ipv6_exists) {
        (true, true) => Some(ipv4),
        (true, false) => Some(ipv4),
        (false, true) => Some(ipv6),
        (false, false) => None,
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to parse lease data in '{}': {}", path.display(), source))]
        LeaseParseFailed { path: PathBuf, source: envy::Error },

        #[snafu(display("Failed to read lease data in '{}': {}", path.display(), source))]
        LeaseReadFailed { path: PathBuf, source: io::Error },
    }
}

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
