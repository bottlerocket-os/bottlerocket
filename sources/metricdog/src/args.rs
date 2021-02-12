use log::LevelFilter;
use std::path::PathBuf;
use structopt::StructOpt;

/// Command line arguments for the metricdog program.
#[derive(StructOpt)]
pub(crate) struct Arguments {
    /// Path to the TOML config file [default: /etc/metricdog]
    #[structopt(short = "c", long = "config")]
    pub(crate) config: Option<PathBuf>,
    /// Logging verbosity [trace|debug|info|warn|error]
    #[structopt(short = "l", long = "log-level", default_value = "info")]
    pub(crate) log_level: LevelFilter,
    /// Path to the os-release file [default: /etc/os-release]
    #[structopt(short = "o", long = "os-release")]
    pub(crate) os_release: Option<PathBuf>,
    #[structopt(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, StructOpt)]
pub(crate) enum Command {
    /// report a successful boot.
    SendBootSuccess,
    /// check services and report their health.
    SendHealthPing,
}
