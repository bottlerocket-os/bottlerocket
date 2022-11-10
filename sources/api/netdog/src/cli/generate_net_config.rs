use super::{error, Result};
use crate::{
    net_config, DEFAULT_NET_CONFIG_FILE, KERNEL_CMDLINE, OVERRIDE_NET_CONFIG_FILE,
    PRIMARY_INTERFACE,
};
use argh::FromArgs;
use snafu::{OptionExt, ResultExt};
use std::{fs, path::Path};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "generate-net-config")]
/// Generate wicked network configuration
pub(crate) struct GenerateNetConfigArgs {}

/// Generate configuration for network interfaces.
pub(crate) fn run() -> Result<()> {
    let maybe_net_config = if Path::exists(Path::new(OVERRIDE_NET_CONFIG_FILE)) {
        net_config::from_path(OVERRIDE_NET_CONFIG_FILE).context(error::NetConfigParseSnafu {
            path: OVERRIDE_NET_CONFIG_FILE,
        })?
    } else if Path::exists(Path::new(DEFAULT_NET_CONFIG_FILE)) {
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

    let wicked_interfaces = net_config.as_wicked_interfaces();
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
