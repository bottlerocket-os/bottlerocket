use super::{error, Result};
use crate::interface_id::InterfaceId;
use crate::net_config;
use crate::{
    DEFAULT_NET_CONFIG_FILE, KERNEL_CMDLINE, OVERRIDE_NET_CONFIG_FILE, PRIMARY_INTERFACE,
    PRIMARY_MAC_ADDRESS,
};
use argh::FromArgs;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::path::Path;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-net-config")]
/// Generate wicked network configuration
pub(crate) struct GenerateNetConfigArgs {}

/// Generate configuration for network interfaces.
pub(crate) fn run() -> Result<()> {
    let mut from_cmd_line = false;
    let maybe_net_config = if Path::exists(Path::new(OVERRIDE_NET_CONFIG_FILE)) {
        net_config::from_path(OVERRIDE_NET_CONFIG_FILE).context(error::NetConfigParseSnafu {
            path: OVERRIDE_NET_CONFIG_FILE,
        })?
    } else if Path::exists(Path::new(DEFAULT_NET_CONFIG_FILE)) {
        net_config::from_path(DEFAULT_NET_CONFIG_FILE).context(error::NetConfigParseSnafu {
            path: DEFAULT_NET_CONFIG_FILE,
        })?
    } else {
        from_cmd_line = true;
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
    // Remove existing primary interface files since the current primary may have changed or may
    // now be using a MAC address (via an override net.toml/reboot)
    remove_old_primary_interface()?;
    write_primary_interface(&primary_interface)?;

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
