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
    #[serde(rename = "ipaddr")]
    pub(crate) ip_address: IpNet,
    #[serde(rename = "dnsservers")]
    pub(crate) dns_servers: BTreeSet<IpAddr>,
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

/// Return the path to a given interface's ipv4/ipv6 lease if it exists, favoring ipv4 if both
/// ipv4 and ipv6 exist
pub(crate) fn lease_path<S>(interface: S) -> Option<PathBuf>
where
    S: AsRef<str>,
{
    let interface = interface.as_ref();
    let ipv4 = Path::new(LEASE_DIR).join(format!("leaseinfo.{}.dhcp.ipv4", interface));
    let ipv6 = Path::new(LEASE_DIR).join(format!("leaseinfo.{}.dhcp.ipv6", interface));

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
