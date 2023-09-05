//! The apiclient binary provides some high-level, synchronous methods of interacting with the
//! API, for example an `update` subcommand that wraps the individual API calls needed to update
//! the host.  There's also a low-level `raw` subcommand for direct interaction.

// This file contains the arg parsing and high-level behavior.  (Massaging input data, making
// library calls based on the given flags, etc.)  The library modules contain the code for talking
// to the API, which is intended to be reusable by other crates.

use apiclient::{apply, exec, get, reboot, report, set, update};
use datastore::{serialize_scalar, Key, KeyType};
use log::{info, log_enabled, trace, warn};
use simplelog::{
    ColorChoice, ConfigBuilder as LogConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};
use snafu::ResultExt;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::process;
use std::str::FromStr;
use unindent::unindent;

const DEFAULT_METHOD: &str = "GET";

/// Stores user-supplied global arguments.
#[derive(Debug)]
struct Args {
    log_level: LevelFilter,
    socket_path: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            socket_path: constants::API_SOCKET.to_string(),
        }
    }
}

/// Stores the usage mode specified by the user as a subcommand.
#[derive(Debug)]
enum Subcommand {
    Apply(ApplyArgs),
    Exec(ExecArgs),
    Get(GetArgs),
    Raw(RawArgs),
    Reboot(RebootArgs),
    Set(SetArgs),
    Update(UpdateSubcommand),
    Report(ReportSubcommand),
}

/// Stores user-supplied arguments for the 'apply' subcommand.
#[derive(Debug)]
struct ApplyArgs {
    input_sources: Vec<String>,
}

/// Stores user-supplied arguments for the 'exec' subcommand.
#[derive(Debug)]
struct ExecArgs {
    command: Vec<OsString>,
    target: String,
    tty: Option<bool>,
}

/// Stores user-supplied arguments for the 'get' subcommand.
#[derive(Debug)]
enum GetArgs {
    Prefixes(Vec<String>),
    Uri(String),
}

/// Stores user-supplied arguments for the 'raw' subcommand.
#[derive(Debug)]
struct RawArgs {
    method: String,
    uri: String,
    data: Option<String>,
}

/// Stores user-supplied arguments for the 'reboot' subcommand.
#[derive(Debug)]
struct RebootArgs {}

/// Stores user-supplied arguments for the 'set' subcommand.
#[derive(Debug)]
enum SetArgs {
    Simple(HashMap<Key, String>),
    Json(serde_json::Value),
}

/// Stores the 'update' subcommand specified by the user.
#[derive(Debug)]
enum UpdateSubcommand {
    Check(UpdateCheckArgs),
    Apply(UpdateApplyArgs),
    Cancel(UpdateCancelArgs),
}

/// The available 'report' subcommands.
#[derive(Debug)]
enum ReportSubcommand {
    Cis(CisReportArgs),
    CisK8s(CisReportArgs),
}

/// Stores common user-supplied arguments for the cis report subcommand.
#[derive(Debug)]
struct CisReportArgs {
    level: Option<i32>,
    format: Option<String>,
}

/// Stores user-supplied arguments for the 'update check' subcommand.
#[derive(Debug)]
struct UpdateCheckArgs {}

/// Stores user-supplied arguments for the 'update apply' subcommand.
#[derive(Debug)]
struct UpdateApplyArgs {
    check: bool,
    reboot: bool,
}

/// Stores user-supplied arguments for the 'update cancel' subcommand.
#[derive(Debug)]
struct UpdateCancelArgs {}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let msg = &format!(
        r#"Usage: apiclient [SUBCOMMAND] [OPTION]...

        Global options:
            -s, --socket-path PATH     Override the server socket path.  Default: {socket}
            --log-level                Desired amount of output; trace|debug|info|warn|error
            -v, --verbose              Sets log level to 'debug'.  This prints extra info,
                                       like HTTP status code to stderr in 'raw' mode.

        Subcommands:
            raw                        Makes an HTTP request and prints the response on stdout.
                                       'raw' is the default subcommand and may be omitted.
            apply                      Applies settings from TOML/JSON files at given URIs,
                                       or from stdin.
            get                        Retrieve and print settings.
            set                        Changes settings and applies them to the system.
            update check               Prints information about available updates.
            update apply               Applies available updates.
            update cancel              Deactivates an applied update.
            reboot                     Reboots the host.
            exec                       Execute a command in a host container.
            report cis                 Retrieve a Bottlerocket CIS benchmark compliance report.
            report cis-k8s             Retrieve a Kubernetes CIS benchmark compliance report.

        raw options:
            -u, --uri URI              Required; URI to request from the server, e.g. /tx
            -m, -X, --method METHOD    HTTP method to use in request.  Default: {method}
            -d, --data DATA            Data to include in the request body.  Default: empty

        apply options:
            [ URI ...]                 The list of URIs to TOML or JSON settings files that you
                                       want to apply to the system.  If no URI is specified, or
                                       if "-" is given, reads from stdin.

        reboot options:
            None.

        get options:
            [ PREFIX [PREFIX ...] ]    The settings you want to get.  Full settings names work fine,
                                       or you can specify prefixes to fetch all settings under them.
            [ /desired-uri ]           The API URI to fetch.  Cannot be specified with prefixes.

                                       If neither prefixes nor URI are specified, get will show
                                       settings and OS info.

        set options:
            KEY=VALUE [KEY=VALUE ...]  The settings you want to set.  For example:
                                          settings.motd="hi there" settings.ecs.cluster=example
                                       The "settings." prefix is optional.
                                       Settings with dots in the name require nested quotes:
                                          'kubernetes.node-labels."my.label"=hello'
            -j, --json JSON            Alternatively, you can specify settings in JSON format,
                                       which can simplify setting multiple values, and is necessary
                                       for some numeric settings.  For example:
                                          -j '{{"kernel": {{"sysctl": {{"vm.max_map_count": "262144"}}}}}}'

        update check options:
            None.

        update apply options:
            -c, --check                Automatically `update check` and apply whatever is found.
            -r, --reboot               Automatically reboot if an update was found and applied.

        update cancel options:
            None.

        exec options:
            -t, --tty                  Force the server to run the program in a pseudoterminal.
            -T, --no-tty               Force the server not to run the program in a pseudoterminal.

            TARGET                     Required; the name of the container in which to run the command.
            COMMAND                    Required; the command to run.
            [ ARG ...]                 Any desired arguments to the command.

        report cis options:
            -f, --format               Format of the CIS report (text or json). Default format is text.
            -l, --level                CIS compliance level to report on (1 or 2). Default is 1.

        report cis-k8s options:
            -f, --format               Format of the CIS report (text or json). Default format is text.
            -l, --level                CIS compliance level to report on (1 or 2). Default is 1."#,
        socket = constants::API_SOCKET,
        method = DEFAULT_METHOD,
    );
    eprintln!("{}", unindent(msg));
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
// Arg parsing

/// Parses user arguments into an Args structure.
fn parse_args(args: env::Args) -> (Args, Subcommand) {
    let mut global_args = Args::default();
    let mut subcommand = None;
    let mut subcommand_args = Vec::new();

    let mut iter = args.into_iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-h" | "--help" => usage(),

            // Global args
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                global_args.log_level =
                    LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                        usage_msg(format!("Invalid log level '{}'", log_level_str))
                    });
            }

            "-v" | "--verbose" => global_args.log_level = LevelFilter::Debug,

            "-s" | "--socket-path" => {
                global_args.socket_path = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -s | --socket-path"))
            }

            // Subcommands
            "raw" | "apply" | "exec" | "get" | "reboot" | "report" | "set" | "update"
                if subcommand.is_none() && !arg.starts_with('-') =>
            {
                subcommand = Some(arg)
            }

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    match subcommand.as_deref() {
        // Default subcommand is 'raw'
        None | Some("raw") => (global_args, parse_raw_args(subcommand_args)),
        Some("apply") => (global_args, parse_apply_args(subcommand_args)),
        Some("exec") => (global_args, parse_exec_args(subcommand_args)),
        Some("get") => (global_args, parse_get_args(subcommand_args)),
        Some("reboot") => (global_args, parse_reboot_args(subcommand_args)),
        Some("report") => (global_args, parse_report_args(subcommand_args)),
        Some("set") => (global_args, parse_set_args(subcommand_args)),
        Some("update") => (global_args, parse_update_args(subcommand_args)),
        _ => usage_msg("Missing or unknown subcommand"),
    }
}

/// Parses arguments for the 'raw' subcommand, which is also the default if no subcommand is
/// provided.
fn parse_raw_args(args: Vec<String>) -> Subcommand {
    let mut method = None;
    let mut uri = None;
    let mut data = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-X" | "-m" | "--method" => {
                method = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -m | --method")),
                )
            }

            "-u" | "--uri" => {
                uri = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -u | --uri")),
                )
            }

            "-d" | "--data" => {
                data = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -d | --data")),
                )
            }

            x => usage_msg(format!("Unknown argument '{}'", x)),
        }
    }

    Subcommand::Raw(RawArgs {
        method: method.unwrap_or_else(|| DEFAULT_METHOD.to_string()),
        uri: uri.unwrap_or_else(|| usage_msg("Missing required argument '--uri'")),
        data,
    })
}

/// Parses arguments for the 'apply' subcommand.
fn parse_apply_args(args: Vec<String>) -> Subcommand {
    let mut input_sources = Vec::new();

    for arg in args.into_iter() {
        match arg {
            // Allow "-" for stdin, but we have no other parameters.
            x if x.starts_with('-') && x != "-" => {
                usage_msg("apiclient apply takes no parameters, just a list of URIs.")
            }

            x => input_sources.push(x),
        }
    }

    if input_sources.is_empty() {
        // Read from stdin if no URIs were given.
        input_sources.push("-".to_string());
    }

    Subcommand::Apply(ApplyArgs { input_sources })
}

/// Parses arguments for the 'exec' subcommand.
fn parse_exec_args(args: Vec<String>) -> Subcommand {
    let mut command = vec![];
    let mut target = None;
    let mut tty = None;

    for arg in args.into_iter() {
        match arg.as_ref() {
            // Check for our own arguments, but stop once we start to see the user's command; we
            // don't want to intercept its own arguments.
            "-t" | "--tty" if command.is_empty() => {
                tty = Some(true);
            }
            "-T" | "--no-tty" if command.is_empty() => {
                tty = Some(false);
            }
            x if x.starts_with('-') && command.is_empty() => {
                usage_msg(format!("Unknown argument '{}'", x))
            }

            // Target is the first arg we see.
            _ if target.is_none() => target = Some(arg),
            // Anything remaining goes to the command.
            _ => command.push(arg.into()),
        }
    }

    // (check target here because it's clearer to error about it before an error about a missing command)
    let target = target.unwrap_or_else(|| usage_msg("Missing required argument 'target'"));
    if command.is_empty() {
        usage_msg("Must specify a command for 'exec' to run.");
    }

    Subcommand::Exec(ExecArgs {
        command,
        target,
        tty,
    })
}

/// Parses arguments for the 'get' subcommand.
fn parse_get_args(args: Vec<String>) -> Subcommand {
    let mut prefixes = vec![];
    let mut uri = None;

    for arg in args.into_iter() {
        match &arg {
            x if x.starts_with('-') => usage_msg(format!("Unknown argument '{}'", x)),

            x if x.starts_with('/') => {
                if let Some(_existing_val) = uri.replace(arg) {
                    usage_msg("You can only specify one URI.");
                }
            }

            // All other arguments are settings prefixes to fetch.
            _ => prefixes.push(arg),
        }
    }

    if let Some(uri) = uri {
        if !prefixes.is_empty() {
            usage_msg("You can specify prefixes or a URI, but not both.");
        }
        Subcommand::Get(GetArgs::Uri(uri))
    } else if !prefixes.is_empty() {
        if uri.is_some() {
            usage_msg("You can specify prefixes or a URI, but not both.");
        }
        Subcommand::Get(GetArgs::Prefixes(prefixes))
    } else {
        // A reasonable default is showing OS info and settings.
        Subcommand::Get(GetArgs::Prefixes(vec![
            "os.".to_string(),
            "settings.".to_string(),
        ]))
    }
}

/// Parses arguments for the 'reboot' subcommand.
fn parse_reboot_args(args: Vec<String>) -> Subcommand {
    if !args.is_empty() {
        usage_msg(format!("Unknown arguments: {}", args.join(", ")));
    }
    Subcommand::Reboot(RebootArgs {})
}

/// Parses arguments for the 'set' subcommand.
// Note: the API doesn't allow setting non-settings keys, e.g. services, configuration-files, and
// metadata.  If we allow it in the future, we should revisit this 'set' parsing code and decide
// what formats to accept.  This code currently makes it as convenient as possible to set settings,
// by adding/removing a "settings" prefix as necessary.
fn parse_set_args(args: Vec<String>) -> Subcommand {
    let mut simple = HashMap::new();
    let mut json = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-j" | "--json" if json.is_some() => {
                usage_msg(
                    "Can't specify the --json argument multiple times.  You can set as many \
                     settings as needed within the JSON object.",
                );
            }
            "-j" | "--json" if json.is_none() => {
                let raw_json = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -j | --json"));

                let input_val: serde_json::Value =
                    serde_json::from_str(&raw_json).unwrap_or_else(|e| {
                        usage_msg(format!("Couldn't parse given JSON input: {}", e))
                    });

                let mut input_map = match input_val {
                    serde_json::Value::Object(map) => map,
                    _ => usage_msg("JSON input must be an object (map)"),
                };

                // To be nice, if the user specified a "settings" layer around their data, we
                // remove it.  (This should only happen if there's a single key, since we only
                // allow setting settings; fail otherwise.  If we allow setting other types in the
                // future, we'll have to do more map manipulation here to save the other values.)
                if let Some(val) = input_map.remove("settings") {
                    match val {
                        serde_json::Value::Object(map) => input_map.extend(map),
                        _ => usage_msg("JSON 'settings' value must be an object (map)"),
                    };
                }

                json = Some(input_map.into());
            }

            x if x.contains('=') => {
                let (raw_key, value) = x.split_once('=').unwrap();

                let mut key = Key::new(KeyType::Data, raw_key).unwrap_or_else(|_| {
                    usage_msg(format!("Given key '{}' is not a valid format", raw_key))
                });

                // Add "settings" prefix if the user didn't give a known prefix, to ease usage
                let key_prefix = &key.segments()[0];
                if key_prefix != "settings" {
                    let mut segments = key.segments().clone();
                    segments.insert(0, "settings".to_string());
                    key = Key::from_segments(KeyType::Data, &segments)
                        .expect("Adding prefix to key resulted in invalid key?!");
                }

                simple.insert(key, value.to_string());
            }

            x => usage_msg(format!("Unknown argument '{}'", x)),
        }
    }

    if json.is_some() && !simple.is_empty() {
        usage_msg("Cannot specify key=value pairs and --json settings with 'set'");
    } else if let Some(json) = json {
        Subcommand::Set(SetArgs::Json(json))
    } else if !simple.is_empty() {
        Subcommand::Set(SetArgs::Simple(simple))
    } else {
        usage_msg("Must specify key=value settings or --json settings with 'set'");
    }
}

/// Parses the desired subcommand of 'update'.
fn parse_update_args(args: Vec<String>) -> Subcommand {
    let mut subcommand = None;
    let mut subcommand_args = Vec::new();

    for arg in args.into_iter() {
        match arg.as_ref() {
            // Subcommands
            "check" | "apply" | "cancel" if subcommand.is_none() && !arg.starts_with('-') => {
                subcommand = Some(arg)
            }

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    let update = match subcommand.as_deref() {
        Some("check") => parse_update_check_args(subcommand_args),
        Some("apply") => parse_update_apply_args(subcommand_args),
        Some("cancel") => parse_update_cancel_args(subcommand_args),
        _ => usage_msg("Missing or unknown subcommand for 'update'"),
    };

    Subcommand::Update(update)
}

/// Parses arguments for the 'update check' subcommand.
fn parse_update_check_args(args: Vec<String>) -> UpdateSubcommand {
    if !args.is_empty() {
        usage_msg(format!("Unknown arguments: {}", args.join(", ")));
    }
    UpdateSubcommand::Check(UpdateCheckArgs {})
}

/// Parses arguments for the 'update apply' subcommand.
fn parse_update_apply_args(args: Vec<String>) -> UpdateSubcommand {
    let mut check = false;
    let mut reboot = false;

    for arg in args.into_iter() {
        match arg.as_ref() {
            "-c" | "--check" => check = true,
            "-r" | "--reboot" => reboot = true,

            x => usage_msg(format!("Unknown argument '{}'", x)),
        }
    }

    UpdateSubcommand::Apply(UpdateApplyArgs { check, reboot })
}

/// Parses arguments for the 'update cancel' subcommand.
fn parse_update_cancel_args(args: Vec<String>) -> UpdateSubcommand {
    if !args.is_empty() {
        usage_msg(format!("Unknown arguments: {}", args.join(", ")));
    }
    UpdateSubcommand::Cancel(UpdateCancelArgs {})
}

/// Parses the desired subcommand of 'report'.
fn parse_report_args(args: Vec<String>) -> Subcommand {
    let mut subcommand = None;
    let mut subcommand_args = Vec::new();

    for arg in args.into_iter() {
        match arg.as_ref() {
            // Subcommands
            "cis" if subcommand.is_none() && !arg.starts_with('-') => subcommand = Some(arg),
            "cis-k8s" if subcommand.is_none() && !arg.starts_with('-') => subcommand = Some(arg),

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    let report_type = match subcommand.as_deref() {
        Some("cis") => parse_report_cis_args(subcommand_args),
        Some("cis-k8s") => parse_report_cis_k8s_args(subcommand_args),
        _ => usage_msg("Missing or unknown subcommand for 'report'"),
    };

    Subcommand::Report(report_type)
}

/// Parses arguments for the 'report' cis subcommand.
fn parse_report_cis_args(args: Vec<String>) -> ReportSubcommand {
    ReportSubcommand::Cis(parse_cis_arguments(args))
}

/// Parses arguments for the 'report' cis-k8s subcommand.
fn parse_report_cis_k8s_args(args: Vec<String>) -> ReportSubcommand {
    ReportSubcommand::CisK8s(parse_cis_arguments(args))
}

fn parse_cis_arguments(args: Vec<String>) -> CisReportArgs {
    let mut level: Option<i32> = None;
    let mut format = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-l" | "--level" => {
                let level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -l | --level"));
                let level_int = level_str
                    .parse::<i32>()
                    .unwrap_or_else(|_| usage_msg("Invalid argument to -l | --level"));
                level = Some(level_int);
            }

            "-f" | "--format" => {
                format = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -f | --format")),
                )
            }

            x => usage_msg(format!("Unknown argument '{}'", x)),
        }
    }

    CisReportArgs { level, format }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
// Helpers

/// Requests an update status check through the API, printing the updated status, in a pretty
/// format if possible.
async fn check(args: &Args) -> Result<String> {
    let output = update::check(&args.socket_path)
        .await
        .context(error::UpdateCheckSnafu)?;

    match serde_json::from_str::<serde_json::Value>(&output) {
        Ok(value) => println!("{:#}", value),
        Err(e) => {
            warn!("Unable to deserialize response (invalid JSON?): {}", e);
            println!("{}", output);
        }
    }

    Ok(output)
}

/// We want the key=val form of 'set' to be as simple as possible; we don't want users to have to
/// annotate or structure their input too much just to tell us the data type, but unfortunately
/// knowledge of the data type is required to deserialize with the current datastore ser/de code.
///
/// To simplify usage, we use some heuristics to determine the type of each input.  We try to parse
/// each value as a number and boolean, and if those fail, we assume a string.  (API communication
/// is in JSON form, limiting the set of types; the API doesn't allow arrays or null, and "objects"
/// (maps) are represented natively through our nested tree-like settings structure.)
///
/// If this goes wrong -- for example the user wants a string "42" -- we'll get a deserialization
/// error, and can print a clear error and request the user use JSON input form to handle
/// situations with more complex types.
///
/// If you have an idea for how to improve deserialization so we don't have to do this, please say!
fn massage_set_input(input_map: HashMap<Key, String>) -> Result<HashMap<Key, String>> {
    // Deserialize the given value into the matching Rust type.  When we find a matching type, we
    // serialize back out to the data store format, which is required to build a Settings object
    // through the data store deserialization code.
    let mut massaged_map = HashMap::with_capacity(input_map.len());
    for (key, in_val) in input_map {
        let serialized = if let Ok(b) = serde_json::from_str::<bool>(&in_val) {
            serialize_scalar(&b).context(error::SerializeSnafu)?
        } else if let Ok(u) = serde_json::from_str::<u64>(&in_val) {
            serialize_scalar(&u).context(error::SerializeSnafu)?
        } else if let Ok(f) = serde_json::from_str::<f64>(&in_val) {
            serialize_scalar(&f).context(error::SerializeSnafu)?
        } else {
            // No deserialization, already a string, just serialize
            serialize_scalar(&in_val).context(error::SerializeSnafu)?
        };
        massaged_map.insert(key, serialized);
    }
    Ok(massaged_map)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
// Main dispatch

/// Main entry point, dispatches subcommands.
async fn run() -> Result<()> {
    let (args, subcommand) = parse_args(env::args());
    trace!("Parsed args for subcommand {:?}: {:?}", subcommand, args);

    // We use TerminalMode::Stderr because apiclient users expect server response data on stdout.
    TermLogger::init(
        args.log_level,
        LogConfigBuilder::new()
            .add_filter_allow_str("apiclient")
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .context(error::LoggerSnafu)?;

    match subcommand {
        Subcommand::Raw(raw) => {
            let (status, body) =
                apiclient::raw_request(&args.socket_path, &raw.uri, &raw.method, raw.data)
                    .await
                    .context(error::RequestSnafu {
                        uri: &raw.uri,
                        method: &raw.method,
                    })?;

            // In raw mode, the user is expecting only the server response on stdout, so we more
            // carefully control other output and only write it to stderr.
            if log_enabled!(log::Level::Debug) {
                eprintln!("{}", status);
            }
            if !body.is_empty() {
                println!("{}", body);
            }
        }

        Subcommand::Apply(apply) => {
            apply::apply(&args.socket_path, apply.input_sources)
                .await
                .context(error::ApplySnafu)?;
        }

        Subcommand::Exec(exec) => {
            exec::exec(&args.socket_path, exec.command, exec.target, exec.tty)
                .await
                .context(error::ExecSnafu)?;
        }

        Subcommand::Get(get) => {
            let result = match get {
                GetArgs::Uri(uri) => get::get_uri(&args.socket_path, uri).await,
                GetArgs::Prefixes(prefixes) => get::get_prefixes(&args.socket_path, prefixes).await,
            };
            let value = result.context(error::GetSnafu)?;
            let pretty =
                serde_json::to_string_pretty(&value).expect("JSON Value already validated as JSON");
            println!("{}", pretty);
        }

        Subcommand::Reboot(_reboot) => {
            reboot::reboot(&args.socket_path)
                .await
                .context(error::RebootSnafu)?;
        }

        Subcommand::Set(set) => {
            let settings = match set {
                SetArgs::Simple(input_map) => {
                    // For key=val, we need some type information to deserialize into a Settings.
                    trace!("Original key=value input: {:#?}", input_map);
                    let massaged_map = massage_set_input(input_map)?;
                    trace!("Massaged key=value input: {:#?}", massaged_map);

                    // The data store deserialization code understands how to turn the key names
                    // (a.b.c) and serialized values into the nested Settings structure.
                    datastore::deserialization::from_map(&massaged_map)
                        .context(error::DeserializeMapSnafu)?
                }
                SetArgs::Json(json) => {
                    // No processing to do on JSON input; the format determines the types.  serde
                    // can turn a Value into the nested Settings structure itself.
                    serde_json::from_value(json).context(error::DeserializeJsonSnafu)?
                }
            };

            set::set(&args.socket_path, &settings)
                .await
                .context(error::SetSnafu)?;
        }

        Subcommand::Update(subcommand) => match subcommand {
            UpdateSubcommand::Check(_check) => {
                check(&args).await?;
            }

            UpdateSubcommand::Apply(apply) => {
                if apply.check {
                    let output = check(&args).await?;
                    // Exit early if no update is required, either because none is available or one
                    // is already applied and ready.
                    if !update::required(&output) {
                        return Ok(());
                    }
                }

                update::apply(&args.socket_path)
                    .await
                    .context(error::UpdateApplySnafu)?;

                // If the user requested it, and if we applied an update, reboot.  (update::apply
                // will fail if no update was available or it couldn't apply the update.)
                if apply.reboot {
                    reboot::reboot(&args.socket_path)
                        .await
                        .context(error::RebootSnafu)?;
                } else {
                    info!("Update has been applied and will take effect on next reboot.");
                }
            }

            UpdateSubcommand::Cancel(_cancel) => {
                update::cancel(&args.socket_path)
                    .await
                    .context(error::UpdateCancelSnafu)?;
            }
        },

        Subcommand::Report(subcommand) => match subcommand {
            ReportSubcommand::Cis(cis_args) => {
                let body = report::get_cis_report(
                    &args.socket_path,
                    "bottlerocket",
                    cis_args.format,
                    cis_args.level,
                )
                .await
                .context(error::ReportSnafu)?;

                if !body.is_empty() {
                    print!("{}", body);
                }
            }

            ReportSubcommand::CisK8s(cis_args) => {
                let body = report::get_cis_report(
                    &args.socket_path,
                    "kubernetes",
                    cis_args.format,
                    cis_args.level,
                )
                .await
                .context(error::ReportSnafu)?;

                if !body.is_empty() {
                    print!("{}", body);
                }
            }
        },
    }

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

mod error {
    use apiclient::{apply, exec, get, reboot, report, set, update};
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to apply settings: {}", source))]
        Apply { source: apply::Error },

        #[snafu(display("Unable to deserialize input JSON into model: {}", source))]
        DeserializeJson { source: serde_json::Error },

        // This is an important error, it's shown when the user uses 'apiclient set' with the
        // key=value form and we don't have enough data to deserialize the value.  It's not the
        // user's fault and so we want to be very clear and give an alternative.
        #[snafu(display("Unable to match your input to the data model.  We may not have enough type information.  Please try the --json input form.  Cause: {}", source))]
        DeserializeMap {
            source: datastore::deserialization::Error,
        },

        #[snafu(display("Failed to exec: {}", source))]
        Exec { source: exec::Error },

        #[snafu(display("Failed to get settings: {}", source))]
        Get { source: get::Error },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Failed to reboot: {}", source))]
        Reboot { source: reboot::Error },

        #[snafu(display("Failed {} request to '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Failed to get report: {}", source))]
        Report { source: report::Error },

        #[snafu(display("Unable to serialize data: {}", source))]
        Serialize { source: serde_json::Error },

        #[snafu(display("Failed to change settings: {}", source))]
        Set { source: set::Error },

        #[snafu(display("Failed to apply update: {}", source))]
        UpdateApply { source: update::Error },

        #[snafu(display("Failed to cancel update: {}", source))]
        UpdateCancel { source: update::Error },

        #[snafu(display("Failed to check for updates: {}", source))]
        UpdateCheck { source: update::Error },
    }
}
type Result<T> = std::result::Result<T, error::Error>;
