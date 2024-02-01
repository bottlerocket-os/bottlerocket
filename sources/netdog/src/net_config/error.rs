use crate::interface_id;
use snafu::Snafu;
use std::io;
use std::path::PathBuf;

#[cfg(net_backend = "systemd-networkd")]
use crate::networkd;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum Error {
    #[snafu(display("Unable to create '{}', from '{}'", what, given))]
    CreateFromStr { what: String, given: String },

    #[snafu(display(
        "Invalid interface definition, expected 'name:option1,option2', got {}",
        definition
    ))]
    InvalidInterfaceDef { definition: String },

    #[snafu(display("Invalid interface name: {}", source))]
    InvalidInterfaceName { source: interface_id::Error },

    #[snafu(display(
        "Invalid interface option, expected 'dhcp4' or 'dhcp6', got '{}'",
        given
    ))]
    InvalidInterfaceOption { given: String },

    #[snafu(display("Invalid network configuration: {}", reason))]
    InvalidNetConfig { reason: String },

    #[snafu(display("Failed to read kernel command line from '{}': {}", path.display(), source))]
    KernelCmdlineReadFailed { path: PathBuf, source: io::Error },

    #[snafu(display("Multiple default interfaces defined on kernel command line, expected 1",))]
    MultipleDefaultInterfaces,

    #[snafu(display("Failed to read network config from '{}': {}", path.display(), source))]
    NetConfigReadFailed { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to parse network config: {}", source))]
    NetConfigParse { source: toml::de::Error },

    #[cfg(net_backend = "systemd-networkd")]
    #[snafu(display("Unable to create systemd-networkd config: {}", source))]
    NetworkDConfigCreate { source: networkd::Error },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
