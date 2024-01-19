/*!
# Bootstrap containers

bootstrap-containers ensures that bootstrap containers are executed as defined in the system settings

It queries the API for their settings, then configures the system by:

* creating a user-data file in the host container's persistent storage area, if a base64-encoded
  user-data setting is set for the host container.  (The decoded contents are available to the
  container at /.bottlerocket/bootstrap-containers/<name>/user-data)
* creating an environment file used by a bootstrap-container-specific instance of a systemd service
* creating a systemd drop-in configuration file used by a bootstrap-container-specific
instance of a systemd service
* ensuring that the bootstrap container's systemd service is enabled/disabled for the next boot

# Examples
Given a bootstrap container called `bear` with the following configuration:

```toml
[settings.bootstrap-containers.bear]
source="<SOURCE>"
mode="once"
user-data="ypXCt82h4bSlwrfKlA=="
```

Where `<SOURCE>`, is the url of an image with the following definition:

```Dockerfile
FROM alpine
ADD bootstrap-script /
RUN chmod +x /bootstrap-script
ENTRYPOINT ["sh", "bootstrap-script"]
```

And `bootstrap-script` as:

```shell
#!/usr/bin/env sh
# We'll read some data to be written out from given user-data.
USER_DATA_DIR=/.bottlerocket/bootstrap-containers/current
# This is the in-container view of where the host's `/var` can be accessed.
HOST_VAR_DIR=/.bottlerocket/rootfs/var
# The directory that'll be created by this bootstrap container
MY_HOST_DIR=$HOST_VAR_DIR/lib/my_directory
# Create it!
mkdir -p "$MY_HOST_DIR"
# Write the user-data to stdout (to the journal) and to our new path:
tee /dev/stdout "$MY_HOST_DIR/bear.txt" < "$USER_DATA_DIR/user-data"
# The bootstrap container can set the permissions which are seen by the host:
chmod -R o+r "$MY_HOST_DIR"
chown -R 1000:1000 "$MY_HOST_DIR"
# Bootstrap containers *must* finish before boot continues.
#
# With this, the boot process will be delayed 120 seconds. You can check the
# status of `preconfigured.target` and `bootstrap-containers@bear` to see
# that this sleep kept the system from starting up the apiserver.
#
# From the admin container:
#
# systemctl status preconfigured.target bootstrap-containers@bear
sleep 120
```

You should see a new directory under `/var/lib` called `my_directory`, a file in that
directory called `bear.txt` and the following command should show `ʕ·͡ᴥ·ʔ` in the bootstrap
containers logs:

```shell
journalctl -u bootstrap-containers@bear.service
```
*/

#[macro_use]
extern crate log;

use base64::Engine;
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::str::FromStr;

use modeled_types::{BootstrapContainerMode, Identifier, Url, ValidBase64};

const ENV_FILE_DIR: &str = "/etc/bootstrap-containers";
const DROPIN_FILE_DIR: &str = "/etc/systemd/system";
const PERSISTENT_STORAGE_DIR: &str = "/local/bootstrap-containers";
const DROP_IN_FILENAME: &str = "overrides.conf";
const DEFAULT_CONFIG_PATH: &str = "/etc/bootstrap-containers/bootstrap-containers.toml";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BootstrapContainer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<Url>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<BootstrapContainerMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_data: Option<ValidBase64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    essential: Option<bool>,
}

/// Stores user-supplied global arguments
#[derive(Debug)]
struct Args {
    log_level: LevelFilter,
    config_path: PathBuf,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            config_path: PathBuf::from_str(DEFAULT_CONFIG_PATH).unwrap(),
        }
    }
}

/// Stores the subcommand to be executed
#[derive(Debug)]
enum Subcommand {
    CreateContainers,
    MarkBootstrap(MarkBootstrapArgs),
}

#[derive(Debug)]
struct MarkBootstrapArgs {
    container_id: String,
    mode: BootstrapContainerMode,
}

/// Print a usage message in the event a bad arg is passed
fn usage() {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {} SUBCOMMAND [ ARGUMENTS... ]

    Subcommands:
        create-containers
        mark-bootstrap

    Global arguments:
        [ --config-path PATH ]
        [ --log-level trace|debug|info|warn|error ]

    Mark bootstrap arguments:
        --container-id CONTAINER-ID
        --mode MODE

    Config path defaults to {}",
        program_name, DEFAULT_CONFIG_PATH,
    );
}

/// Parses user arguments into an Args struct
fn parse_args(args: env::Args) -> Result<(Args, Subcommand)> {
    let mut global_args = Args::default();
    let mut subcommand = None;
    let mut subcommand_args = Vec::new();

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            // Global args
            "--log-level" => {
                let log_level = iter.next().context(error::UsageSnafu {
                    message: "Did not give argument to --log-level",
                })?;
                global_args.log_level = LevelFilter::from_str(&log_level)
                    .context(error::LogLevelSnafu { log_level })?;
            }

            "-c" | "--config-path" => {
                let config_str = iter.next().context(error::UsageSnafu {
                    message: "Did not give argument to --config-path",
                })?;
                global_args.config_path = PathBuf::from(config_str.as_str());
            }

            // Subcommands
            "create-containers" | "mark-bootstrap"
                if subcommand.is_none() && !arg.starts_with('-') =>
            {
                subcommand = Some(arg)
            }

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    match subcommand.as_deref() {
        Some("create-containers") => Ok((global_args, Subcommand::CreateContainers {})),
        Some("mark-bootstrap") => Ok((global_args, parse_mark_bootstrap_args(subcommand_args)?)),
        None => error::UsageSnafu {
            message: "Missing subcommand".to_string(),
        }
        .fail(),
        Some(x) => error::UsageSnafu {
            message: format!("Unknown subcommand '{}'", x),
        }
        .fail(),
    }
}

/// Parses arguments for the 'mark-bootstrap' subcommand
fn parse_mark_bootstrap_args(args: Vec<String>) -> Result<Subcommand> {
    let mut container_id = None;
    let mut mode = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--container-id" => {
                container_id = Some(iter.next().context(error::UsageSnafu {
                    message: "Did not give argument to --container-id",
                })?);
            }

            "--mode" => {
                mode = Some(iter.next().context(error::UsageSnafu {
                    message: "Did not give argument to --mode",
                })?);
            }

            x => {
                return error::UsageSnafu {
                    message: format!("Unexpected argument '{}'", x),
                }
                .fail()
            }
        }
    }

    let container_id = container_id.context(error::UsageSnafu {
        message: "Did not give argument to --container-id".to_string(),
    })?;

    let mode = mode.context(error::UsageSnafu {
        message: "Did not give argument to --mode".to_string(),
    })?;

    Ok(Subcommand::MarkBootstrap(MarkBootstrapArgs {
        container_id,
        // Fail if 'mode' is invalid
        mode: BootstrapContainerMode::try_from(mode).context(error::BootstrapContainerModeSnafu)?,
    }))
}

/// Handles how the bootstrap containers' systemd units are created
fn handle_bootstrap_container<S>(name: S, container_details: &BootstrapContainer) -> Result<()>
where
    S: AsRef<str>,
{
    let name = name.as_ref();

    info!("Handling bootstrap container '{}'", name);

    // Get basic settings, as retrieved from API.
    let source = container_details
        .source
        .as_ref()
        .context(error::MissingFieldSnafu {
            name,
            field: "source",
        })?;

    let mode = container_details.mode.clone().unwrap_or_default();

    let essential = container_details.essential.unwrap_or(false);

    // Create the directory regardless if user data was provided for the container
    let dir = Path::new(PERSISTENT_STORAGE_DIR).join(name);
    fs::create_dir_all(&dir).context(error::MkdirSnafu { dir: &dir })?;

    // If user data was specified, decode it and write it out
    if let Some(user_data) = &container_details.user_data {
        debug!("Decoding user data for container '{}'", name);
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(user_data.as_bytes())
            .context(error::Base64DecodeSnafu { name })?;

        let path = dir.join("user-data");
        debug!("Storing user data in {}", path.display());
        fs::write(path, decoded_bytes).context(error::UserDataWriteSnafu { name })?;
    }

    // Start/stop the container according to the 'mode' setting
    let unit_name = format!("bootstrap-containers@{}.service", name);
    let systemd_unit = SystemdUnit::new(&unit_name);
    let host_containerd_unit = SystemdUnit::new("host-containerd.service");

    // Write the environment file needed for the systemd service to have details
    // this specific bootstrap container
    write_config_files(name, source, &mode, essential)?;

    if mode == "off" {
        // If mode is 'off', disable the container, and clean up any left over tasks
        info!(
            "Bootstrap container mode for '{}' is 'off', disabling unit",
            name
        );
        systemd_unit.disable()?;

        if host_containerd_unit.is_active()? {
            debug!("Cleaning up container '{}'", name);
            command(
                constants::HOST_CTR_BIN,
                [
                    "clean-up",
                    "--container-id",
                    format!("boot.{}", name).as_ref(),
                ],
            )?;
        }
    } else {
        info!("Bootstrap container mode for '{}' is '{}'", name, mode);

        // Clean up any left over tasks, before the container is enabled
        if host_containerd_unit.is_active()? && !systemd_unit.is_enabled()? {
            command(
                constants::HOST_CTR_BIN,
                [
                    "clean-up",
                    "--container-id",
                    format!("boot.{}", name).as_ref(),
                ],
            )?;
        }

        info!("Enabling unit '{}'", unit_name);
        systemd_unit.enable()?;
    }

    Ok(())
}

/// Write out the EnvironmentFile that systemd uses to fill in arguments to host-ctr
fn write_config_files<S1, S2, S3>(name: S1, source: S2, mode: S3, essential: bool) -> Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    S3: AsRef<str>,
{
    let name = name.as_ref();

    // Build environment file
    let env_filename = format!("{}.env", name);
    let env_path = Path::new(ENV_FILE_DIR).join(env_filename);
    let mut output = String::new();

    writeln!(output, "CTR_SOURCE={}", source.as_ref()).context(
        error::WriteConfigurationValueSnafu {
            value: source.as_ref(),
        },
    )?;
    writeln!(output, "CTR_MODE={}", mode.as_ref()).context(
        error::WriteConfigurationValueSnafu {
            value: mode.as_ref(),
        },
    )?;

    debug!("Writing environment file for unit '{}'", name);
    fs::write(&env_path, output).context(error::WriteConfigurationFileSnafu { path: env_path })?;

    // Build unit's drop-in file, used to override the unit's configurations
    let mut output = String::new();
    let drop_in_dir =
        Path::new(DROPIN_FILE_DIR).join(format!("bootstrap-containers@{}.service.d", name));
    let drop_in_path = drop_in_dir.join(DROP_IN_FILENAME);

    // Override the type of dependency the `configured` target has in the unit
    let dependency = if essential { "RequiredBy" } else { "WantedBy" };

    writeln!(output, "[Install]")
        .context(error::WriteConfigurationValueSnafu { value: "[Install]" })?;
    writeln!(output, "{}=configured.target", dependency)
        .context(error::WriteConfigurationValueSnafu { value: dependency })?;
    debug!("Writing drop-in file for {}", name);
    fs::create_dir_all(&drop_in_dir).context(error::MkdirSnafu { dir: &drop_in_dir })?;
    fs::write(&drop_in_path, output)
        .context(error::WriteConfigurationFileSnafu { path: drop_in_path })?;

    Ok(())
}

/// Query the API for the currently defined bootstrap containers
fn get_bootstrap_containers<P>(config_path: P) -> Result<HashMap<Identifier, BootstrapContainer>>
where
    P: AsRef<Path>,
{
    let config_str = fs::read_to_string(config_path.as_ref()).context(error::ReadConfigSnafu {
        config_path: config_path.as_ref(),
    })?;

    toml::from_str(&config_str).context(error::DeserializationSnafu {
        config_path: config_path.as_ref(),
    })
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
        match command(constants::SYSTEMCTL_BIN, ["is-enabled", self.unit]) {
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
        match command(constants::SYSTEMCTL_BIN, ["is-active", self.unit]) {
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

    fn enable(&self) -> Result<()> {
        // Only enable the unit, since it will be started once systemd reaches the `preconfigured`
        // target. There's an implied daemon-reload when the target changes, so defer the reload
        // until then.
        command(
            constants::SYSTEMCTL_BIN,
            ["enable", self.unit, "--no-reload"],
        )
    }

    fn disable(&self) -> Result<()> {
        // Bootstrap containers won't be up by the time the user sends configurations through
        // `apiclient`, so there is no need to add `--now` to stop them, and no need to reload.
        command(
            constants::SYSTEMCTL_BIN,
            ["disable", self.unit, "--no-reload"],
        )
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
        .context(error::ExecutionFailureSnafu { command })?;

    trace!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    trace!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    ensure!(
        output.status.success(),
        error::CommandFailureSnafu { bin_path, output }
    );
    Ok(())
}

/// Handles the `create-containers` subcommand
fn create_containers<P>(config_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut failed = 0usize;
    let bootstrap_containers = get_bootstrap_containers(config_path)?;
    for (name, container_details) in bootstrap_containers.iter() {
        // Continue to handle other bootstrap containers if we fail one
        if let Err(e) = handle_bootstrap_container(name, container_details) {
            failed += 1;
            error!("Failed to handle bootstrap container '{}': {}", &name, e);
        }
    }

    ensure!(
        failed == 0,
        error::ManageContainersFailedSnafu {
            failed,
            tried: bootstrap_containers.len()
        }
    );

    Ok(())
}

/// Handles the `mark-bootstrap` subcommand, which is called by the bootstrap
/// container's systemd unit, which could potentially cause a concurrent invocation
/// in this binary after the API setting finalizes.
fn mark_bootstrap(args: MarkBootstrapArgs) -> Result<()> {
    let container_id: &str = args.container_id.as_ref();
    let mode = args.mode.as_ref();
    info!("Mode for '{}' is '{}'", container_id, mode);

    // When 'mode' is 'once', the container is marked as 'off' once it
    // finishes. This guarantees that the container is only started in
    // the boot where it was created.
    if mode != "always" {
        let formatted = format!("settings.bootstrap-containers.{}.mode=off", container_id);
        info!("Turning off container '{}'", container_id);
        command("apiclient", ["set", formatted.as_str()])?;
    }

    Ok(())
}

fn run() -> Result<()> {
    let (args, subcommand) = parse_args(env::args())?;

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("bootstrap-containers started");

    match subcommand {
        Subcommand::CreateContainers => create_containers(args.config_path),
        Subcommand::MarkBootstrap(mark_bootstrap_args) => mark_bootstrap(mark_bootstrap_args),
    }
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        match e {
            error::Error::Usage { .. } => {
                eprintln!("{}", e);
                usage();
                process::exit(1);
            }
            _ => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

mod error {
    use snafu::Snafu;
    use std::fmt;
    use std::io;
    use std::path::PathBuf;
    use std::process::{Command, Output};

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Failed to read settings from config at {}: {}", config_path.display(), source))]
        ReadConfig {
            config_path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to deserialize settings from config at {}: {}", config_path.display(), source))]
        Deserialization {
            config_path: PathBuf,
            source: toml::de::Error,
        },

        #[snafu(display(
            "Unable to decode base64 in user data of bootstrap container '{}': '{}'",
            name,
            source
        ))]
        Base64Decode {
            name: String,
            source: base64::DecodeError,
        },

        // `try_from` in `BootstrapContainerMode` already returns a useful error message
        #[snafu(display("Failed to parse mode: {}", source))]
        BootstrapContainerMode { source: modeled_types::error::Error },

        #[snafu(display("'{}' failed - stderr: {}",
                        bin_path, String::from_utf8_lossy(&output.stderr)))]
        CommandFailure { bin_path: String, output: Output },

        #[snafu(display("Failed to execute '{:?}': {}", command, source))]
        ExecutionFailure {
            command: Command,
            source: std::io::Error,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Invalid log level '{}'", log_level))]
        LogLevel {
            log_level: String,
            source: log::ParseLevelError,
        },

        #[snafu(display("Failed to manage {} of {} bootstrap containers", failed, tried))]
        ManageContainersFailed { failed: usize, tried: usize },

        #[snafu(display("Bootstrap containers '{}' missing field '{}'", name, field))]
        MissingField { name: String, field: String },

        #[snafu(display("Failed to create directory '{}': '{}'", dir.display(), source))]
        Mkdir {
            dir: PathBuf,
            source: std::io::Error,
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

        #[snafu(display("Unable to serialize data: {}", source))]
        Serialize { source: serde_json::Error },

        #[snafu(display("Failed to change settings via apiclient: {}", source))]
        Set { source: io::Error },

        #[snafu(display("Failed to change settings, apiclient returned an error"))]
        SetClient,

        #[snafu(display("{}", message))]
        Usage { message: String },

        #[snafu(display(
            "Failed to write user-data for bootstrap container '{}': {}",
            name,
            source
        ))]
        UserDataWrite {
            name: String,
            source: std::io::Error,
        },

        #[snafu(display("Failed to write configuration file {}: {}", path.display(), source))]
        WriteConfigurationFile { path: PathBuf, source: io::Error },

        #[snafu(display("Failed write value '{}': {}", value, source))]
        WriteConfigurationValue { value: String, source: fmt::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
