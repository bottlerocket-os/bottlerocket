/*!
# Introduction

netdog is a small helper program for wicked, to apply network settings received from DHCP.  It
generates `/etc/resolv.conf`, generates and sets the hostname, and persists the current IP to a
file.

It contains two subcommands meant for use as settings generators:
* `node-ip`: returns the node's current IP address in JSON format
* `generate-hostname`: returns the node's hostname in JSON format. If the lookup is unsuccessful, the IP of the node is used.

The subcommand `set-hostname` sets the hostname for the system.

The subcommand `generate-net-config` generates the network interface configuration for the host. If
a `net.toml` file exists in `/var/lib/bottlerocket`, it is used to generate the configuration. If
`net.toml` doesn't exist, the kernel command line `/proc/cmdline` is checked for the prefix
`netdog.default-interface`.  If an interface is defined with that prefix, it is used to generate an
interface configuration.  A single default interface may be defined on the kernel command line with
the format: `netdog.default-interface=interface-name:option1,option2`.  "interface-name" is the
name of the interface, and valid options are "dhcp4" and "dhcp6".  A "?" may be added to the option
to signify that the lease for the protocol is optional and the system shouldn't wait for it.  A
valid example: `netdog.default-interface=eno1:dhcp4,dhcp6?`.

The subcommand `prepare-primary-interface` writes the default sysctls for the primary interface to
file in `/etc/sysctl.d`, and then executes `systemd-sysctl` to apply them.
*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate serde_plain;

mod interface_name;
mod net_config;
mod wicked;

use argh::FromArgs;
use dns_lookup::lookup_addr;
use envy;
use ipnet::IpNet;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::str::FromStr;

static RESOLV_CONF: &str = "/etc/resolv.conf";
static KERNEL_HOSTNAME: &str = "/proc/sys/kernel/hostname";
static CURRENT_IP: &str = "/var/lib/netdog/current_ip";
static KERNEL_CMDLINE: &str = "/proc/cmdline";
static PRIMARY_INTERFACE: &str = "/var/lib/netdog/primary_interface";
static DEFAULT_NET_CONFIG_FILE: &str = "/var/lib/bottlerocket/net.toml";
static PRIMARY_SYSCTL_CONF: &str = "/etc/sysctl.d/90-primary_interface.conf";
static SYSTEMD_SYSCTL: &str = "/usr/lib/systemd/systemd-sysctl";

// Matches wicked's shell-like syntax for DHCP lease variables:
//     FOO='BAR' -> key=FOO, val=BAR
lazy_static! {
    static ref LEASE_PARAM: Regex = Regex::new(r"^(?P<key>[A-Z]+)='(?P<val>.+)'$").unwrap();
}

/// Stores fields extracted from a DHCP lease.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
derive_fromstr_from_deserialize!(InterfaceType);
derive_fromstr_from_deserialize!(InterfaceFamily);

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
    GenerateNetConfig(GenerateNetConfigArgs),
    SetHostname(SetHostnameArgs),
    PreparePrimaryInterface(PreparePrimaryInterfaceArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "install")]
/// Write resolv.conf and current IP to disk
struct InstallArgs {
    #[argh(option, short = 'i')]
    /// name of the network interface
    interface_name: String,

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
    interface_name: String,

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
#[argh(subcommand, name = "generate-net-config")]
/// Generate wicked network configuration
struct GenerateNetConfigArgs {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set-hostname")]
/// Sets the hostname
struct SetHostnameArgs {
    #[argh(positional)]
    /// hostname for the system
    hostname: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "prepare-primary-interface")]
/// Sets the default sysctls for the primary interface
struct PreparePrimaryInterfaceArgs {}

/// Parse lease data file into a LeaseInfo structure.
fn parse_lease_info<P>(lease_file: P) -> Result<LeaseInfo>
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
                env.push((k.to_string(), v.replace(" ", ",")))
            }
        }
    }

    // Envy implements a serde `Deserializer` for an iterator of key/value pairs. That lets us
    // feed in the key/value pairs from the lease file and get a `LeaseInfo` struct. If not all
    // expected values are present in the file, it will fail; any extra values are ignored.
    Ok(envy::from_iter::<_, LeaseInfo>(env)
        .context(error::LeaseParseFailedSnafu { path: lease_file })?)
}

/// Write resolver configuration for libc.
fn write_resolv_conf(dns_servers: &[&IpAddr], dns_search: &Option<Vec<String>>) -> Result<()> {
    let mut output = String::new();

    if let Some(s) = dns_search {
        writeln!(output, "search {}", s.join(" ")).context(error::ResolvConfBuildFailedSnafu)?;
    }

    for n in dns_servers {
        writeln!(output, "nameserver {}", n).context(error::ResolvConfBuildFailedSnafu)?;
    }

    fs::write(RESOLV_CONF, output)
        .context(error::ResolvConfWriteFailedSnafu { path: RESOLV_CONF })?;
    Ok(())
}

/// Persist the current IP address to file
fn write_current_ip(ip: &IpAddr) -> Result<()> {
    fs::write(CURRENT_IP, ip.to_string())
        .context(error::CurrentIpWriteFailedSnafu { path: CURRENT_IP })
}

fn install(args: InstallArgs) -> Result<()> {
    // Wicked doesn't mangle interface names, but let's be defensive.
    let install_interface = args.interface_name.trim().to_lowercase();
    let primary_interface = fs::read_to_string(PRIMARY_INTERFACE)
        .context(error::PrimaryInterfaceReadSnafu {
            path: PRIMARY_INTERFACE,
        })?
        .trim()
        .to_lowercase();

    if install_interface != primary_interface {
        return Ok(());
    }

    match (&args.interface_type, &args.interface_family) {
        (InterfaceType::Dhcp, InterfaceFamily::Ipv4) => {
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
    let ip_string = fs::read_to_string(CURRENT_IP)
        .context(error::CurrentIpReadFailedSnafu { path: CURRENT_IP })?;
    // Validate that we read a proper IP address
    let _ = IpAddr::from_str(&ip_string).context(error::IpFromStringSnafu { ip: &ip_string })?;

    // sundog expects JSON-serialized output
    Ok(print_json(ip_string)?)
}

/// Attempt to resolve assigned IP address, if unsuccessful use the IP as the hostname.
///
/// The result is returned as JSON. (intended for use as a settings generator)
fn generate_hostname() -> Result<()> {
    let ip_string = fs::read_to_string(CURRENT_IP)
        .context(error::CurrentIpReadFailedSnafu { path: CURRENT_IP })?;
    let ip = IpAddr::from_str(&ip_string).context(error::IpFromStringSnafu { ip: &ip_string })?;
    let hostname = match lookup_addr(&ip) {
        Ok(hostname) => hostname,
        Err(e) => {
            eprintln!("Reverse DNS lookup failed: {}", e);
            ip_string
        }
    };

    // sundog expects JSON-serialized output
    Ok(print_json(hostname)?)
}

/// Generate configuration for network interfaces.
fn generate_net_config() -> Result<()> {
    let maybe_net_config = if Path::exists(Path::new(DEFAULT_NET_CONFIG_FILE)) {
        net_config::from_path(DEFAULT_NET_CONFIG_FILE).context(error::NetConfigParseSnafu {
            path: DEFAULT_NET_CONFIG_FILE,
        })?
    } else {
        net_config::from_command_line(KERNEL_CMDLINE).context(error::NetConfigParseSnafu {
            path: KERNEL_CMDLINE,
        })?
    };

    // `maybe_net_config` could be `None` if no interfaces were defined
    let net_config = match maybe_net_config {
        Some(net_config) => net_config,
        None => {
            eprintln!("No network interfaces were configured");
            return Ok(());
        }
    };

    let primary_interface = net_config
        .primary_interface()
        .context(error::GetPrimaryInterfaceSnafu)?;
    write_primary_interface(primary_interface)?;

    let wicked_interfaces = net_config.into_wicked_interfaces();
    for interface in wicked_interfaces {
        interface
            .write_config_file()
            .context(error::InterfaceConfigWriteSnafu)?;
    }
    Ok(())
}

/// Persist the primary interface name to file
fn write_primary_interface<S>(interface: S) -> Result<()>
where
    S: AsRef<str>,
{
    let interface = interface.as_ref();
    fs::write(PRIMARY_INTERFACE, interface).context(error::PrimaryInterfaceWriteSnafu {
        path: PRIMARY_INTERFACE,
    })
}

/// Helper function that serializes the input to JSON and prints it
fn print_json<S>(val: S) -> Result<()>
where
    S: AsRef<str> + Serialize,
{
    let val = val.as_ref();
    let output = serde_json::to_string(val).context(error::JsonSerializeSnafu { output: val })?;
    println!("{}", output);
    Ok(())
}

/// Sets the hostname for the system
fn set_hostname(args: SetHostnameArgs) -> Result<()> {
    fs::write(KERNEL_HOSTNAME, args.hostname).context(error::HostnameWriteFailedSnafu {
        path: KERNEL_HOSTNAME,
    })?;
    Ok(())
}

/// Set and apply default sysctls for the primary network interface
fn prepare_primary_interface() -> Result<()> {
    let primary_interface =
        fs::read_to_string(PRIMARY_INTERFACE).context(error::PrimaryInterfaceReadSnafu {
            path: PRIMARY_INTERFACE,
        })?;
    write_interface_sysctl(primary_interface, PRIMARY_SYSCTL_CONF)?;

    // Execute `systemd-sysctl` with our configuration file to set the sysctls
    let systemd_sysctl_result = Command::new(SYSTEMD_SYSCTL)
        .arg(PRIMARY_SYSCTL_CONF)
        .output()
        .context(error::SystemdSysctlExecutionSnafu)?;
    ensure!(
        systemd_sysctl_result.status.success(),
        error::FailedSystemdSysctlSnafu {
            stderr: String::from_utf8_lossy(&systemd_sysctl_result.stderr)
        }
    );
    Ok(())
}

/// Write the default sysctls for a given interface to a given path
fn write_interface_sysctl<S, P>(interface: S, path: P) -> Result<()>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let interface = interface.as_ref();
    let path = path.as_ref();
    // TODO if we accumulate more of these we should have a better way to create than format!()
    // Note: The dash (-) preceding the "net..." variable assignment below is important; it
    // ensures failure to set the variable for any reason will be logged, but not cause the sysctl
    // service to fail
    // Accept router advertisement (RA) packets even if IPv6 forwarding is enabled on interface
    let ipv6_accept_ra = format!("-net.ipv6.conf.{}.accept_ra = 2", interface);
    // Enable loose mode for reverse path filter
    let ipv4_rp_filter = format!("-net.ipv4.conf.{}.rp_filter = 2", interface);

    let mut output = String::new();
    writeln!(output, "{}", ipv6_accept_ra).context(error::SysctlConfBuildSnafu)?;
    writeln!(output, "{}", ipv4_rp_filter).context(error::SysctlConfBuildSnafu)?;

    fs::write(path, output).context(error::SysctlConfWriteSnafu { path })?;
    Ok(())
}

fn run() -> Result<()> {
    let args: Args = argh::from_env();
    match args.subcommand {
        SubCommand::Install(args) => install(args)?,
        SubCommand::Remove(args) => remove(args)?,
        SubCommand::NodeIp(_) => node_ip()?,
        SubCommand::GenerateHostname(_) => generate_hostname()?,
        SubCommand::GenerateNetConfig(_) => generate_net_config()?,
        SubCommand::SetHostname(args) => set_hostname(args)?,
        SubCommand::PreparePrimaryInterface(_) => prepare_primary_interface()?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sysctls() {
        let interface = "eno1";
        let fake_file = tempfile::NamedTempFile::new().unwrap();
        let expected = "-net.ipv6.conf.eno1.accept_ra = 2\n-net.ipv4.conf.eno1.rp_filter = 2\n";
        write_interface_sysctl(interface, &fake_file).unwrap();
        assert_eq!(std::fs::read_to_string(&fake_file).unwrap(), expected);
    }
}

/// Potential errors during netdog execution
mod error {
    use crate::{net_config, wicked};
    use envy;
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
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

        #[snafu(display("Unable to read/parse network config from '{}': {}", path.display(), source))]
        NetConfigParse {
            path: PathBuf,
            source: net_config::Error,
        },

        #[snafu(display("Failed to write network interface configuration: {}", source))]
        InterfaceConfigWrite { source: wicked::Error },

        #[snafu(display("Failed to write primary interface to '{}': {}", path.display(), source))]
        PrimaryInterfaceWrite { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to read primary interface from '{}': {}", path.display(), source))]
        PrimaryInterfaceRead { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to discern primary interface"))]
        GetPrimaryInterface,

        #[snafu(display("Failed to build sysctl config: {}", source))]
        SysctlConfBuild { source: std::fmt::Error },

        #[snafu(display("Failed to write sysctl config to '{}': {}", path.display(), source))]
        SysctlConfWrite { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to run 'systemd-sysctl': {}", source))]
        SystemdSysctlExecution { source: io::Error },

        #[snafu(display("'systemd-sysctl' failed: {}", stderr))]
        FailedSystemdSysctl { stderr: String },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
