/*!
# Background

host-containers ensures that host containers are running as defined in system settings.

It queries the API for their settings, then configures the system by:
* creating a user-data file in the host container's persistent storage area, if a base64-encoded
  user-data setting is set for the host container.  (The decoded contents are available to the
  container at /.bottlerocket/host-containers/NAME/user-data)
* creating an environment file used by a host-container-specific instance of a systemd service
* ensuring the host container's systemd service is enabled/started or disabled/stopped
*/

#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::str::FromStr;

use model::modeled_types::Identifier;

const ENV_FILE_DIR: &str = "/etc/host-containers";
const PERSISTENT_STORAGE_BASE_DIR: &str = "/local/host-containers";

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::fmt;
    use std::io;
    use std::path::PathBuf;
    use std::process::{Command, Output};

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
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
                        bin_path, std::str::from_utf8(&output.stderr).unwrap_or("<invalid UTF-8>")))]
        CommandFailure { bin_path: String, output: Output },

        #[snafu(display("Failed to manage {} of {} host containers", failed, tried))]
        ManageContainersFailed { failed: usize, tried: usize },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display(
            "Unable to base64 decode user-data for container '{}': '{}'",
            name,
            source
        ))]
        Base64Decode {
            name: String,
            source: base64::DecodeError,
        },

        #[snafu(display("Failed to create directory '{}': '{}'", dir.display(), source))]
        Mkdir {
            dir: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to write user-data for host container '{}': {}", name, source))]
        UserDataWrite {
            name: String,
            source: std::io::Error,
        },

        #[snafu(display(
            "Failed to chmod host container '{}' storage directory: {}",
            name,
            source
        ))]
        SetPermissions {
            name: String,
            source: std::io::Error,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

/// Query the API for the currently defined host containers
async fn get_host_containers<P>(socket_path: P) -> Result<HashMap<Identifier, model::HostContainer>>
where
    P: AsRef<Path>,
{
    debug!("Querying the API for settings");

    let method = "GET";
    let uri = constants::API_SETTINGS_URI;
    let (code, response_body) = apiclient::raw_request(&socket_path, uri, method, None)
        .await
        .context(error::APIRequestSnafu { method, uri })?;
    ensure!(
        code.is_success(),
        error::APIResponseSnafu {
            method,
            uri,
            code,
            response_body,
        }
    );

    // Build a Settings struct from the response string
    let settings: model::Settings =
        serde_json::from_str(&response_body).context(error::ResponseJsonSnafu { method, uri })?;

    // If host containers aren't defined, return an empty map
    Ok(settings.host_containers.unwrap_or_default())
}

/// SystemdUnit stores the systemd unit being manipulated
struct SystemdUnit<'a> {
    unit: &'a str,
}

impl<'a> SystemdUnit<'a> {
    fn new(unit: &'a str) -> Self {
        SystemdUnit { unit }
    }

    fn is_active(&self) -> Result<bool> {
        match command(constants::SYSTEMCTL_BIN, ["is-active", self.unit]) {
            Ok(_) => Ok(true),
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

    fn stop(&self) -> Result<()> {
        // This is intentionally blocking to simplify reasoning about the state
        // of the system. The stop command might fail if the unit has just been
        // created and we haven't done a `systemctl daemon-reload` yet.
        let _ = command(constants::SYSTEMCTL_BIN, ["stop", self.unit]);
        Ok(())
    }

    fn enable(&self) -> Result<()> {
        command(
            constants::SYSTEMCTL_BIN,
            ["enable", self.unit, "--no-reload", "--no-block"],
        )?;
        Ok(())
    }

    fn enable_now(&self) -> Result<()> {
        command(
            constants::SYSTEMCTL_BIN,
            ["enable", self.unit, "--now", "--no-block"],
        )?;
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        command(
            constants::SYSTEMCTL_BIN,
            ["disable", self.unit, "--no-reload", "--no-block"],
        )?;
        Ok(())
    }

    fn disable_now(&self) -> Result<()> {
        command(
            constants::SYSTEMCTL_BIN,
            ["disable", self.unit, "--now", "--no-block"],
        )?;
        Ok(())
    }
}

/// Wrapper around process::Command that adds error checking.
fn command<I, S>(bin_path: &str, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(bin_path);
    command.args(args);
    let output = command
        .output()
        .context(error::ExecutionFailureSnafu { command })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    trace!("stdout: {}", stdout);
    trace!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    ensure!(
        output.status.success(),
        error::CommandFailureSnafu { bin_path, output }
    );
    Ok(stdout)
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
        .context(error::EnvFileBuildFailedSnafu { name })?;
    writeln!(output, "CTR_SOURCE={}", source.as_ref())
        .context(error::EnvFileBuildFailedSnafu { name })?;

    writeln!(
        output,
        "\n# Just for reference; service is enabled or disabled by host-containers service"
    )
    .context(error::EnvFileBuildFailedSnafu { name })?;
    writeln!(output, "# CTR_ENABLED={}", enabled)
        .context(error::EnvFileBuildFailedSnafu { name })?;

    fs::write(&path, output).context(error::EnvFileWriteFailedSnafu { path })?;

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
        program_name,
        constants::API_SOCKET,
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
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.into()),
    }
}

fn handle_host_container<S>(name: S, image_details: &model::HostContainer) -> Result<()>
where
    S: AsRef<str>,
{
    // Get basic settings, as retrieved from API.
    let name = name.as_ref();
    let source = image_details
        .source
        .as_ref()
        .context(error::MissingFieldSnafu {
            name,
            field: "source",
        })?;
    let enabled = image_details.enabled.unwrap_or(false);
    let superpowered = image_details.superpowered.unwrap_or(false);

    info!(
        "Host container '{}' is enabled: {}, superpowered: {}, with source: {}",
        name, enabled, superpowered, source
    );

    // Create the directory regardless if user data was provided for the container
    let dir = Path::new(PERSISTENT_STORAGE_BASE_DIR).join(name);
    fs::create_dir_all(&dir).context(error::MkdirSnafu { dir: &dir })?;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
        .context(error::SetPermissionsSnafu { name })?;

    // If user data was specified, unencode it and write it out before we start the container.
    if let Some(user_data) = &image_details.user_data {
        let decoded_bytes =
            base64::decode(user_data.as_bytes()).context(error::Base64DecodeSnafu { name })?;

        let path = dir.join("user-data");
        fs::write(path, decoded_bytes).context(error::UserDataWriteSnafu { name })?;
    }

    // Write the environment file needed for the systemd service to have details about this
    // specific host container
    write_env_file(name, source, enabled, superpowered)?;

    // Now start/stop the container according to the 'enabled' setting
    let unit_name = format!("host-containers@{}.service", name);
    let systemd_unit = SystemdUnit::new(&unit_name);
    let host_containerd_unit = SystemdUnit::new("host-containerd.service");

    // Unconditionally stop the container, and wait for it to complete. Don't worry about
    // the enabled or disabled status for the unit yet - we'll fix that up later.
    debug!("Stopping host container: '{}'", unit_name);
    systemd_unit.stop()?;

    // Let's make sure there's no lingering container tasks that host-ctr might bind to.
    // We want to ensure the host container is running with its most recent configuration.
    if host_containerd_unit.is_active()? {
        debug!("Cleaning up host container: '{}'", unit_name);
        command(
            constants::HOST_CTR_BIN,
            ["clean-up", "--container-id", name],
        )?;
    }

    let systemd_target = command(constants::SYSTEMCTL_BIN, ["get-default"])?;

    // What happens next depends on whether the system has finished booting, and whether the
    // host container is enabled.
    match (systemd_target.trim(), enabled) {
        // If the systemd target is 'multi-user', then we've finished booting. The container
        // should be running if it's enabled, and left stopped if it's disabled.
        ("multi-user.target", true) => {
            debug!("Immediately enabling host container: '{}'", unit_name);
            systemd_unit.enable_now()?
        }
        ("multi-user.target", false) => {
            debug!("Immediately disabling host container: '{}'", unit_name);
            systemd_unit.disable_now()?;
        }

        // If it's any other target, then we haven't finished booting and the system may not
        // be fully configured. The unit state should match the host container status.
        (_, true) => {
            debug!("Enabling host container: '{}'", unit_name);
            systemd_unit.enable()?
        }
        (_, false) => {
            debug!("Disabling host container: '{}'", unit_name);
            systemd_unit.disable()?;
        }
    }

    Ok(())
}

fn is_container_affected(settings: &[&str], container_name: &str) -> bool {
    if settings.is_empty() {
        // it means that Bottlerocket is booting - all containers need to be started
        info!(
            "Handling host container '{}' during full configuration process",
            container_name
        );
        return true;
    }

    let setting_prefix = "settings.host-containers.";
    let container_prefix = format!("{}{}.", setting_prefix, container_name);

    for setting in settings {
        if setting.starts_with(&container_prefix) {
            info!("Handling host container '{}' because it's directly affected by changed setting '{}' (and maybe others)", container_name, setting);
            return true;
        }
        if !setting.starts_with(setting_prefix) {
            // if its some other setting, return true for all host-containers, example: network
            info!("Handling host container '{}' because it's indirectly affected by changed setting '{}' (and maybe others)", container_name, setting);
            return true;
        }
    }
    info!(
        "Not handling host container '{}', no changed settings affect it",
        container_name
    );
    false
}

async fn run() -> Result<()> {
    let args = parse_args(env::args());
    // this env var is passed by thar-be-settings
    let changed_settings_env = env::var("CHANGED_SETTINGS").unwrap_or_else(|_| "".to_string());
    let changed_settings: Vec<&str> = changed_settings_env.split_whitespace().collect();

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("host-containers started");

    let mut failed = 0usize;
    let host_containers = get_host_containers(args.socket_path).await?;
    for (name, image_details) in host_containers.iter() {
        // handle all host containers during startup
        // handle the host container that has settings changed during restart
        if is_container_affected(&changed_settings, name.as_ref()) {
            if let Err(e) = handle_host_container(name, image_details) {
                failed += 1;
                error!("Failed to handle host container '{}': {}", &name, e);
            }
        }
    }

    ensure!(
        failed == 0,
        error::ManageContainersFailedSnafu {
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
