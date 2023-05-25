use aws_sdk_s3::error::SdkError;
use snafu::Snafu;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display(
        "Failed to create CFN stack '{}' in '{}': {}",
        stack_name,
        region,
        source
    ))]
    CreateStack {
        stack_name: String,
        region: String,
        source: SdkError<aws_sdk_cloudformation::operation::create_stack::CreateStackError>,
    },

    #[snafu(display(
        "Received CREATE_FAILED status for CFN stack '{}' in '{}'",
        stack_name,
        region
    ))]
    CreateStackFailure { stack_name: String, region: String },

    #[snafu(display("Error splitting shell command '{}': {}", command, source))]
    CommandSplit {
        command: String,
        source: shell_words::ParseError,
    },

    #[snafu(display("Error reading Infra.toml: {}", source))]
    Config { source: pubsys_config::Error },

    #[snafu(display(
        "Stuck in indefinite CREATE_IN_PROGRESS loop for CFN stack '{}' in '{}'",
        stack_name,
        region
    ))]
    CreateStackTimeout { stack_name: String, region: String },

    #[snafu(display("No stack data returned for CFN stack '{}' in {}", stack_name, region))]
    MissingStack { stack_name: String, region: String },

    #[snafu(display(
        "Failed to fetch stack details for CFN stack '{}' in '{}': {}",
        stack_name,
        region,
        source
    ))]
    DescribeStack {
        stack_name: String,
        region: String,
        source: SdkError<aws_sdk_cloudformation::operation::describe_stacks::DescribeStacksError>,
    },

    #[snafu(display("Missing environment variable '{}'", var))]
    Environment {
        var: String,
        source: std::env::VarError,
    },

    #[snafu(display("File already exists at '{}'", path.display()))]
    FileExists { path: PathBuf },

    #[snafu(display("Failed to open file at '{}': {}", path.display(), source))]
    FileOpen { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to read file at '{}': {}", path.display(), source))]
    FileRead { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to write file at '{}': {}", path.display(), source))]
    FileWrite { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to get bucket policy statement for bucket '{}'", bucket_name))]
    GetPolicyStatement { bucket_name: String },

    #[snafu(display("Failed to convert '{}' to yaml: {}", what, source))]
    InvalidJson {
        what: String,
        source: serde_json::Error,
    },

    #[snafu(display("Invalid path '{}' for '{}'", path.display(), thing))]
    InvalidPath { path: PathBuf, thing: String },

    #[snafu(display("Publication/Root key threshold must be <= {}, currently {}", num_keys.to_string(), threshold))]
    InvalidThreshold { threshold: String, num_keys: usize },

    #[snafu(display("Failed to convert updated Infra.toml information to yaml: {}", source))]
    InvalidYaml { source: serde_yaml::Error },

    #[snafu(display(
        "Failed to create keys due to invalid key config. Missing '{}'.",
        missing
    ))]
    KeyConfig { missing: String },

    #[snafu(display(
        "Failed to create new keys or access pre-existing keys in available_keys list."
    ))]
    KeyCreation,

    #[snafu(display("Logger setup error: {}", source))]
    Logger { source: log::SetLoggerError },

    #[snafu(display("Infra.toml is missing '{}'", missing))]
    MissingConfig { missing: String },

    #[snafu(display("Failed to create directory '{}': {}", path.display(), source))]
    Mkdir { path: PathBuf, source: io::Error },

    #[snafu(display("Failed to get parent of path '{}'", path.display()))]
    Parent { path: PathBuf },

    #[snafu(display("Failed to parse '{}' to int: {}", what, source))]
    ParseInt {
        what: String,
        source: std::num::ParseIntError,
    },

    #[snafu(display("Failed to find default region"))]
    DefaultRegion,

    #[snafu(display("Unable to parse stack status"))]
    ParseStatus,

    #[snafu(display(
        "Failed to find field '{}' after attempting to create resource '{}'",
        what,
        resource_name
    ))]
    ParseResponse { what: String, resource_name: String },

    #[snafu(display("Failed to convert '{}' to URL: {}", input, source))]
    ParseUrl {
        input: String,
        source: url::ParseError,
    },

    #[snafu(display("Failed to push object to bucket '{}': {}", bucket_name, source))]
    PutObject {
        bucket_name: String,
        source: SdkError<aws_sdk_s3::operation::put_object::PutObjectError>,
    },

    #[snafu(display(
        "Failed to update bucket policy for bucket '{}': {}",
        bucket_name,
        source
    ))]
    PutPolicy {
        bucket_name: String,
        source: SdkError<aws_sdk_s3::operation::put_bucket_policy::PutBucketPolicyError>,
    },

    #[snafu(display("Failed to create async runtime: {}", source))]
    Runtime { source: std::io::Error },

    #[snafu(display("'tuftool {}' returned {}", command, code))]
    TuftoolResult { command: String, code: String },

    #[snafu(display("Failed to start tuftool: {}", source))]
    TuftoolSpawn { source: io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;
