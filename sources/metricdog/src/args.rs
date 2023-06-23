use argh::FromArgs;
use log::LevelFilter;
use std::path::PathBuf;

fn default_logging() -> LevelFilter {
    LevelFilter::Info
}

/// Command line arguments for the metricdog program.
#[derive(FromArgs)]
pub(crate) struct Arguments {
    /// path to the TOML config file [default: /etc/metricdog]
    #[argh(option, short = 'c', long = "config")]
    pub config: Option<PathBuf>,
    /// logging verbosity [trace|debug|info|warn|error]
    #[argh(option, short = 'l', long = "log-level", default = "default_logging()")]
    pub log_level: LevelFilter,
    /// path to the os-release file [default: /etc/os-release]
    #[argh(option, short = 'o', long = "os-release")]
    pub os_release: Option<PathBuf>,
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub(crate) enum Command {
    SendBootSuccess(SendBootSuccess),
    SendHealthPing(SendHealthPing),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "send-boot-success")]
/// report a successful boot
pub(crate) struct SendBootSuccess {}

#[derive(FromArgs)]
#[argh(subcommand, name = "send-health-ping")]
/// check services and report their health
pub(crate) struct SendHealthPing {}
