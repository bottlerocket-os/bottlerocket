/*!
# Background

host-containers is a tool that queries the API for the currently enabled host containers and
ensures the relevant systemd service is enabled/started or disabled/stopped for each one depending
on its 'enabled' flag.
*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::str::FromStr;

use model::modeled_types::Identifier;

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const API_SETTINGS_URI: &str = "/settings";
const ENV_FILE_DIR: &str = "/etc/host-containers";

const SYSTEMCTL_BIN: &str = "/bin/systemctl";
const HOST_CTR_BIN: &str = "/bin/host-ctr";

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::fmt;
    use std::io;
    use std::path::PathBuf;
    use std::process::{Command, Output};

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            source: apiclient::Error,
        },

        #[snafu(display("Error {} when sending {} to {}: {}", code, method, uri, response_body))]
        APIResponse {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display(
            "Error deserializing response as JSON from {} to {}: {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: serde_json::Error,
        },

        #[snafu(display("settings.host_containers missing in API response"))]
        MissingSettings {},

        #[snafu(display("Host containers '{}' missing field '{}'", name, field))]
        MissingField { name: String, field: String },

        #[snafu(display("Unable to create host-containers config dir {}: {}", path.display(), source))]
        EnvFileDirCreate { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to build EnvironmentFile for {}: {}", name, source))]
        EnvFileBuildFailed { name: String, source: fmt::Error },

        #[snafu(display("Failed to write EnvironmentFile to {}: {}", path.display(), source))]
        EnvFileWriteFailed { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to execute '{:?}': {}", command, source))]
        ExecutionFailure {
            command: Command,
            source: std::io::Error,
        },

        #[snafu(display("'{}' failed - stderr: {}",
                        bin_path, std::str::from_utf8(&output.stderr).unwrap_or_else(|_| "<invalid UTF-8>")))]
        CommandFailure { bin_path: String, output: Output },

        #[snafu(display("Failed to manage {} of {} host containers", failed, tried))]
        ManageContainersFailed { failed: usize, tried: usize },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: simplelog::TermLogError },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

/// Query the API for the currently defined host containers
async fn get_host_containers<P>(
    socket_path: P,
) -> Result<HashMap<Identifier, model::ContainerImage>>
where
    P: AsRef<Path>,
{
    debug!("Querying the API for settings");

    let method = "GET";
    let uri = API_SETTINGS_URI;
    let (code, response_body) = apiclient::raw_request(&socket_path, uri, method, None)
        .await
        .context(error::APIRequest { method, uri })?;
    ensure!(
        code.is_success(),
        error::APIResponse {
            method,
            uri,
            code,
            response_body,
        }
    );

    // Build a Settings struct from the response string
    let settings: model::Settings =
        serde_json::from_str(&response_body).context(error::ResponseJson { method, uri })?;

    settings.host_containers.context(error::MissingSettings)
}

/// SystemdUnit stores the systemd unit being manipulated
struct SystemdUnit<'a> {
    unit: &'a str,
}

impl<'a> SystemdUnit<'a> {
    fn new(unit: &'a str) -> Self {
        SystemdUnit { unit }
    }

    fn is_enabled(&self) -> Result<bool> {
        match command(SYSTEMCTL_BIN, &["is-enabled", &self.unit]) {
            Ok(()) => Ok(true),
            Err(e) => {
                // If the systemd unit is not enabled, then `systemctl is-enabled` will return a
                // non-zero exit code.
                match e {
                    error::Error::CommandFailure { .. } => Ok(false),
                    _ => {
                        // Otherwise, we return the error
                        Err(e)
                    }
                }
            }
        }
    }

    fn is_active(&self) -> Result<bool> {
        match command(SYSTEMCTL_BIN, &["is-active", &self.unit]) {
            Ok(()) => Ok(true),
            Err(e) => {
                // If the systemd unit is not active(running), then `systemctl is-active` will
                // return a non-zero exit code.
                match e {
                    error::Error::CommandFailure { .. } => Ok(false),
                    _ => {
                        // Otherwise, we return the error
                        Err(e)
                    }
                }
            }
        }
    }

    fn enable_and_start(&self) -> Result<()> {
        // We enable/start units with --no-block to work around cyclic dependency issues at boot
        // time.  It would probably be better to give systemd more of a chance to tell us that
        // something failed to start, if dependencies can be resolved in another way.
        command(
            SYSTEMCTL_BIN,
            &["enable", &self.unit, "--now", "--no-block"],
        )
    }

    fn disable_and_stop(&self) -> Result<()> {
        command(SYSTEMCTL_BIN, &["disable", &self.unit, "--now"])
    }
}

/// Wrapper around process::Command that adds error checking.
fn command<I, S>(bin_path: &str, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(bin_path);
    command.args(args);
    let output = command
        .output()
        .context(error::ExecutionFailure { command })?;

    trace!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    trace!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    ensure!(
        output.status.success(),
        error::CommandFailure { bin_path, output }
    );
    Ok(())
}

/// Write out the EnvironmentFile that systemd uses to fill in arguments to host-ctr
fn write_env_file<S1, S2>(name: S1, source: S2, enabled: bool, superpowered: bool) -> Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let name = name.as_ref();
    let filename = format!("{}.env", name);
    let path = Path::new(ENV_FILE_DIR).join(filename);

    let mut output = String::new();
    writeln!(output, "CTR_SUPERPOWERED={}", superpowered)
        .context(error::EnvFileBuildFailed { name })?;
    writeln!(output, "CTR_SOURCE={}", source.as_ref())
        .context(error::EnvFileBuildFailed { name })?;

    writeln!(
        output,
        "\n# Just for reference; service is enabled or disabled by host-containers service"
    )
    .context(error::EnvFileBuildFailed { name })?;
    writeln!(output, "# CTR_ENABLED={}", enabled).context(error::EnvFileBuildFailed { name })?;

    fs::write(&path, output).context(error::EnvFileWriteFailed { path })?;

    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    log_level: LevelFilter,
    socket_path: PathBuf,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --socket-path PATH ]
            [ --log-level trace|debug|info|warn|error ]

    Socket path defaults to {}",
        program_name, DEFAULT_API_SOCKET,
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut log_level = None;
    let mut socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }

            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path"))
                        .into(),
                )
            }

            _ => usage(),
        }
    }

    Args {
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.into()),
    }
}

fn handle_host_container<S>(name: S, image_details: &model::ContainerImage) -> Result<()>
where
    S: AsRef<str>,
{
    let name = name.as_ref();
    let source = image_details.source.as_ref().context(error::MissingField {
        name,
        field: "source",
    })?;
    let enabled = image_details.enabled.context(error::MissingField {
        name,
        field: "enabled",
    })?;
    let superpowered = image_details.superpowered.context(error::MissingField {
        name,
        field: "superpowered",
    })?;

    info!(
        "Handling host container '{}' which is enabled: {}",
        name, enabled
    );

    // Write the environment file needed for the systemd service to have details about this
    // specific host container
    write_env_file(name, source, enabled, superpowered)?;

    // Now start/stop the container according to the 'enabled' setting
    let unit_name = format!("host-containers@{}.service", name);
    let systemd_unit = SystemdUnit::new(&unit_name);
    let host_containerd_unit = SystemdUnit::new("host-containerd.service");

    if enabled {
        // If this particular host-container was previously disabled. Let's make sure there's no
        // lingering container tasks left over previously that host-ctr might bind to.
        // We want to ensure we're running the host-container with the latest configuration.
        //
        // We only attempt to do this only if host-containerd is active and running
        if host_containerd_unit.is_active()? && !systemd_unit.is_enabled()? {
            command(HOST_CTR_BIN, &["clean-up", "--container-id", name])?;
        }
        systemd_unit.enable_and_start()?;
    } else {
        systemd_unit.disable_and_stop()?;

        // Ensure there's no lingering host-container after it's been disabled.
        //
        // We only attempt to do this only if host-containerd is active and running
        if host_containerd_unit.is_active()? {
            command(HOST_CTR_BIN, &["clean-up", "--container-id", name])?;
        }
    }

    Ok(())
}

async fn run() -> Result<()> {
    let args = parse_args(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    info!("host-containers started");

    let mut failed = 0usize;
    let host_containers = get_host_containers(args.socket_path).await?;
    for (name, image_details) in host_containers.iter() {
        // Continue to handle other host containers if we fail one
        if let Err(e) = handle_host_container(name, image_details) {
            failed += 1;
            error!("Failed to handle host container '{}': {}", &name, e);
        }
    }

    ensure!(
        failed == 0,
        error::ManageContainersFailed {
            failed,
            tried: host_containers.len()
        }
    );

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
