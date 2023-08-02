/*!
# Introduction

sundog is a small program to handle settings that must be generated at OS runtime.

It requests settings generators from the API and runs them.
The output is collected and sent to a known Bottlerocket API server endpoint.
*/

#[macro_use]
extern crate log;

use shlex::Shlex;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;
use std::process;
use std::str::{self, FromStr};
use std::time::Duration;
use tokio::process::Command as AsyncCommand;

use datastore::serialization::to_pairs_with_prefix;
use datastore::{self, deserialization, Key, KeyType};

// Limit settings generator execution to at most 6 minutes to prevent boot from hanging for too long.
const SETTINGS_GENERATOR_TIMEOUT: Duration = Duration::from_secs(360);

/// Potential errors during Sundog execution
mod error {
    use http::StatusCode;
    use snafu::Snafu;

    use datastore::{self, deserialization, serialization, KeyType};

    /// Potential errors during dynamic settings retrieval
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum SundogError {
        #[snafu(display("Failed to start generator '{}': {}", program, source))]
        CommandFailure {
            program: String,
            source: std::io::Error,
        },

        #[snafu(display(
            "Timed-out waiting for settings generator '{}' to finish: {}",
            generator,
            source
        ))]
        CommandTimeout {
            generator: String,
            source: tokio::time::error::Elapsed,
        },

        #[snafu(display("Generator command is invalid (empty, etc.) - '{}'", command))]
        InvalidCommand { command: String },

        #[snafu(display(
            "Setting generator '{}' failed with exit code {} - stderr: {}",
            program,
            code,
            stderr
        ))]
        FailedSettingGenerator {
            program: String,
            code: String,
            stderr: String,
        },

        #[snafu(display(
            "Setting generator '{}' returned unexpected exit code '{}' - stderr: {}",
            program,
            code,
            stderr
        ))]
        UnexpectedReturnCode {
            program: String,
            code: String,
            stderr: String,
        },

        #[snafu(display("Invalid (non-utf8) output from generator '{}': {}", program, source))]
        GeneratorOutput {
            program: String,
            source: std::str::Utf8Error,
        },

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
            "Error deserializing response as JSON from {} to '{}': {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: serde_json::Error,
        },

        #[snafu(display(
            "Error deserializing command output as JSON from {}: '{}' ;: input: {}",
            generator,
            source,
            input,
        ))]
        CommandJson {
            generator: String,
            input: String,
            source: serde_json::Error,
        },

        #[snafu(display("Failed to get settings with prefixes '{:?}': '{}'", prefixes, source))]
        GetPrefix {
            prefixes: Vec<String>,
            source: apiclient::get::Error,
        },

        #[snafu(display("Error interpreting JSON value as API model: {}", source))]
        InterpretModel { source: serde_json::Error },

        #[snafu(display("Can't deserialize {} from command output '{}'", expected, input,))]
        CommandJsonType {
            expected: &'static str,
            input: serde_json::Value,
        },

        #[snafu(display("Error deserializing HashMap to Settings: {}", source))]
        Deserialize { source: deserialization::Error },

        #[snafu(display("Error serializing Settings to JSON: {}", source))]
        SerializeRequest { source: serde_json::error::Error },

        #[snafu(display("Error serializing Settings: {} ", source))]
        SerializeSettings { source: serialization::Error },

        #[snafu(display("Error serializing command output '{}': {}", value, source))]
        SerializeScalar {
            value: serde_json::Value,
            source: datastore::ScalarError,
        },

        #[snafu(display("Unable to create {:?} key '{}': {}", key_type, key, source))]
        InvalidKey {
            key_type: KeyType,
            key: String,
            #[snafu(source(from(datastore::Error, Box::new)))]
            source: Box<datastore::Error>,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

use error::SundogError;

type Result<T> = std::result::Result<T, SundogError>;

/// Request the setting generators from the API.
async fn get_setting_generators<S>(socket_path: S) -> Result<HashMap<String, String>>
where
    S: AsRef<str>,
{
    let uri = constants::API_SETTINGS_GENERATORS_URI;

    debug!("Requesting setting generators from API");
    let (code, response_body) = apiclient::raw_request(socket_path.as_ref(), uri, "GET", None)
        .await
        .context(error::APIRequestSnafu { method: "GET", uri })?;
    ensure!(
        code.is_success(),
        error::APIResponseSnafu {
            method: "GET",
            uri,
            code,
            response_body,
        }
    );

    let generators: HashMap<String, String> = serde_json::from_str(&response_body)
        .context(error::ResponseJsonSnafu { method: "GET", uri })?;
    trace!("Generators: {:?}", &generators);

    Ok(generators)
}

/// Given a list of settings, query the API for any that are currently set.
async fn get_populated_settings<P>(socket_path: P, to_query: Vec<&str>) -> Result<HashSet<Key>>
where
    P: AsRef<Path>,
{
    debug!("Querying API for populated settings");

    let mut populated_settings = HashSet::new();

    // Retrieve settings by querying the settings by prefix
    let prefixes: Vec<String> = to_query.into_iter().map(|s| s.to_string()).collect();
    let response = apiclient::get::get_prefixes(socket_path, prefixes.to_owned())
        .await
        .context(error::GetPrefixSnafu { prefixes })?;
    debug!("API response model: {}", response.to_string());

    // Build a Settings struct from the response.
    let settings = serde_json::from_value::<model::Model>(response)
        .context(error::InterpretModelSnafu)?
        .settings;

    // Serialize the Settings struct into key/value pairs. This builds the dotted
    // string representation of the setting
    let settings_keypairs =
        to_pairs_with_prefix("settings", &settings).context(error::SerializeSettingsSnafu)?;

    // Put the setting into our hashset of populated keys
    for (k, _) in settings_keypairs {
        populated_settings.insert(k);
    }

    trace!("Found populated settings: {:#?}", &populated_settings);
    Ok(populated_settings)
}

// Builds the proxy environment variables to pass to settings generators
async fn build_proxy_env<P>(socket_path: P) -> Result<HashMap<String, String>>
where
    P: AsRef<Path>,
{
    // Retrieve network proxy related settings.
    let prefixes = vec!["settings.network".to_string()];
    let response = apiclient::get::get_prefixes(&socket_path, prefixes.to_owned())
        .await
        .context(error::GetPrefixSnafu { prefixes })?;

    let mut proxy_envs = HashMap::new();
    if let Some(https_proxy) = response
        .get("settings")
        .and_then(|settings| settings.get("network"))
        .and_then(|network_settings| network_settings.get("https-proxy"))
        .and_then(|s| s.as_str())
    {
        proxy_envs.insert("https_proxy".to_string(), https_proxy.to_string());
        proxy_envs.insert("HTTPS_PROXY".to_string(), https_proxy.to_string());
    } else {
        // If the https-proxy isn't set, we can return early since no-proxy has no effect.
        return Ok(proxy_envs);
    }

    let mut no_proxy = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    // Append user-specified no-proxy setting to the no-proxy list
    if let Some(np) = response
        .get("settings")
        .and_then(|settings| settings.get("network"))
        .and_then(|network_settings| network_settings.get("no-proxy"))
        .and_then(|v| v.as_array())
    {
        no_proxy.append(
            &mut np
                .iter()
                .map(|s| s.as_str().unwrap_or_default().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        );
    }
    // We potentially need to also no-proxy some K8s related domains for K8s variants
    let prefixes = vec!["settings.kubernetes".to_string()];
    let response = apiclient::get::get_prefixes(&socket_path, prefixes.to_owned())
        .await
        .context(error::GetPrefixSnafu { prefixes })?;

    if let Some(k8s_settings) = response
        .get("settings")
        .and_then(|settings| settings.get("kubernetes"))
    {
        if let Some(k8s_apiserver) = k8s_settings.get("api-server").and_then(|s| s.as_str()) {
            no_proxy.push(k8s_apiserver.to_string());
        }
        if let Some(k8s_cluster_domain) =
            k8s_settings.get("cluster-domain").and_then(|s| s.as_str())
        {
            no_proxy.push(k8s_cluster_domain.to_string());
        }
    }
    let no_proxy_value = no_proxy.join(",");
    proxy_envs.insert("no_proxy".to_string(), no_proxy_value.to_owned());
    proxy_envs.insert("NO_PROXY".to_string(), no_proxy_value);

    Ok(proxy_envs)
}

/// Run the setting generators and collect the output
async fn get_dynamic_settings<P>(
    socket_path: P,
    generators: HashMap<String, String>,
) -> Result<model::Settings>
where
    P: AsRef<Path>,
{
    let mut settings = HashMap::new();

    // Build the list of settings to query from the datastore to see if they
    // are currently populated.
    // `generators` keys are setting names in the proper dotted
    // format, i.e. "settings.kubernetes.node-ip"
    let settings_to_query: Vec<&str> = generators.keys().map(|s| s.as_ref()).collect();
    let populated_settings = get_populated_settings(&socket_path, settings_to_query).await?;

    // Get the proxy envs for the settings generators
    let proxy_envs = build_proxy_env(socket_path).await?;

    // For each generator, run it and capture the output
    for (setting_str, generator) in generators {
        let setting = Key::new(KeyType::Data, &setting_str).context(error::InvalidKeySnafu {
            key_type: KeyType::Data,
            key: &setting_str,
        })?;

        // Don't clobber settings that are already populated
        // We're checking for prefix matches here because some generators are for settings that map
        // to a top level struct that might have subfields. For example, a settings generator for
        // `settings.boot` encompasses both `settings.boot.kernel` and `settings.boot.init`.
        // TODO: We can optimize by using a prefix trie here. But there are no satisfactory trie
        //  implementations I can find on crates.io. We can roll our own at some point if this
        //  becomes a bottleneck
        if populated_settings
            .iter()
            .any(|k| k.starts_with_segments(setting.segments()))
        {
            debug!("Setting '{}' is already populated, skipping", setting);
            continue;
        }

        debug!("Running generator: '{}'", &generator);

        // Split on space, assume the first item is the command
        // and the rest are args.
        let mut command_strings = Shlex::new(&generator);

        let command = command_strings.next().context(error::InvalidCommandSnafu {
            command: generator.as_str(),
        })?;
        let command = command.as_str();

        let result = tokio::time::timeout(
            SETTINGS_GENERATOR_TIMEOUT,
            AsyncCommand::new(command)
                .envs(&proxy_envs)
                .args(command_strings)
                .output(),
        )
        .await
        .context(error::CommandTimeoutSnafu {
            generator: generator.as_str(),
        })?
        .context(error::CommandFailureSnafu {
            program: generator.as_str(),
        })?;

        // Match on the generator's exit code. This code lays the foundation
        // for handling alternative exit codes from generators.
        match result.status.code() {
            Some(0) => {
                if !result.stderr.is_empty() {
                    let cmd_stderr = String::from_utf8_lossy(&result.stderr);
                    for line in cmd_stderr.lines() {
                        info!("Setting generator command '{}' stderr: {}", command, line);
                    }
                }
            }
            Some(1) => {
                return error::FailedSettingGeneratorSnafu {
                    program: generator.as_str(),
                    code: 1.to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
                .fail()
            }
            Some(2) => {
                warn!(
                    "'{}' returned 2, not setting '{}', continuing with other generators",
                    command, generator
                );
                continue;
            }
            Some(x) => {
                return error::UnexpectedReturnCodeSnafu {
                    program: generator.as_str(),
                    code: x.to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
                .fail()
            }
            // A process will return None if terminated by a signal, regard this as
            // a failure since we could have incomplete data
            None => {
                return error::FailedSettingGeneratorSnafu {
                    program: generator.as_str(),
                    code: "signal",
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
                .fail()
            }
        }

        // Sundog programs are expected to output JSON, which allows them to represent types other
        // than strings, which in turn allows our API model to use types more accurate than strings
        // for generated settings.
        //
        // First, we pull the raw string from the process output.
        let output_raw = str::from_utf8(&result.stdout)
            .context(error::GeneratorOutputSnafu {
                program: generator.as_str(),
            })?
            .trim()
            .to_string();
        trace!("Generator '{}' output: {}", &generator, &output_raw);

        // Next, we deserialize the text into a Value that can represent any JSON type.
        let output_value: serde_json::Value =
            serde_json::from_str(&output_raw).context(error::CommandJsonSnafu {
                generator: &generator,
                input: &output_raw,
            })?;

        // Finally, we re-serialize the command output; we intend to call the datastore-level
        // construct `from_map` on it, which expects serialized values.
        //
        // We have to go through the round-trip of serialization because the data store
        // serialization format may not be the same as the format we choose for sundog.
        let serialized_output =
            datastore::serialize_scalar(&output_value).context(error::SerializeScalarSnafu {
                value: output_value,
            })?;
        trace!("Serialized output: {}", &serialized_output);

        settings.insert(setting, serialized_output);
    }

    // The API takes a properly nested Settings struct, so deserialize our map to a Settings
    // and ensure it is correct
    let settings_struct: model::Settings =
        deserialization::from_map(&settings).context(error::DeserializeSnafu)?;

    Ok(settings_struct)
}

/// Send the settings to the datastore through the API
async fn set_settings<S>(socket_path: S, settings: model::Settings) -> Result<()>
where
    S: AsRef<str>,
{
    // Serialize our Settings struct to the JSON wire format
    let request_body = serde_json::to_string(&settings).context(error::SerializeRequestSnafu)?;

    let uri = &format!(
        "{}?tx={}",
        constants::API_SETTINGS_URI,
        constants::LAUNCH_TRANSACTION
    );
    let method = "PATCH";
    trace!("Settings to {} to {}: {}", method, uri, &request_body);
    let (code, response_body) =
        apiclient::raw_request(socket_path.as_ref(), uri, method, Some(request_body))
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

    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    log_level: LevelFilter,
    socket_path: String,
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
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path")),
                )
            }

            _ => usage(),
        }
    }

    Args {
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
    }
}

async fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("Sundog started");

    info!("Retrieving setting generators");
    let generators = get_setting_generators(&args.socket_path).await?;
    if generators.is_empty() {
        info!("No settings to generate, exiting");
        process::exit(0)
    }

    info!("Retrieving settings values");
    let settings = get_dynamic_settings(&args.socket_path, generators).await?;

    info!("Sending settings values to the API");
    set_settings(&args.socket_path, settings).await?;

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
