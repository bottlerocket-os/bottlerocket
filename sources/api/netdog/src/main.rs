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

The subcommand `write-resolv-conf` writes the resolv.conf, favoring DNS API settings and
supplementing any missing settings with DNS settings from the primary interface's DHCP lease.  It
is meant to be used as a restart command for DNS API settings.
*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate serde_plain;

mod cli;
mod dns;
mod interface_name;
mod lease;
mod net_config;
mod wicked;

use argh::FromArgs;
use std::process;

static RESOLV_CONF: &str = "/etc/resolv.conf";
static KERNEL_HOSTNAME: &str = "/proc/sys/kernel/hostname";
static CURRENT_IP: &str = "/var/lib/netdog/current_ip";
static KERNEL_CMDLINE: &str = "/proc/cmdline";
static PRIMARY_INTERFACE: &str = "/var/lib/netdog/primary_interface";
static DEFAULT_NET_CONFIG_FILE: &str = "/var/lib/bottlerocket/net.toml";
static PRIMARY_SYSCTL_CONF: &str = "/etc/sysctl.d/90-primary_interface.conf";
static SYSTEMD_SYSCTL: &str = "/usr/lib/systemd/systemd-sysctl";
static LEASE_DIR: &str = "/run/wicked";

/// Stores user-supplied arguments.
#[derive(FromArgs, PartialEq, Debug)]
struct Args {
    #[argh(subcommand)]
    subcommand: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    Install(cli::InstallArgs),
    Remove(cli::RemoveArgs),
    NodeIp(cli::NodeIpArgs),
    GenerateHostname(cli::GenerateHostnameArgs),
    GenerateNetConfig(cli::GenerateNetConfigArgs),
    SetHostname(cli::SetHostnameArgs),
    PreparePrimaryInterface(cli::PreparePrimaryInterfaceArgs),
    WriteResolvConf(cli::WriteResolvConfArgs),
}

fn run() -> cli::Result<()> {
    let args: Args = argh::from_env();
    match args.subcommand {
        SubCommand::Install(args) => cli::install::run(args)?,
        SubCommand::Remove(args) => cli::remove::run(args)?,
        SubCommand::NodeIp(_) => cli::node_ip::run()?,
        SubCommand::GenerateHostname(_) => cli::generate_hostname::run()?,
        SubCommand::GenerateNetConfig(_) => cli::generate_net_config::run()?,
        SubCommand::SetHostname(args) => cli::set_hostname::run(args)?,
        SubCommand::PreparePrimaryInterface(_) => cli::prepare_primary_interface::run()?,
        SubCommand::WriteResolvConf(_) => cli::write_resolv_conf::run()?,
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
