/*!
  shimpei is an OCI compatible shim wrapper around `oci-add-hooks`. Its sole purpose is
  to call `oci-add-hooks` with the additional `--hook-config-path` and `--runtime-path`
  parameters that can't be provided by containerd.
*/

#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{OptionExt, ResultExt};
use std::env;
use std::ffi::CString;
use std::path::Path;
use std::process;

/// Path to runc binary
const RUNC_BIN_PATH: &str = "/usr/bin/runc";

/// Path to hooks definitions
const HOOKS_CONFIG_BASE_PATH: &str = "/etc/shimpei";

/// Path to oci-add-hooks
const OCI_ADD_HOOKS: &str = "/usr/bin/oci-add-hooks";

fn run() -> Result<()> {
    setup_logger()?;
    let mut args = env::args();
    let prefix = args
        .next()
        .context(error::MissingArgSnafu { what: "name" })?;
    let hook_path = Path::new(HOOKS_CONFIG_BASE_PATH).join(format!("{}-hooks.json", prefix));

    let mut oci_add_hooks_args: Vec<CString> = vec![
        CString::new("oci-add-hooks").expect("Couldn't create CString from 'oci-add-hooks'"),
        CString::new("--hook-config-path")
            .expect("Couldn't create CString from '--hook-config-path'"),
        CString::new(hook_path.display().to_string()).context(error::InvalidStringSnafu {
            input: hook_path.display().to_string(),
        })?,
        CString::new("--runtime-path").expect("Couldn't create CString from '--runtime-path'"),
        CString::new(RUNC_BIN_PATH).context(error::InvalidStringSnafu {
            input: RUNC_BIN_PATH.to_string(),
        })?,
    ];
    for arg in args {
        oci_add_hooks_args
            .push(CString::new(arg.as_bytes()).context(error::InvalidStringSnafu { input: arg })?);
    }

    // Use the `execv` syscall instead of `std::process::Command`, since
    // it will call `posix_spawn` under the hood, which forks instead of
    // replacing the current process

    nix::unistd::execv(
        &CString::new(OCI_ADD_HOOKS).context(error::InvalidStringSnafu {
            input: OCI_ADD_HOOKS.to_string(),
        })?,
        &oci_add_hooks_args,
    )
    .context(error::ExecvSnafu {
        program: OCI_ADD_HOOKS.to_string(),
    })?;

    Ok(())
}

fn setup_logger() -> Result<()> {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).context(error::LoggerSnafu)
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        process::exit(1);
    }
}

/// ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡
mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Failed to setup logger: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Invalid log level '{}'", log_level))]
        LogLevel {
            log_level: String,
            source: log::ParseLevelError,
        },

        #[snafu(display("Couldn't create CString from '{}': {}", input, source))]
        InvalidString {
            input: String,
            source: std::ffi::NulError,
        },

        #[snafu(display("Failed to exec '{}' : {}", program, source))]
        Execv { program: String, source: nix::Error },

        #[snafu(display("Missing argument '{}'", what))]
        MissingArg { what: String },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
