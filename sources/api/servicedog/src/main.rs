/*!
# Background

servicedog is a simple systemd unit supervisor.
Its job is to start/stop and enable/disable systemd units based on a setting value it is told to query.

When a setting changes, thar-be-settings does its job and renders configuration files and calls all restart-commands for any affected services.
For settings that represent the desire state of a service, servicedog can be included in the list of restart-commands to manipulate the state of the service based on the value of the setting.
It's provided the name of a setting to query, as well as the systemd unit to act on.
First it queries the value of the setting; the only supported values at this time are "true" and "false".
If the setting is true, servicedog attempts to start and enable the given systemd unit. If the setting is false, it stops and disables the unit.
As its very last step, service dog calls `systemd daemon-reload` to ensure all changes take affect.

*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ensure, OptionExt, ResultExt};
use std::env;
use std::ffi::OsStr;
use std::process::{self, Command};
use std::str::FromStr;

use datastore::serialization::to_pairs_with_prefix;
use datastore::{Key, KeyType};

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const API_SETTINGS_URI: &str = "/settings";

const SYSTEMCTL_BIN: &str = "/bin/systemctl";

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::process::{Command, Output};

    use datastore::{self, serialization};

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

        #[snafu(display("Error serializing settings: {} ", source))]
        SerializeSettings { source: serialization::Error },

        #[snafu(display("Unable to create key '{}': {}", key, source))]
        InvalidKey {
            key: String,
            source: datastore::Error,
        },

        #[snafu(display(
            "Unknown value for '{}': got '{}', expected 'true' or 'false'",
            setting,
            state
        ))]
        UnknownSettingState { setting: String, state: String },

        #[snafu(display("Setting '{}' does not exist in the data store", setting))]
        NonexistentSetting { setting: String },

        #[snafu(display("Failed to execute '{:?}': {}", command, source))]
        ExecutionFailure {
            command: Command,
            source: std::io::Error,
        },

        #[snafu(display("Systemd command failed - stderr: {}",
                        std::str::from_utf8(&output.stderr).unwrap_or_else(|_| "<invalid UTF-8>")))]
        SystemdCommandFailure { output: Output },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

/// SettingState represents the possible states of systemd units for Bottlerocket
enum SettingState {
    Enabled,
    Disabled,
}

impl SettingState {
    /// Query the datastore for a given setting and return the corresponding
    /// SettingState.
    async fn query<S>(setting: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        match query_setting_value(&setting).await?.as_ref() {
            "true" => Ok(SettingState::Enabled),
            "false" => Ok(SettingState::Disabled),
            other => {
                return error::UnknownSettingState {
                    setting: setting.as_ref(),
                    state: other,
                }
                .fail()
            }
        }
    }
}

/// Query the datastore for a given setting and return the setting's value.
// Currently getting the value of a setting requires a few gyrations. The
// API returns a nested structure that can be deserialized to a Settings struct.
// We can then serialize this structure to a map of
// dotted.key.setting -> value. Using this map we can get the setting value.
// FIXME remove this when we have an API client.
async fn query_setting_value<S>(key_str: S) -> Result<String>
where
    S: AsRef<str>,
{
    let key_str = key_str.as_ref();
    let key = Key::new(KeyType::Data, key_str).context(error::InvalidKey { key: key_str })?;
    debug!("Querying the API for setting: {}", key_str);

    let uri = format!("{}?keys={}", API_SETTINGS_URI, key_str);
    let (code, response_body) = apiclient::raw_request(DEFAULT_API_SOCKET, &uri, "GET", None)
        .await
        .context(error::APIRequest {
            method: "GET",
            uri: uri.to_string(),
        })?;
    ensure!(
        code.is_success(),
        error::APIResponse {
            method: "GET",
            uri,
            code,
            response_body,
        }
    );

    // Build a Settings struct from the response string
    let settings: model::Settings =
        serde_json::from_str(&response_body).context(error::ResponseJson { method: "GET", uri })?;

    // Serialize the Settings struct into key/value pairs. This builds the dotted
    // string representation of the setting
    let setting_keypair =
        to_pairs_with_prefix("settings", &settings).context(error::SerializeSettings)?;
    debug!("Retrieved setting keypair: {:#?}", &setting_keypair);

    // (Hopefully) get the value from the map using the dotted string supplied
    // to the function
    Ok(setting_keypair
        .get(&key)
        .context(error::NonexistentSetting { setting: key_str })?
        .to_string())
}

/// SystemdUnit stores the systemd unit being manipulated
struct SystemdUnit {
    unit: String,
}

// FIXME: In the future we should probably look into interfacing directly
// with systemd either via dbus or Rust bindings to systemd
impl SystemdUnit {
    fn new<S>(unit: S) -> Self
    where
        S: AsRef<str>,
    {
        SystemdUnit {
            unit: unit.as_ref().to_string(),
        }
    }

    /// Starts the current systemd unit with the `--no-block` option
    fn start_no_block(&self) -> Result<()> {
        systemctl(&["start", "--no-block", &self.unit])
    }

    /// Stops the current systemd unit
    fn stop(&self) -> Result<()> {
        systemctl(&["stop", &self.unit])
    }

    /// Enables the current systemd unit
    fn enable(&self) -> Result<()> {
        systemctl(&["enable", &self.unit])
    }

    /// Disables the current systemd unit
    fn disable(&self) -> Result<()> {
        systemctl(&["disable", &self.unit])
    }
}

/// Calls `systemd daemon-reload` which reloads the systemd configuration.
/// According to docs, this *shouldn't* be necessary but experience
/// has show this isn't always the case. It shoudn't hurt to call it
fn systemd_daemon_reload() -> Result<()> {
    systemctl(&["daemon-reload"])
}

/// Wrapper around process::Command that does error handling.
fn systemctl<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    // Instantiate command before adding args. Command::new() returns
    // a `Command` whereas `.args()` returns a `&mut Command`. We use
    // `Command` in our error reporting, which does not play nice with
    // mutable references.
    let mut command = Command::new(SYSTEMCTL_BIN);
    command.args(args);
    let output = command
        .output()
        .context(error::ExecutionFailure { command })?;

    ensure!(
        output.status.success(),
        error::SystemdCommandFailure { output }
    );
    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    log_level: LevelFilter,
    setting: String,
    systemd_unit: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ -s | --setting SETTING ]
            [ -u | --systemd-unit UNIT ]
            [ --log-level trace|debug|info|warn|error ]",
        program_name
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
    let mut setting = None;
    let mut systemd_unit = None;

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

            "-s" | "--setting" => {
                setting = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -s | --setting")),
                )
            }

            "-u" | "--systemd-unit" => {
                systemd_unit =
                    Some(iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to -u | --systemd-unit")
                    }))
            }

            _ => usage(),
        }
    }

    Args {
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        setting: setting.unwrap_or_else(|| usage_msg("-s|--setting is a required argument")),
        systemd_unit: systemd_unit
            .unwrap_or_else(|| usage_msg("-u|--systemd-unit is a required argument")),
    }
}

async fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    info!("servicedog started for unit {}", &args.systemd_unit);

    let systemd_unit = SystemdUnit::new(&args.systemd_unit);

    match SettingState::query(args.setting).await? {
        SettingState::Enabled => {
            info!("Starting and enabling unit {}", &args.systemd_unit);
            systemd_daemon_reload()?;
            systemd_unit.enable()?;
            // Don't block on starting the unit
            systemd_unit.start_no_block()?;
        }

        SettingState::Disabled => {
            info!("Stopping and disabling unit {}", &args.systemd_unit);
            systemd_unit.stop()?;
            systemd_unit.disable()?;
            systemd_daemon_reload()?;
        }
    };
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
