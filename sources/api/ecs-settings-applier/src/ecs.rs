/*!
# Introduction

ecs-settings-applier generates a configuration file for the ECS agent from Bottlerocket settings.

The configuration file for ECS is a JSON-formatted document with conditionally-defined keys and
embedded lists.  The structure and names of fields in the document can be found
[here](https://github.com/aws/amazon-ecs-agent/blob/a250409cf5eb4ad84a7b889023f1e4d2e274b7ab/agent/config/types.go).
*/
use log::debug;
use serde::Serialize;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::path::Path;
use std::{env, process};

const DEFAULT_ECS_CONFIG_PATH: &str = "/etc/ecs/ecs.config.json";
const VARIANT_ATTRIBUTE_NAME: &str = "bottlerocket.variant";

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
struct ECSConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster: Option<String>,

    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    instance_attributes: std::collections::HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    privileged_disabled: Option<bool>,

    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    available_logging_drivers: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    spot_instance_draining_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    warm_pools_support: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    image_pull_behavior: Option<u8>,

    #[serde(rename = "TaskIAMRoleEnabled")]
    task_iam_role_enabled: bool,

    #[serde(rename = "TaskIAMRoleEnabledForNetworkHost")]
    task_iam_role_enabled_for_network_host: bool,

    #[serde(rename = "SELinuxCapable")]
    selinux_capable: bool,

    #[serde(rename = "OverrideAWSLogsExecutionRole")]
    override_awslogs_execution_role: bool,

    #[serde(rename = "TaskENIEnabled")]
    task_eni_enabled: bool,

    #[serde(rename = "GPUSupportEnabled")]
    gpu_support_enabled: bool,

    #[serde(
        rename = "TaskMetadataSteadyStateRate",
        skip_serializing_if = "Option::is_none"
    )]
    metadata_service_rps: Option<i64>,

    #[serde(
        rename = "TaskMetadataBurstRate",
        skip_serializing_if = "Option::is_none"
    )]
    metadata_service_burst: Option<i64>,

    #[serde(rename = "ReservedMemory", skip_serializing_if = "Option::is_none")]
    reserved_memory: Option<u16>,

    #[serde(
        rename = "NumImagesToDeletePerCycle",
        skip_serializing_if = "Option::is_none"
    )]
    image_cleanup_delete_per_cycle: Option<i64>,

    image_cleanup_disabled: bool,

    #[serde(rename = "CNIPluginsPath")]
    cni_plugins_path: String,

    credentials_audit_log_file: String,

    data_dir: String,
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
pub(crate) async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = parse_args(env::args());
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).context(error::LoggerSnafu)?;

    // Get all ecs and autoscaling settings values for config file templates
    debug!("Requesting ecs and autoscaling settings values");
    let settings = schnauzer::get_settings(&args.socket_path)
        .await
        .context(error::SettingsSnafu)?;

    debug!("settings = {:#?}", settings.settings);
    let ecs = settings
        .settings
        .as_ref()
        .and_then(|s| s.ecs.as_ref())
        .context(error::ModelSnafu)?;

    let autoscaling = settings
        .settings
        .as_ref()
        .and_then(|s| s.autoscaling.as_ref())
        .context(error::ModelSnafu)?;

    let mut config = ECSConfig {
        cluster: ecs.cluster.clone(),
        privileged_disabled: ecs.allow_privileged_containers.map(|s| !s),
        available_logging_drivers: ecs
            .logging_drivers
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        spot_instance_draining_enabled: ecs.enable_spot_instance_draining,
        warm_pools_support: autoscaling.should_wait,
        image_pull_behavior: ecs.image_pull_behavior.as_ref().map(|b| b.as_u8()),

        // Task role support is always enabled
        task_iam_role_enabled: true,
        task_iam_role_enabled_for_network_host: true,

        // SELinux is always available
        selinux_capable: true,

        // Always supported with Docker newer than v17.11.0
        // See https://github.com/docker/engine/commit/c7cc9d67590dd11343336c121e3629924a9894e9
        override_awslogs_execution_role: true,

        // awsvpc mode is always available
        task_eni_enabled: true,

        gpu_support_enabled: cfg!(variant_flavor = "nvidia"),
        reserved_memory: ecs.reserved_memory,
        metadata_service_rps: ecs.metadata_service_rps,
        metadata_service_burst: ecs.metadata_service_burst,
        image_cleanup_delete_per_cycle: ecs.image_cleanup_delete_per_cycle,
        image_cleanup_disabled: !ecs.image_cleanup_enabled.unwrap_or(true),
        cni_plugins_path: "/usr/libexec/amazon-ecs-agent".to_string(),
        credentials_audit_log_file: "/var/log/ecs/audit.log".to_string(),
        data_dir: "/var/lib/ecs/data".to_string(),
        ..Default::default()
    };
    if let Some(os) = settings.os {
        config
            .instance_attributes
            .insert(VARIANT_ATTRIBUTE_NAME.to_string(), os.variant_id);
    }
    if let Some(attributes) = &ecs.instance_attributes {
        for (key, value) in attributes {
            config
                .instance_attributes
                .insert(key.to_string(), value.to_string());
        }
    }
    let serialized = serde_json::to_string(&config).context(error::SerializationSnafu)?;
    debug!("serialized = {}", serialized);

    write_to_disk(DEFAULT_ECS_CONFIG_PATH, serialized).context(error::FSSnafu {
        path: DEFAULT_ECS_CONFIG_PATH,
    })?;
    Ok(())
}

/// Writes the rendered data at the proper location
fn write_to_disk<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    if let Some(dirname) = path.as_ref().parent() {
        fs::create_dir_all(dirname)?;
    };

    fs::write(path, contents)
}

// Stores user-supplied arguments.
struct Args {
    socket_path: String,
}

fn parse_args(args: env::Args) -> Args {
    let mut socket_path = None;
    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
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
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
    }
}

// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ (-s | --socket-path) PATH ]

    Socket path defaults to {}",
        program_name,
        constants::API_SOCKET
    );
    process::exit(2);
}

type Result<T> = std::result::Result<T, error::Error>;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Failed to read settings: {}", source))]
        Settings {
            source: schnauzer::Error,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger {
            source: log::SetLoggerError,
        },

        Model,

        #[snafu(display("Failed to serialize ECS config: {}", source))]
        Serialization {
            source: serde_json::error::Error,
        },

        #[snafu(display("Filesystem operation for path {} failed: {}", path, source))]
        FS {
            path: &'static str,
            source: std::io::Error,
        },
    }
}
