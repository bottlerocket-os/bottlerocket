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

use snafu::{ensure, OptionExt, ResultExt};
use std::env;
use std::ffi::OsStr;
use std::process::{self, Command};

use apiserver::datastore::serialization::to_pairs_with_prefix;
use apiserver::model;

#[macro_use]
extern crate log;

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const API_SETTINGS_URI: &str = "/settings";

const SYSTEMCTL_BIN: &str = "/bin/systemctl";

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::process::{Command, Output};

    use apiserver::datastore::serialization;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

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

        #[snafu(display(
            "Unknown value for '{}': got '{}', expected 'true' or 'false'",
            setting,
            state
        ))]
        UnknownSettingState { setting: String, state: String },

        #[snafu(display("Setting '{}' does not exist in the data store", setting))]
        NonexistentSettting { setting: String },

        #[snafu(display("Failed to execute '{:?}': {}", command, source))]
        ExecutionFailure {
            command: Command,
            source: std::io::Error,
        },

        #[snafu(display("Systemd command failed - stderr: {}",
                        std::str::from_utf8(&output.stderr).unwrap_or_else(|_| "<invalid UTF-8>")))]
        SystemdCommandFailure { output: Output },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

/// SettingState represents the possible states of systemd units for Thar
enum SettingState {
    Enabled,
    Disabled,
}

impl SettingState {
    /// Query the datastore for a given setting and return the corresponding
    /// SettingState.
    fn query<S>(setting: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        match query_setting_value(&setting)?.as_ref() {
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
fn query_setting_value<S>(setting: S) -> Result<String>
where
    S: AsRef<str>,
{
    let setting = setting.as_ref();
    debug!("Querying the API for setting: {}", setting);

    let uri = format!("{}?keys={}", API_SETTINGS_URI, setting);
    let (code, response_body) = apiclient::raw_request(DEFAULT_API_SOCKET, &uri, "GET", None)
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
    let setting_keypair = to_pairs_with_prefix("settings".to_string(), &settings)
        .context(error::SerializeSettings)?;
    debug!("Retrieved setting keypair: {:#?}", &setting_keypair);

    // (Hopefully) get the value from the map using the dotted string supplied
    // to the function
    Ok(setting_keypair
        .get(setting)
        .context(error::NonexistentSettting { setting: setting })?
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
        run(&["start", "--no-block", &self.unit])
    }

    /// Stops the current systemd unit
    fn stop(&self) -> Result<()> {
        run(&["stop", &self.unit])
    }

    /// Enables the current systemd unit
    fn enable(&self) -> Result<()> {
        run(&["enable", &self.unit])
    }

    /// Disables the current systemd unit
    fn disable(&self) -> Result<()> {
        run(&["disable", &self.unit])
    }
}

/// Calls `systemd daemon-reload` which reloads the systemd configuration.
/// According to docs, this *shouldn't* be necessary but experience
/// has show this isn't always the case. It shoudn't hurt to call it
fn systemd_daemon_reload() -> Result<()> {
    run(&["daemon-reload"])
}

/// Wrapper around process::Command that does error handling.
fn run<I, S>(args: I) -> Result<()>
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
    setting: String,
    systemd_unit: String,
    verbosity: usize,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ -s | --setting SETTING ]
            [ -u | --systemd-unit UNIT ]
            [ --verbose --verbose ... ]",
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
    let mut setting = None;
    let mut systemd_unit = None;
    let mut verbosity = 2;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--verbose" => verbosity += 1,

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
        setting: setting.unwrap_or_else(|| usage_msg("-s|--setting is a required argument")),
        systemd_unit: systemd_unit
            .unwrap_or_else(|| usage_msg("-u|--systemd-unit is a required argument")),
        verbosity,
    }
}

fn main() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TODO Fix this later when we decide our logging story
    // Start the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(args.verbosity)
        .color(stderrlog::ColorChoice::Never)
        .init()
        .context(error::Logger)?;

    info!("servicedog started for unit {}", &args.systemd_unit);

    let systemd_unit = SystemdUnit::new(&args.systemd_unit);

    match SettingState::query(args.setting)? {
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
