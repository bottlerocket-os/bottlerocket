use super::{error, fetch_net_config, Result};
use crate::interface_id::InterfaceId;
use crate::net_config::Interfaces;
use crate::{KERNEL_CMDLINE, PRIMARY_INTERFACE, PRIMARY_MAC_ADDRESS};
use argh::FromArgs;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::path::Path;

#[cfg(net_backend = "systemd-networkd")]
use crate::networkd::config::{NetworkDConfigFile, NETWORKD_CONFIG_DIR};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-net-config")]
/// Generate wicked network configuration
pub(crate) struct GenerateNetConfigArgs {}

/// Generate configuration for network interfaces.
pub(crate) fn run() -> Result<()> {
    let mut from_cmd_line = false;

    let (maybe_net_config, source) = fetch_net_config()?;
    if source == Path::new(KERNEL_CMDLINE) {
        from_cmd_line = true;
    }

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
    // Remove existing primary interface files since the current primary may have changed or may
    // now be using a MAC address (via an override net.toml/reboot)
    remove_old_primary_interface()?;
    write_primary_interface(&primary_interface)?;

    write_network_config_files(net_config, from_cmd_line)?;
    Ok(())
}

/// Remove primary interface and mac address files
fn remove_old_primary_interface() -> Result<()> {
    for file in &[PRIMARY_INTERFACE, PRIMARY_MAC_ADDRESS] {
        if Path::exists(Path::new(file)) {
            fs::remove_file(file).context(error::FileRemoveSnafu { path: file })?;
        };
    }
    Ok(())
}

/// Persist the primary interface name or MAC to file
fn write_primary_interface(interface_id: &InterfaceId) -> Result<()> {
    match interface_id {
        InterfaceId::Name(name) => fs::write(PRIMARY_INTERFACE, name.to_string()),
        InterfaceId::MacAddress(mac) => fs::write(PRIMARY_MAC_ADDRESS, mac.to_string()),
    }
    .context(error::PrimaryInterfaceWriteSnafu {
        path: PRIMARY_INTERFACE,
    })
}

#[cfg(net_backend = "wicked")]
fn write_network_config_files(net_config: Box<dyn Interfaces>, from_cmd_line: bool) -> Result<()> {
    let mut wicked_interfaces = net_config.as_wicked_interfaces();
    for interface in &mut wicked_interfaces {
        // The kernel command line is too limited to fully specify an interface's configuration;
        // fix some defaults to match legacy behavior.
        // Note: we only allow 1 interface to be listed via kernel command line, so this will only
        // be added to a single interface
        if from_cmd_line {
            interface.accept_ra();
        }

        interface
            .write_config_file()
            .context(error::InterfaceConfigWriteSnafu)?;
    }
    Ok(())
}

#[cfg(net_backend = "systemd-networkd")]
fn write_network_config_files(net_config: Box<dyn Interfaces>, from_cmd_line: bool) -> Result<()> {
    fs::create_dir_all(NETWORKD_CONFIG_DIR).context(error::CreateDirSnafu {
        path: NETWORKD_CONFIG_DIR,
    })?;

    let networkd_config = net_config
        .as_networkd_config()
        .context(error::NetworkDConfigCreateSnafu)?;
    for mut config in networkd_config.create_files() {
        // The kernel command line is too limited to fully specify an interface's configuration;
        // fix some defaults to match legacy behavior.
        // Note: we only allow 1 interface to be listed via kernel command line, so this will only
        // be added to a single interface
        if from_cmd_line {
            if let NetworkDConfigFile::Network(ref mut n) = config {
                n.accept_ra();
                n.disable_dad();
            }
        }

        config
            .write_config_file()
            .context(error::NetworkDConfigWriteSnafu)?;
    }
    Ok(())
}
