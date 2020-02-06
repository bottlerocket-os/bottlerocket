/*!
# Introduction

This tool is meant to update an existing TUF repo with new contents and sign the updated contents.
Given a set of environment variables, it will pull down an existing TUF repo and update the manifest, targets.json, snapshot.json, and timestamp.json.
Using a signing key that it pulls down via SSM Secure Parameters, it will sign the updated files, along with any new targets and leave them in a known location to be deployed to a "real" TUF repo at a later step.

# Running

In order the run this code, you must have:
* Current `Thar` code repository (more specifically `Release.toml`, and a trusted `root.json`)
* Built Thar artifacts in a directory (the images that end up in `/build` and suffixed with `.lz4`)
* The metadata and target URLs for an existing TUF repository (most likely in S3)

Currently the code expects the following environment variables to be set:
* `CODEBUILD_SRC_DIR` (subject to change) This is the directory where your `Thar` repository lives
* `ARCH` : architecture for your current set of images (i.e. `x86_64`)
* `FLAVOR` : Variant of Thar for your current set of images (i.e. `aws-k8s`)
* `INPUT_BUILDSYS_ARTIFACTS` : A directory containing the built Thar images
* `METADATA_URL` : Metadata URL for your existing TUF repo
* `TARGET_URL` : Target URL for your existing TUF repo
* `REFRESH_DAYS` : After how many days does metadata expire? (an integer, i.e. `7`)
* `TIMESTAMP_REFRESH_DAYS` : After how many days does `timestamp.json` expire? (an integer, i.e. `7`)
* `SIGNING_ROLE_ARN` : ARN for a role that allows access to signing keys (most likely in another account)
* `SIGNING_KEY_PARAMETER_NAME` : The SSM parameter key name for the signing key

# Output

After a successful run of this code, you will have a directory `/tmp/tuf_out` which will contain `/metadata` and `/target` directories.
All items (other than `manifest.json`) are signed and are suitable for syncing to your "real" TUF repository.
*/

#[macro_use]
extern crate log;

use chrono::{Duration, Utc};
use data_store_version::Version as DataVersion;
use olpc_cjson::CanonicalFormatter;
use ring::digest::{digest, Context, SHA256, SHA256_OUTPUT_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use rusoto_core::request::HttpClient;
use rusoto_ssm::{GetParameterRequest, Ssm, SsmClient};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use semver::Version as SemVer;
use serde::Serialize;
use serde_derive::Deserialize;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroU64;
use std::path::Path;
use std::str::FromStr;
use std::{fs, process};
use tempdir::TempDir;
use tough::schema::decoded::{Decoded, Hex};
use tough::schema::{
    Hashes, Role, RoleType, Root, Signature, Signed, SnapshotMeta, Target, TimestampMeta,
};
use tough::sign::{parse_keypair, Sign};
use tough::{HttpTransport, Limits, Repository, Settings};
use update_metadata::{Images, Manifest};

const EXISTING_TUF_REPO_DIR: &str = "/tmp/tuf_in";
const UPDATED_TUF_REPO_DIR: &str = "/tmp/tuf_out";
const ROOT_JSON: &str = "root.json";
const TUF_MANIFEST_JSON: &str = "manifest.json";
const RELEASE_TOML: &str = "Release.toml";
const FILES_TO_SIGN: &[&str] = &["boot", "root", "verity"];
const OS_NAME: &str = "thar";

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Unable to add migration to manifest: {}", source))]
        AddMigration {
            source: update_metadata::error::Error,
        },

        #[snafu(display("Unable to add update wave to manifest: {}", source))]
        AddWave {
            source: update_metadata::error::Error,
        },

        #[snafu(display("Current UTC time should be non-zero"))]
        CurrentTime {},

        #[snafu(display(
            "Failed to create data store version from {}: {}",
            version_string,
            source
        ))]
        DataVersion {
            version_string: String,
            source: data_store_version::error::Error,
        },

        #[snafu(display("Missing required environment variables: {}", source))]
        EnvironmentVariables { source: envy::Error },

        #[snafu(display("Failed to create {}: {}", path.display(), source))]
        FileCreate {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to copy {} to {}: {}", src.display(), dst.display(), source))]
        FileCopy {
            src: PathBuf,
            dst: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to open {}: {}", path.display(), source))]
        FileOpen {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to read {}: {}", path.display(), source))]
        FileRead {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to write to {}: {}", path.display(), source))]
        FileWriteJson {
            path: PathBuf,
            source: serde_json::Error,
        },

        #[snafu(display("Failed to create HTTP Client: {}", source))]
        HttpClientCreate {
            source: rusoto_core::request::TlsError,
        },

        #[snafu(display("Failed to serialize JSON: {}", source))]
        JSONSerialize { source: serde_json::error::Error },

        #[snafu(display("Failed to deserialize JSON: {}", source))]
        JSONDeserialize { source: serde_json::error::Error },

        #[snafu(display("Unable to parse keypair: {}", source))]
        KeyPairParse { source: tough::error::Error },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: simplelog::TermLogError },

        #[snafu(display("Failed to update manifest: {}", source))]
        ManifestUpdate {
            source: update_metadata::error::Error,
        },

        #[snafu(display("Missing image name: {}", name))]
        MissingImageName { name: String },

        #[snafu(display("Failed to open trusted root metadata file {}: {}", path.display(), source))]
        OpenRoot {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Missing target: {}", target))]
        MissingTarget { target: String },

        #[snafu(display("Error reading target from TUF repository: {}", source))]
        ReadTarget { source: tough::error::Error },

        #[snafu(display("Unable to find keys for root in signing key"))]
        KeysForRoot {},

        #[snafu(display("Nonexistent role keys for current root.json"))]
        RoleKeys {},

        #[snafu(display("Failed to create semver from {}: {}", version_string, source))]
        SemVer {
            version_string: String,
            source: semver::SemVerError,
        },

        #[snafu(display("Failed to sign message"))]
        Sign { source: tough::error::Error },

        #[snafu(display("Failed to serialize role for signing: {}", source))]
        SignJson { source: serde_json::Error },

        #[snafu(display(
            "Failed to retrieve signing key SSM parameter: '{}': {}",
            parameter,
            source
        ))]
        SSMParameterRetrieve {
            parameter: String,
            source: rusoto_core::RusotoError<rusoto_ssm::GetParameterError>,
        },

        #[snafu(display("Unable to read SSM parameter: '{}'", parameter))]
        SSMParameterRead { parameter: String },

        #[snafu(display("Failed to create temporary directory: {}", source))]
        TempDir { source: std::io::Error },

        #[snafu(display("Failed to load TUF repository: {}", source))]
        TUFRepoLoad { source: tough::error::Error },

        #[snafu(display("Unexpected image name in constants"))]
        UnexpectedImageName {},
    }
}

type Result<T> = std::result::Result<T, error::Error>;

// Contains the environment variables we need to execute the program
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct EnvVars {
    codebuild_src_dir: String,
    arch: String,
    flavor: String,
    input_buildsys_artifacts: String,
    metadata_url: String,
    refresh_days: i64,
    signing_role_arn: String,
    signing_key_parameter_name: String,
    target_url: String,
    timestamp_refresh_days: i64,
}

// Represents Release.toml
// TODO: Make this into a crate
#[derive(Deserialize, Debug)]
struct ReleaseInfo {
    version: String,
    datastore_version: String,
    migrations: Vec<Migration>,
}

// Represents migration info from Release.toml
#[derive(Deserialize, Debug)]
struct Migration {
    from: String,
    to: String,
    names: Vec<String>,
}

// Contains related information needed to sign metadata
struct SigningMaterial {
    root: Signed<Root>,
    keys: RootKeys,
    rng: SystemRandom,
    version: NonZeroU64,
}

type RootKeys = HashMap<Decoded<Hex>, Box<dyn Sign>>;

// FIXME: This code (not quite verbatim) lives in tuftool and should be librarized
// Get the approprate keys from root that match the current signing keypair
fn keys_for_root(key: String, root: &Root) -> Result<RootKeys> {
    let mut map = HashMap::new();
    let key_pair: Box<dyn Sign> =
        Box::new(parse_keypair(&key.as_bytes().to_vec()).context(error::KeyPairParse)?);
    if let Some((keyid, _)) = root
        .keys
        .iter()
        .find(|(_, key)| key_pair.tuf_key() == **key)
    {
        map.insert(keyid.clone(), key_pair);
    }

    Ok(map)
}

// Get the signing key from the SSM parameter
fn get_signing_key(env: &EnvVars) -> Result<String> {
    // Assume a role that has access to signing keys
    // Make an sts client to get credentials
    // Create an ssm client with those credentials
    let sts_client = StsClient::new(Default::default());
    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts_client,
        env.signing_role_arn.to_string(),
        "sign-tuf-repo".to_owned(),
        Some("update_sign_tuf_repo".to_string()),
        None,
        None,
        None,
    );
    let http_client = HttpClient::new().context(error::HttpClientCreate)?;
    let ssm_client = SsmClient::new_with(http_client, provider, Default::default());

    let get_signing_key_req = GetParameterRequest {
        name: env.signing_key_parameter_name.to_string(),
        with_decryption: Some(true),
    };
    match ssm_client.get_parameter(get_signing_key_req).sync() {
        Ok(ssm_return) => {
            if let Some(signing_key) = ssm_return.parameter {
                if let Some(key) = signing_key.value {
                    return Ok(key);
                }
            }
            return error::SSMParameterRead {
                parameter: &env.signing_key_parameter_name,
            }
            .fail();
        }
        Err(e) => {
            return Err(e).context(error::SSMParameterRetrieve {
                parameter: &env.signing_key_parameter_name,
            });
        }
    }
}

// Builds the names of the images we expect to come out of the build process
// FIXME: This deserves extra thought. Should the build process push these?
// There are obvious disadvantages here, however one advantage of being
// very strict but naive is that this code would need to be edited and
// pushed to change what actually gets signed.
fn build_target_names(env: &EnvVars, release: &ReleaseInfo) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    let name_stub = format!(
        "{}-{}-{}-v{}",
        OS_NAME, env.arch, env.flavor, release.version
    );
    for file in FILES_TO_SIGN {
        let name = match file.as_ref() {
            "boot" | "root" => format!("{}-{}.ext4.lz4", &name_stub, &file),
            "verity" => format!("{}-{}.verity.lz4", &name_stub, &file),
            _ => return error::UnexpectedImageName {}.fail(),
        };
        map.insert(file.to_string(), name.to_string());
    }
    Ok(map)
}

// Calculate the length and hash of a target located at target_dir/target_name,
// create a target object with that info, and lastly, copy the file to /targets.
// FIXME: This code (not quite verbatim) lives in tuftool and should be librarized
fn write_target<S, P>(root: &Root, target_dir: P, target_name: S) -> Result<(String, Target)>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let target_name = target_name.as_ref();
    let target_dir = target_dir.as_ref();
    let target_path = target_dir.join(&target_name);

    // Calculate the length and hash of the supplied target file
    let mut file = File::open(&target_path).context(error::FileOpen { path: &target_path })?;
    let mut digest = Context::new(&SHA256);
    let mut buf = [0; 8 * 1024];
    let mut length = 0;
    loop {
        match file
            .read(&mut buf)
            .context(error::FileRead { path: &target_path })?
        {
            0 => break,
            n => {
                digest.update(&buf[..n]);
                length += n as u64;
            }
        }
    }
    let target = Target {
        length,
        hashes: Hashes {
            sha256: Decoded::from(digest.finish().as_ref().to_vec()),
            _extra: HashMap::new(),
        },
        custom: HashMap::new(),
        _extra: HashMap::new(),
    };

    // Using the hash, create a filename to copy to in /targets
    let output_dir = Path::new(UPDATED_TUF_REPO_DIR).join("targets");
    let dst = if root.consistent_snapshot {
        output_dir.join(format!(
            "{}.{}",
            hex::encode(&target.hashes.sha256),
            target_name
        ))
    } else {
        output_dir.join(&target_name)
    };

    // Create the destination folder if it doesn't exist
    fs::create_dir_all(&output_dir).context(error::FileCreate { path: &output_dir })?;
    fs::copy(&target_path, &dst).context(error::FileCopy {
        src: &target_path,
        dst: &dst,
    })?;

    Ok((target_name.to_string(), target))
}

// Write signed metadata to the TUF '/metadata' folder
// FIXME: This code (not quite verbatim) lives in tuftool and should be librarized
fn write_metadata<T: Role + Serialize>(
    role: T,
    signing_material: &SigningMaterial,
    filename: &'static str,
) -> Result<([u8; SHA256_OUTPUT_LEN], u64)> {
    let metadata_dir = Path::new(UPDATED_TUF_REPO_DIR).join("metadata");
    fs::create_dir_all(&metadata_dir).context(error::FileCreate {
        path: &metadata_dir,
    })?;

    let path = metadata_dir.join(
        if T::TYPE != RoleType::Timestamp && signing_material.root.signed.consistent_snapshot {
            format!("{}.{}", signing_material.version, filename)
        } else {
            filename.to_owned()
        },
    );

    let mut role = Signed {
        signed: role,
        signatures: Vec::new(),
    };
    sign_metadata(
        &signing_material.root.signed,
        &signing_material.keys,
        T::TYPE,
        &mut role,
        &signing_material.rng,
    )?;

    let mut buf = serde_json::to_vec_pretty(&role).context(error::FileWriteJson { path: &path })?;
    buf.push(b'\n');
    std::fs::write(&path, &buf).context(error::FileCreate { path: &path })?;

    let mut sha256 = [0; SHA256_OUTPUT_LEN];
    sha256.copy_from_slice(digest(&SHA256, &buf).as_ref());
    Ok((sha256, buf.len() as u64))
}

// Sign a given piece of metadata
// FIXME: This code (not quite verbatim) lives in tuftool and should be librarized
fn sign_metadata<T: Serialize>(
    root: &Root,
    keys: &RootKeys,
    role_type: RoleType,
    role: &mut Signed<T>,
    rng: &dyn SecureRandom,
) -> Result<()> {
    if let Some(role_keys) = root.roles.get(&role_type) {
        for (keyid, key) in keys {
            if role_keys.keyids.contains(&keyid) {
                let mut data = Vec::new();
                let mut ser =
                    serde_json::Serializer::with_formatter(&mut data, CanonicalFormatter::new());
                role.signed.serialize(&mut ser).context(error::SignJson)?;
                let sig = key.sign(&data, rng).context(error::Sign)?;
                role.signatures.push(Signature {
                    keyid: keyid.clone(),
                    sig: sig.into(),
                });
            }
        }
    } else {
        return error::RoleKeys {}.fail();
    }

    Ok(())
}

// TODO: Moar logs? (debug/trace?)
fn run() -> Result<()> {
    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    // Get the configured environment variables
    info!("Parsing environment variables");
    let env_vars = match envy::from_env::<EnvVars>() {
        Ok(env_vars) => env_vars,
        Err(error) => return Err(error).context(error::EnvironmentVariables)?,
    };

    // Parse the Release.toml into a ReleaseInfo struct
    // Release.toml is located at ${CODEBUILD_SRC_DIR}/Release.toml
    info!("Reading and deserializing Release.toml");
    let release_path = Path::new(&env_vars.codebuild_src_dir).join(RELEASE_TOML);
    let release_reader = File::open(&release_path).context(error::FileOpen {
        path: &release_path,
    })?;
    let release: ReleaseInfo =
        serde_json::from_reader(release_reader).context(error::JSONDeserialize)?;

    // Load TUF repository into memory from metadata/target paths
    info!("Pulling TUF repository");
    let transport = HttpTransport::new();
    let repo_dir = TempDir::new(EXISTING_TUF_REPO_DIR).context(error::TempDir)?;
    // ${CODEBUILD_SRC_DIR}/packages/workspaces/root.json
    let root_json_path = Path::new(&env_vars.codebuild_src_dir)
        .join("packages")
        .join("workspaces")
        .join(ROOT_JSON);
    let tuf_repo = Repository::load(
        &transport,
        Settings {
            root: File::open(&root_json_path).context(error::OpenRoot {
                path: &root_json_path,
            })?,
            datastore: repo_dir.path(),
            metadata_base_url: &env_vars.metadata_url,
            target_base_url: &env_vars.target_url,
            limits: Limits {
                ..tough::Limits::default()
            },
        },
    )
    .context(error::TUFRepoLoad)?;

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Read the manifest target into memory so we can update it
    info!("Parsing manifest via TUF repo");
    let manifest_reader = tuf_repo.read_target(TUF_MANIFEST_JSON);
    let mut manifest: Manifest = match manifest_reader {
        Err(error) => return Err(error).context(error::ReadTarget),
        Ok(manifest_reader) => match manifest_reader {
            Some(reader) => serde_json::from_reader(reader).context(error::JSONSerialize)?,
            None => {
                return error::MissingTarget {
                    target: TUF_MANIFEST_JSON,
                }
                .fail()
            }
        },
    };

    // If there are migrations, add them to the Manifest
    if !release.migrations.is_empty() {
        for migration in release.migrations.iter() {
            if release.version == migration.to {
                info!("Adding migrations to manifest");
                let from = DataVersion::from_str(&migration.from).context(error::DataVersion {
                    version_string: &migration.from,
                })?;
                let to = DataVersion::from_str(&migration.to).context(error::DataVersion {
                    version_string: &migration.to,
                })?;
                // "true" in this call will append the migrations to the list rather than
                // overwrite them
                manifest
                    .add_migration(true, from, to, migration.names.clone())
                    .context(error::AddMigration)?;
            }
        }
    }

    // Add the current update images to the Manifest
    // TODO: This needs more validation. We need to make sure that the image
    // actually exists before and that it's named correctly
    let new_targets = build_target_names(&env_vars, &release)?;
    let images = Images {
        boot: new_targets
            .get("boot")
            .context(error::MissingImageName {
                name: "boot".to_string(),
            })?
            .to_string(),
        root: new_targets
            .get("root")
            .context(error::MissingImageName {
                name: "root".to_string(),
            })?
            .to_string(),
        hash: new_targets
            .get("verity")
            .context(error::MissingImageName {
                name: "verity".to_string(),
            })?
            .to_string(),
    };
    let release_semver = SemVer::parse(&release.version).context(error::SemVer {
        version_string: release.version,
    })?;
    let datastore_version =
        DataVersion::from_str(&release.datastore_version).context(error::DataVersion {
            version_string: &release.datastore_version,
        })?;

    // Add the update to the manifest.
    info!("Adding current update to manifest");
    manifest
        .add_update(
            release_semver.clone(),
            Some(release_semver.clone()),
            datastore_version,
            env_vars.arch.clone(),
            env_vars.flavor.clone(),
            images,
        )
        .context(error::ManifestUpdate)?;

    // Add waves to the manifest
    // FIXME: Make waves configurable for this code via args/env variables,
    // an issue exists to set "profiles" that can be referred to:
    // https://github.com/amazonlinux/PRIVATE-thar/issues/596
    info!("Adding wave(s) to manifest");
    let now = Utc::now();
    // First wave starts today
    manifest
        .add_wave(
            env_vars.flavor.clone(),
            env_vars.arch.clone(),
            release_semver.clone(),
            512,
            now.clone(),
        )
        .context(error::AddWave)?;
    // Second wave starts tomorrow
    manifest
        .add_wave(
            env_vars.flavor.clone(),
            env_vars.arch.clone(),
            release_semver.clone(),
            1024,
            now.clone() + Duration::days(1),
        )
        .context(error::AddWave)?;
    // Third wave starts the day after tomorrow
    manifest
        .add_wave(
            env_vars.flavor.clone(),
            env_vars.arch.clone(),
            release_semver.clone(),
            1576,
            now.clone() + Duration::days(2),
        )
        .context(error::AddWave)?;

    // Write the updated manifest to a file. This must be a file for now as we
    // compute hashes and size for it in the next step, and copy it to the
    // final '/targets' dir
    let manifest_path = Path::new(EXISTING_TUF_REPO_DIR).join("manifest.json");
    let pretty_manifest =
        serde_json::to_string_pretty(&manifest).context(error::JSONDeserialize)?;
    fs::write(&manifest_path, &pretty_manifest).context(error::FileCreate {
        path: manifest_path,
    })?;
    // Write the updated manifest to /targets
    info!("Writing manifest to targets");
    let (manifest_name, manifest) = write_target(
        &tuf_repo.root().signed,
        &EXISTING_TUF_REPO_DIR,
        "manifest.json",
    )?;

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Create the items necessary to sign metadata
    let metadata_version =
        NonZeroU64::new(Utc::now().timestamp() as u64).context(error::CurrentTime)?;
    let timestamp_expiration = Utc::now() + Duration::days(env_vars.timestamp_refresh_days);
    let other_expiration = Utc::now() + Duration::days(env_vars.refresh_days);
    let signing_key = get_signing_key(&env_vars)?;
    let root_keys = keys_for_root(signing_key, &tuf_repo.root().signed)?;
    let signing_material = SigningMaterial {
        root: tuf_repo.root().clone(),
        keys: root_keys,
        rng: SystemRandom::new(),
        version: metadata_version,
    };

    // Clone existing 'targets' struct from the TUF repo; we will update it
    info!("Updating 'targets.json'");
    let mut targets = tuf_repo.targets().clone().signed;
    // Add the previously updated manifest to targets
    targets.targets.insert(manifest_name, manifest);
    // Add all new images to /targets and 'targets' (the object)
    for (_, new_target) in new_targets {
        let (target_name, target) = write_target(
            &tuf_repo.root().signed,
            &env_vars.input_buildsys_artifacts,
            new_target,
        )?;

        targets.targets.insert(target_name, target);
    }

    // Update the targets version and expiration
    targets.version = metadata_version;
    targets.expires = other_expiration;
    let (targets_sha256, targets_length) =
        write_metadata(targets, &signing_material, "targets.json")?;

    // Fetch snapshot, update version and expiration, sign
    info!("Updating 'snapshot.json'");
    let mut snapshot = tuf_repo.snapshot().clone().signed;
    snapshot.version = metadata_version;
    snapshot.expires = other_expiration;
    snapshot.meta.insert(
        "targets.json".to_owned(),
        SnapshotMeta {
            hashes: Some(Hashes {
                sha256: targets_sha256.to_vec().into(),
                _extra: HashMap::new(),
            }),
            length: Some(targets_length),
            version: metadata_version,
            _extra: HashMap::new(),
        },
    );
    let (snapshot_sha256, snapshot_length) =
        write_metadata(snapshot, &signing_material, "snapshot.json")?;

    // Fetch timestamp, update version and expiration, sign
    info!("Updating 'timestamp.json'");
    let mut timestamp = tuf_repo.timestamp().clone().signed;
    timestamp.version = metadata_version;
    timestamp.expires = timestamp_expiration;
    timestamp.meta.insert(
        "snapshot.json".to_owned(),
        TimestampMeta {
            hashes: Hashes {
                sha256: snapshot_sha256.to_vec().into(),
                _extra: HashMap::new(),
            },
            length: snapshot_length,
            version: metadata_version,
            _extra: HashMap::new(),
        },
    );
    write_metadata(timestamp, &signing_material, "timestamp.json")?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
