/*!
# Introduction

netdog is a small helper program for wicked, to apply network settings received from DHCP.

It generates `/etc/resolv.conf` and sets the hostname.
*/

// TODO:
// We should rework this to store info in the API and delegate rewriting of
// files and any required process restarts to the existing machinery.
// This is blocked on the ability to apply and commit settings in separate
// transactions; otherwise a lease renewal while settings were being added
// by other processes could cause them to be applied in an incomplete state.

#![deny(rust_2018_idioms)]

use dns_lookup::lookup_addr;
use envy;
use ipnet::IpNet;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use serde::Deserialize;
use snafu::ResultExt;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::{env, process};

static RESOLV_CONF: &str = "/etc/resolv.conf";
static KERNEL_HOSTNAME: &str = "/proc/sys/kernel/hostname";

// Matches wicked's shell-like syntax for DHCP lease variables:
//     FOO='BAR' -> key=FOO, val=BAR
lazy_static! {
    static ref LEASE_PARAM: Regex = Regex::new(r"^(?P<key>[A-Z]+)='(?P<val>.+)'$").unwrap();
}

/// Potential errors during netdog execution
mod error {
    use envy;
    use snafu::Snafu;
    use std::io;
    use std::net::IpAddr;
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

        #[snafu(display("Failed to resolve '{}' to hostname: {}", ip, source))]
        HostnameLookupFailed { ip: IpAddr, source: io::Error },

        #[snafu(display("Failed to write hostname to '{}': {}", path.display(), source))]
        HostnameWriteFailed { path: PathBuf, source: io::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SubCommand {
    Install,
    Remove,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceName {
    Eth0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceType {
    Dhcp,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterfaceFamily {
    Ipv4,
    Ipv6,
}

/// Stores user-supplied arguments.
#[derive(Debug)]
struct Args {
    sub_command: SubCommand,
    interface_name: InterfaceName,
    interface_type: InterfaceType,
    interface_family: InterfaceFamily,
    data_file: PathBuf,
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
    dns_search: Option<String>,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            COMMAND
              -i INTERFACE_NAME
              -t INTERFACE_TYPE
              -f INTERFACE_FAMILY
              DATA_FILE",
        program_name
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parses user arguments into an Args structure.
fn parse_args(args: env::Args) -> Result<Args> {
    let mut iter = args.skip(1);
    let value = iter
        .next()
        .unwrap_or_else(|| usage_msg("Did not specify command"));
    let sub_command = Some(
        serde_plain::from_str::<SubCommand>(&value)
            .unwrap_or_else(|_| usage_msg(format!("Unknown command {}", value))),
    );

    let mut interface_name = None;
    let mut interface_type = None;
    let mut interface_family = None;
    let mut data_file = None;

    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-i" => {
                let value = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -i"));
                interface_name = Some(
                    serde_plain::from_str::<InterfaceName>(&value)
                        .unwrap_or_else(|_| usage_msg(format!("Unknown interface name {}", value))),
                );
            }

            "-t" => {
                let value = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -t"));
                interface_type = Some(
                    serde_plain::from_str::<InterfaceType>(&value)
                        .unwrap_or_else(|_| usage_msg(format!("Unknown interface type {}", value))),
                );
            }

            "-f" => {
                let value = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -f"));
                interface_family = Some(
                    serde_plain::from_str::<InterfaceFamily>(&value).unwrap_or_else(|_| {
                        usage_msg(format!("Unknown interface family {}", value))
                    }),
                );
            }

            // `wicked` may call this with "info" as the final argument, so if
            // we already have a data file then we're done.
            p => match data_file {
                None => data_file = Some(PathBuf::from(p)),
                Some(_) => break,
            },
        }
    }

    Ok(Args {
        sub_command: sub_command.unwrap_or_else(|| usage()),
        interface_name: interface_name.unwrap_or_else(|| usage()),
        interface_type: interface_type.unwrap_or_else(|| usage()),
        interface_family: interface_family.unwrap_or_else(|| usage()),
        data_file: data_file.unwrap_or_else(|| usage()),
    })
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
fn write_resolv_conf(dns_servers: &[&IpAddr], dns_search: &Option<String>) -> Result<()> {
    let mut output = String::new();

    if let Some(s) = dns_search {
        writeln!(output, "search {}", s).context(error::ResolvConfBuildFailed)?;
    }

    for n in dns_servers {
        writeln!(output, "nameserver {}", n).context(error::ResolvConfBuildFailed)?;
    }

    fs::write(RESOLV_CONF, output).context(error::ResolvConfWriteFailed { path: RESOLV_CONF })?;
    Ok(())
}

/// Resolve assigned IP address and persist the result as hostname.
fn update_hostname(ip: &IpNet) -> Result<()> {
    let host =
        lookup_addr(&ip.addr()).with_context(|| error::HostnameLookupFailed { ip: ip.addr() })?;
    fs::write(KERNEL_HOSTNAME, host).context(error::HostnameWriteFailed {
        path: KERNEL_HOSTNAME,
    })?;
    Ok(())
}

fn install(args: &Args) -> Result<()> {
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
            update_hostname(&info.ip_address)?;
        }
        _ => eprintln!("Unhandled 'install' command: {:?}", &args),
    }
    Ok(())
}

fn remove(args: &Args) -> Result<()> {
    match (
        &args.interface_name,
        &args.interface_type,
        &args.interface_family,
    ) {
        _ => eprintln!("The 'remove' command is not implemented."),
    }
    Ok(())
}

fn run() -> Result<()> {
    let args = parse_args(env::args())?;
    match args.sub_command {
        SubCommand::Install => install(&args)?,
        SubCommand::Remove => remove(&args)?,
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
