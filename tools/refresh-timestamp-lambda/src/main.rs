/*!
# Introduction

This is a lambda function that periodically refreshes a TUF repository's `timestamp.json` metadata file's expiration date and version.

Every time this lambda runs, the expiration date is pushed out by a custom number of days from the current date (defined by the lambda event).

# Compiling & Building

This rust lambda needs to be statically compiled and linked against [musl-libc](https://www.musl-libc.org/).
Currently building with [clux/muslrust](https://github.com/clux/muslrust).

To build, run `make build`.
Then, to zip the lambda bootstrap binary, run `make zip`.

# Setting up the Lambda with CloudFormation

Use `timestamp-signer.yaml` to create an assumable role in the account where the signing key resides. This lets the lambda have access to the signing key.

Use `tuf-repo-access-role.yaml` to create an assumable role in the account where the TUF repository bucket resides. This lets the lambda have access to update `timestamp.json`.

Use `TimestampRefreshLambda.yaml` to create the CFN stack for this lambda.

*/

#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use chrono::{Duration, Utc};
use failure::format_err;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self, error, info};
use olpc_cjson::CanonicalFormatter;
use ring::rand::SystemRandom;
use rusoto_core::request::HttpClient;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use rusoto_ssm::{GetParameterRequest, Ssm, SsmClient};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use serde::export::from_utf8_lossy;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use simple_logger;
use std::error::Error;
use std::io::Read;
use std::num::NonZeroU64;
use tempfile::tempdir;
use tough::schema::{RoleType, Signature};
use tough::sign::Sign;
use tough::{HttpTransport, Limits, Repository, Settings};

// Contains the environment variables we need to execute the program
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct EnvVars {
    bucket_access_role_arn: String,
    signing_role_arn: String,
    key_parameter_name: String,
    bucket_name: String,
    metadata_path: String,
    metadata_url: String,
    targets_url: String,
    refresh_validity_days: String,
}

#[derive(Deserialize, Copy, Clone)]
struct CustomEvent {}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(handler);
    Ok(())
}

fn get_signing_key(
    ssm_client: &SsmClient,
    key_parameter_name: String,
) -> Result<String, HandlerError> {
    let get_signing_key_req = GetParameterRequest {
        name: key_parameter_name,
        with_decryption: Some(true),
    };
    match ssm_client.get_parameter(get_signing_key_req).sync() {
        Ok(signing_key) => {
            if let Some(signing_key) = signing_key.parameter {
                if let Some(key) = signing_key.value {
                    return Ok(key);
                }
            }
            bail!("Parameter unable to be read")
        }
        Err(error) => {
            error!("failed to retrieve signing key parameter");
            Err(HandlerError::from(failure::Error::from(error)))
        }
    }
}

fn handler(_e: CustomEvent, _c: Context) -> Result<CustomOutput, HandlerError> {
    refresh_timestamp().map_err(HandlerError::from)
}

fn refresh_timestamp() -> failure::Fallible<CustomOutput> {
    info!("Parsing environment variables");
    // Get the configured environment variables
    let env_vars: EnvVars =
        envy::from_env().map_err(|e| HandlerError::from(failure::Error::from(e)))?;

    let s3_http_client = HttpClient::new()?;
    let s3_sts_client = StsClient::new(Default::default());
    let s3_session_cred_provider = StsAssumeRoleSessionCredentialsProvider::new(
        s3_sts_client,
        env_vars.bucket_access_role_arn,
        "sign-timestamp-access-bucket".to_owned(),
        None,
        None,
        None,
        None,
    );
    let s3_client =
        S3Client::new_with(s3_http_client, s3_session_cred_provider, Default::default());

    let ssm_http_client = HttpClient::new()?;
    let ssm_sts_client = StsClient::new(Default::default());
    let ssm_session_cred_provider = StsAssumeRoleSessionCredentialsProvider::new(
        ssm_sts_client,
        env_vars.signing_role_arn,
        "sign-timestamp-get-key".to_owned(),
        None,
        None,
        None,
        None,
    );
    let ssm_client = SsmClient::new_with(
        ssm_http_client,
        ssm_session_cred_provider,
        Default::default(),
    );

    // Retrieves signing key from SSM parameter
    let signing_key = get_signing_key(&ssm_client, env_vars.key_parameter_name)?;
    let keypair: Box<dyn Sign> = Box::new(tough::sign::parse_keypair(
        &signing_key.as_bytes().to_vec(),
    )?);

    // Create the datastore path for storing the metadata files
    let datastore = tempdir()?;

    // Read root.json from the TUF repo for the root keys
    // Note: We're retrieving a root.json directly from the TUF repository. We're not actually updating to anything, just refreshing
    // the timestamp metadata file of the TUF repository itself.
    let get_root_request = GetObjectRequest {
        bucket: env_vars.bucket_name.to_owned(),
        key: (env_vars.metadata_path.to_owned() + "/1.root.json"),
        ..GetObjectRequest::default()
    };
    let mut buffer = Vec::new();
    let root_json = match s3_client.get_object(get_root_request).sync()?.body {
        Some(body) => {
            body.into_blocking_read().read_to_end(&mut buffer)?;
            from_utf8_lossy(&buffer)
        }
        None => return Err(format_err!("Empty timestamp.json file")),
    };

    info!("Loading TUF repo");
    let transport = HttpTransport::new();
    let repo = Repository::load(
        &transport,
        Settings {
            root: root_json.as_bytes(),
            datastore: datastore.path(),
            metadata_base_url: &env_vars.metadata_url,
            target_base_url: &env_vars.targets_url,
            limits: Limits {
                ..tough::Limits::default()
            },
        },
    )?;

    let mut timestamp = repo.timestamp().clone();
    let now = Utc::now();
    let new_version = if let Some(version) = NonZeroU64::new(now.timestamp() as u64) {
        version
    } else {
        return Err(format_err!("Couldn't retrieve current UTC timestamp"));
    };

    info!(
        "Updating version from {} to {}",
        timestamp.signed.version, new_version
    );
    timestamp.signed.version = new_version;

    let new_expiration = now + Duration::days(env_vars.refresh_validity_days.parse::<i64>()?);
    info!(
        "Updating expiration date from {} to {}",
        timestamp.signed.expires.to_rfc3339(),
        new_expiration.to_rfc3339()
    );
    timestamp.signed.expires = new_expiration;

    let signed_root = repo.root();
    let key_id = if let Some(key) = signed_root
        .signed
        .keys
        .iter()
        .find(|(_, key)| keypair.tuf_key() == **key)
    {
        key.0
    } else {
        error!("Couldn't find key pair");
        return Err(format_err!("Couldn't find key"));
    };

    let mut data = Vec::new();
    let role_key = match signed_root.signed.roles.get(&RoleType::Timestamp) {
        Some(key) => key,
        None => return Err(format_err!("Unable to find role keys")),
    };
    if role_key.keyids.contains(key_id) {
        let mut ser = serde_json::Serializer::with_formatter(&mut data, CanonicalFormatter::new());
        timestamp.signed.serialize(&mut ser)?;

        let sig = keypair.sign(&data, &SystemRandom::new())?;
        timestamp.signatures.clear();
        timestamp.signatures.push(Signature {
            keyid: key_id.clone(),
            sig: sig.into(),
        });
    }

    let body = serde_json::to_vec_pretty(&timestamp)?;
    let put_request = PutObjectRequest {
        bucket: env_vars.bucket_name,
        key: (env_vars.metadata_path + "/timestamp.json"),
        body: Some(body.into()),
        ..PutObjectRequest::default()
    };
    s3_client.put_object(put_request).sync()?;

    Ok(CustomOutput {
        message: format!(
            "new version = {}, new expiration date = {}, signed data: {}",
            timestamp.signed.version,
            timestamp.signed.expires,
            from_utf8_lossy(&data)
        ),
    })
}
