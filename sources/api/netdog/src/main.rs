/*!
# Introduction

netdog is a small helper program for wicked, to apply network settings received from DHCP.  It
generates `/etc/resolv.conf`, generates and sets the hostname, and persists the current IP to a
file.

It contains two subcommands meant for use as settings generators:
* `node-ip`: returns the node's current IP address in JSON format
* `generate-hostname`: returns the node's hostname in JSON format (it is the resolved IP or the IP
  in format "ip-x-x-x-x" if resolving fails)

The subcommand `set-hostname` sets the hostname for the system.
*/

// TODO:
// We should rework this to store info in the API and delegate rewriting of
// files and any required process restarts to the existing machinery.
// This is blocked on the ability to apply and commit settings in separate
// transactions; otherwise a lease renewal while settings were being added
// by other processes could cause them to be applied in an incomplete state.

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate serde_plain;

use argh::FromArgs;
use dns_lookup::lookup_addr;
use envy;
use ipnet::IpNet;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;

static RESOLV_CONF: &str = "/etc/resolv.conf";
static KERNEL_HOSTNAME: &str = "/proc/sys/kernel/hostname";
static CURRENT_IP: &str = "/var/lib/netdog/current_ip";

// Matches wicked's shell-like syntax for DHCP lease variables:
//     FOO='BAR' -> key=FOO, val=BAR
lazy_static! {
    static ref LEASE_PARAM: Regex = Regex::new(r"^(?P<key>[A-Z]+)='(?P<val>.+)'$").unwrap();
}

/// Stores fields extracted from a DHCP lease.
#[derive(Debug, Deserialize)]
struct LeaseInfo {
    #[serde(rename = "ipaddr")]
    ip_address: IpNet,
    #[serde(rename = "dnsservers")]
    dns_servers: BTreeSet<IpAddr>,
    #[serde(rename = "dnsdomain")]
    dns_domain: Option<String>,
    #[serde(rename = "dnssearch")]
    dns_search: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceName {
    Eth0,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceType {
    Dhcp,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceFamily {
    Ipv4,
    Ipv6,
}

// Implement `from_str()` so argh can attempt to deserialize args into their proper types
forward_from_str_to_serde!(InterfaceName);
forward_from_str_to_serde!(InterfaceType);
forward_from_str_to_serde!(InterfaceFamily);

/// Stores user-supplied arguments.
#[derive(FromArgs, PartialEq, Debug)]
struct Args {
    #[argh(subcommand)]
    subcommand: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    Install(InstallArgs),
    Remove(RemoveArgs),
    NodeIp(NodeIpArgs),
    GenerateHostname(GenerateHostnameArgs),
    SetHostname(SetHostnameArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "install")]
/// Write resolv.conf and current IP to disk
struct InstallArgs {
    #[argh(option, short = 'i')]
    /// name of the network interface
    interface_name: InterfaceName,

    #[argh(option, short = 't')]
    /// network interface type
    interface_type: InterfaceType,

    #[argh(option, short = 'f')]
    /// network interface family (ipv4/6)
    interface_family: InterfaceFamily,

    #[argh(positional)]
    /// lease info data file
    data_file: PathBuf,

    #[argh(positional)]
    // wicked adds `info` to the call to this program.  We don't do anything with it but must
    // be able to parse the option to avoid failing
    /// ignored
    info: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "remove")]
// `wicked` calls `remove` with the below args and failing to parse them can cause an error in
// `wicked`.
/// Does nothing
struct RemoveArgs {
    #[argh(option, short = 'i')]
    /// name of the network interface
    interface_name: InterfaceName,

    #[argh(option, short = 't')]
    /// network interface type
    interface_type: InterfaceType,

    #[argh(option, short = 'f')]
    /// network interface family (ipv4/6)
    interface_family: InterfaceFamily,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "node-ip")]
/// Return the current IP address
struct NodeIpArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-hostname")]
/// Generate hostname from DNS reverse lookup or use current IP
struct GenerateHostnameArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set-hostname")]
/// Sets the hostname
struct SetHostnameArgs {
    #[argh(positional)]
    /// hostname for the system
    hostname: String,
}

/// Parse lease data file into a LeaseInfo structure.
fn parse_lease_info<P>(lease_file: P) -> Result<LeaseInfo>
where
    P: AsRef<Path>,
{
    let lease_file = lease_file.as_ref();
    let f = File::open(lease_file).context(error::LeaseReadFailed { path: lease_file })?;
    let f = BufReader::new(f);

    let mut env = Vec::new();
    for line in f.lines() {
        let line = line.context(error::LeaseReadFailed { path: lease_file })?;
        // We ignore any line that does not match the regex.
        for cap in LEASE_PARAM.captures_iter(&line) {
            let key = cap.name("key").map(|k| k.as_str());
            let val = cap.name("val").map(|v| v.as_str());
            if let (Some(k), Some(v)) = (key, val) {
                // If present, replace spaces with commas so Envy deserializes into a list.
                env.push((k.to_string(), v.replace(" ", ",")))
            }
        }
    }

    // Envy implements a serde `Deserializer` for an iterator of key/value pairs. That lets us
    // feed in the key/value pairs from the lease file and get a `LeaseInfo` struct. If not all
    // expected values are present in the file, it will fail; any extra values are ignored.
    Ok(envy::from_iter::<_, LeaseInfo>(env)
        .context(error::LeaseParseFailed { path: lease_file })?)
}

/// Write resolver configuration for libc.
fn write_resolv_conf(dns_servers: &[&IpAddr], dns_search: &Option<Vec<String>>) -> Result<()> {
    let mut output = String::new();

    if let Some(s) = dns_search {
        writeln!(output, "search {}", s.join(" ")).context(error::ResolvConfBuildFailed)?;
    }

    for n in dns_servers {
        writeln!(output, "nameserver {}", n).context(error::ResolvConfBuildFailed)?;
    }

    fs::write(RESOLV_CONF, output).context(error::ResolvConfWriteFailed { path: RESOLV_CONF })?;
    Ok(())
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string()).context(error::CurrentIpWriteFailed { path: CURRENT_IP })
}

fn install(args: InstallArgs) -> Result<()> {
    match (
        &args.interface_name,
        &args.interface_type,
        &args.interface_family,
    ) {
        (InterfaceName::Eth0, InterfaceType::Dhcp, InterfaceFamily::Ipv4) => {
            let info = parse_lease_info(&args.data_file)?;
            // Randomize name server order, for libc implementations like musl that send
            // queries to the first N servers.
            let mut dns_servers: Vec<_> = info.dns_servers.iter().collect();
            dns_servers.shuffle(&mut thread_rng());
            write_resolv_conf(&dns_servers, &info.dns_search)?;
            write_current_ip(&info.ip_address.addr())?;
        }
        _ => eprintln!("Unhandled 'install' command: {:?}", &args),
    }
    Ok(())
}

fn remove(args: RemoveArgs) -> Result<()> {
    match (
        &args.interface_name,
        &args.interface_type,
        &args.interface_family,
    ) {
        _ => eprintln!("The 'remove' command is not implemented."),
    }
    Ok(())
}

/// Return the current IP address as JSON (intended for use as a settings generator)
fn node_ip() -> Result<()> {
    let ip =
        fs::read_to_string(CURRENT_IP).context(error::CurrentIpReadFailed { path: CURRENT_IP })?;
    // sundog expects JSON-serialized output
    Ok(print_json(ip)?)
}

/// Attempt to resolve assigned IP address, if unsuccessful use "ip-X-X-X-X" where X's are the
/// octets of the IP.  For example, IP address 1.2.3.4 becomes the hostname "ip-1-2-3-4".  No dots
/// in the hostname (hopefully) avoids any confusion for programs that may read this value.
///
/// The result is returned as JSON. (intended for use as a settings generator)
fn generate_hostname() -> Result<()> {
    let ip_string =
        fs::read_to_string(CURRENT_IP).context(error::CurrentIpReadFailed { path: CURRENT_IP })?;
    let ip = IpAddr::from_str(&ip_string).context(error::IpFromString { ip: &ip_string })?;
    let hostname = match lookup_addr(&ip) {
        Ok(hostname) => {
            // if `lookup_addr()` returns the same IP as we passed it we didn't resolve anything,
            // so return the string "ip-x-x-x-x" in this case
            if hostname == ip_string {
                format!("ip-{}", ip_string.replace(".", "-"))
            } else {
                hostname
            }
        }
        Err(e) => {
            eprintln!("Reverse DNS lookup failed: {}", e);
            format!("ip-{}", ip_string.replace(".", "-"))
        }
    };

    // sundog expects JSON-serialized output
    Ok(print_json(hostname)?)
}

/// Helper function that serializes the input to JSON and prints it
fn print_json<S>(val: S) -> Result<()>
where
    S: AsRef<str> + Serialize,
{
    let val = val.as_ref();
    let output = serde_json::to_string(val).context(error::JsonSerialize { output: val })?;
    println!("{}", output);
    Ok(())
}

/// Sets the hostname for the system
fn set_hostname(args: SetHostnameArgs) -> Result<()> {
    fs::write(KERNEL_HOSTNAME, args.hostname).context(error::HostnameWriteFailed {
        path: KERNEL_HOSTNAME,
    })?;
    Ok(())
}

fn run() -> Result<()> {
    let args: Args = argh::from_env();
    match args.subcommand {
        SubCommand::Install(args) => install(args)?,
        SubCommand::Remove(args) => remove(args)?,
        SubCommand::NodeIp(_) => node_ip()?,
        SubCommand::GenerateHostname(_) => generate_hostname()?,
        SubCommand::SetHostname(args) => set_hostname(args)?,
    }
    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

/// Potential errors during netdog execution
mod error {
    use envy;
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    #[allow(clippy::enum_variant_names)]
    pub(super) enum Error {
        #[snafu(display("Failed to read lease data in '{}': {}", path.display(), source))]
        LeaseReadFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to parse lease data in '{}': {}", path.display(), source))]
        LeaseParseFailed { path: PathBuf, source: envy::Error },

        #[snafu(display("Failed to build resolver configuration: {}", source))]
        ResolvConfBuildFailed { source: std::fmt::Error },

        #[snafu(display("Failed to write resolver configuration to '{}': {}", path.display(), source))]
        ResolvConfWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to write hostname to '{}': {}", path.display(), source))]
        HostnameWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Invalid IP address '{}': {}", ip, source))]
        IpFromString {
            ip: String,
            source: std::net::AddrParseError,
        },

        #[snafu(display("Failed to write current IP to '{}': {}", path.display(), source))]
        CurrentIpWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to read current IP data in '{}': {}", path.display(), source))]
        CurrentIpReadFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Error serializing to JSON: '{}': {}", output, source))]
        JsonSerialize {
            output: String,
            source: serde_json::error::Error,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
