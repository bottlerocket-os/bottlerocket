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

The subcommand `write-resolv-conf` writes the resolv.conf, favoring DNS API settings and
supplementing any missing settings with DNS settings from the primary interface's DHCP lease.  It
is meant to be used as a restart command for DNS API settings.
*/

#[macro_use]
extern crate serde_plain;

mod addressing;
mod bonding;
mod cli;
mod dns;
mod interface_id;
mod net_config;
mod vlan_id;

#[cfg(net_backend = "wicked")]
mod lease;
#[cfg(net_backend = "wicked")]
mod wicked;

#[cfg(net_backend = "systemd-networkd")]
mod networkd;
#[cfg(net_backend = "systemd-networkd")]
mod networkd_status;

use argh::FromArgs;
use std::process;

static KERNEL_HOSTNAME: &str = "/proc/sys/kernel/hostname";
static CURRENT_IP: &str = "/var/lib/netdog/current_ip";
static KERNEL_CMDLINE: &str = "/proc/cmdline";
static PRIMARY_INTERFACE: &str = "/var/lib/netdog/primary_interface";
static PRIMARY_MAC_ADDRESS: &str = "/var/lib/netdog/primary_mac_address";
static DEFAULT_NET_CONFIG_FILE: &str = "/var/lib/bottlerocket/net.toml";
static OVERRIDE_NET_CONFIG_FILE: &str = "/var/lib/netdog/net.toml";
static PRIMARY_SYSCTL_CONF: &str = "/etc/sysctl.d/90-primary_interface.conf";
static SYSCTL_MARKER_FILE: &str = "/run/netdog/primary_sysctls_set";
#[cfg(net_backend = "wicked")]
static LEASE_DIR: &str = "/run/wicked";
static SYS_CLASS_NET: &str = "/sys/class/net";
static SYSTEMD_SYSCTL: &str = "/usr/lib/systemd/systemd-sysctl";
static NETDOG_RESOLV_CONF: &str = "/run/netdog/resolv.conf";

#[cfg(net_backend = "wicked")]
static REAL_RESOLV_CONF: &str = "/etc/resolv.conf";

// This is the path to systemd-resolved's generated simple resolv.conf; see
// https://kubernetes.io/docs/tasks/administer-cluster/dns-debugging-resolution/#known-issues for
// the reasoning behind using this path.
#[cfg(net_backend = "systemd-networkd")]
static REAL_RESOLV_CONF: &str = "/run/systemd/resolve/resolv.conf";
#[cfg(net_backend = "systemd-networkd")]
static NETWORKCTL: &str = "/usr/bin/networkctl";

/// Stores user-supplied arguments.
#[derive(FromArgs, PartialEq, Debug)]
struct Args {
    #[argh(subcommand)]
    subcommand: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    #[cfg(net_backend = "wicked")]
    Install(cli::InstallArgs),
    #[cfg(net_backend = "wicked")]
    Remove(cli::RemoveArgs),
    NodeIp(cli::NodeIpArgs),
    GenerateHostname(cli::GenerateHostnameArgs),
    GenerateNetConfig(cli::GenerateNetConfigArgs),
    SetHostname(cli::SetHostnameArgs),
    WriteResolvConf(cli::WriteResolvConfArgs),
    #[cfg(net_backend = "systemd-networkd")]
    WriteNetworkStatus(cli::WriteNetworkStatusArgs),
}

async fn run() -> cli::Result<()> {
    let args: Args = argh::from_env();
    match args.subcommand {
        #[cfg(net_backend = "wicked")]
        SubCommand::Install(args) => cli::install::run(args)?,
        #[cfg(net_backend = "wicked")]
        SubCommand::Remove(args) => cli::remove::run(args)?,
        SubCommand::NodeIp(_) => cli::node_ip::run()?,
        SubCommand::GenerateHostname(_) => cli::generate_hostname::run().await?,
        SubCommand::GenerateNetConfig(_) => cli::generate_net_config::run()?,
        SubCommand::SetHostname(args) => cli::set_hostname::run(args)?,
        SubCommand::WriteResolvConf(_) => cli::write_resolv_conf::run()?,
        #[cfg(net_backend = "systemd-networkd")]
        SubCommand::WriteNetworkStatus(_) => cli::write_network_status::run()?,
    }
    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
