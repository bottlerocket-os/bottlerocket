use crate::error::{self, Result};
use crate::source::KeySource;
use chrono::{DateTime, Timelike, Utc};
use maplit::hashmap;
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tempfile::NamedTempFile;
use tough_schema::decoded::{Decoded, Hex};
use tough_schema::key::Key;
use tough_schema::{RoleKeys, RoleType, Root, Signed};

#[derive(Debug, StructOpt)]
pub(crate) enum Command {
    /// Create a new root.json metadata file
    Init {
        /// Path to new root.json
        path: PathBuf,
    },
    /// Set the expiration time for root.json
    Expire {
        /// Path to root.json
        path: PathBuf,
        /// When to expire
        time: DateTime<Utc>,
    },
    /// Set the signature count threshold for a role
    SetThreshold {
        /// Path to root.json
        path: PathBuf,
        /// The role to set
        role: RoleType,
        /// The new threshold
        threshold: NonZeroU64,
    },
    /// Add a key (public or private) to a role
    AddKey {
        /// Path to root.json
        path: PathBuf,
        /// The role to add the key to
        role: RoleType,
        /// The new key
        key_path: KeySource,
    },
}

macro_rules! role_keys {
    ($threshold:expr) => {
        RoleKeys {
            keyids: Vec::new(),
            threshold: $threshold,
            _extra: HashMap::new(),
        }
    };

    () => {
        // absurdly high threshold value so that someone realizes they need to change this
        role_keys!(NonZeroU64::new(1507).unwrap())
    };
}

impl Command {
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            Command::Init { path } => write_json(
                path,
                &Signed {
                    signed: Root {
                        spec_version: "1.0".to_owned(),
                        consistent_snapshot: true,
                        version: NonZeroU64::new(1).unwrap(),
                        expires: round_time(Utc::now()),
                        keys: HashMap::new(),
                        roles: hashmap! {
                            RoleType::Root => role_keys!(),
                            RoleType::Snapshot => role_keys!(),
                            RoleType::Targets => role_keys!(),
                            RoleType::Timestamp => role_keys!(),
                        },
                        _extra: HashMap::new(),
                    },
                    signatures: Vec::new(),
                },
            ),
            Command::Expire { path, time } => {
                let mut root = load_root(path)?;
                root.signed.expires = round_time(*time);
                write_json(path, &root)
            }
            Command::SetThreshold {
                path,
                role,
                threshold,
            } => {
                let mut root = load_root(path)?;
                root.signed
                    .roles
                    .entry(*role)
                    .and_modify(|rk| rk.threshold = *threshold)
                    .or_insert_with(|| role_keys!(*threshold));
                write_json(path, &root)
            }
            Command::AddKey {
                path,
                role,
                key_path,
            } => {
                let mut root = load_root(path)?;
                let key_pair = key_path.as_public_key()?;
                let key_id = add_key(&mut root.signed, key_pair)?;
                let entry = root
                    .signed
                    .roles
                    .entry(*role)
                    .or_insert_with(|| role_keys!());
                if !entry.keyids.contains(&key_id) {
                    entry.keyids.push(key_id);
                }
                write_json(path, &root)
            }
        }
    }
}

fn round_time(time: DateTime<Utc>) -> DateTime<Utc> {
    // `Timelike::with_nanosecond` returns None only when passed a value >= 2_000_000_000
    time.with_nanosecond(0).unwrap()
}

fn load_root(path: &Path) -> Result<Signed<Root>> {
    serde_json::from_reader(File::open(path).context(error::FileOpen { path })?)
        .context(error::FileParseJson { path })
}

fn write_json<T: Serialize>(path: &Path, json: &T) -> Result<()> {
    // Use `tempfile::NamedTempFile::persist` to perform an atomic file write.
    let parent = path.parent().context(error::PathParent { path })?;
    let mut writer =
        NamedTempFile::new_in(parent).context(error::FileTempCreate { path: parent })?;
    serde_json::to_writer_pretty(&mut writer, json).context(error::FileWriteJson { path })?;
    writer.write_all(b"\n").context(error::FileWrite { path })?;
    writer.persist(path).context(error::FilePersist { path })?;
    Ok(())
}

/// Adds a key to the root role if not already present, and returns its key ID.
fn add_key(root: &mut Root, key: Key) -> Result<Decoded<Hex>> {
    // Check to see if key is already present
    for (key_id, candidate_key) in &root.keys {
        if key.eq(candidate_key) {
            return Ok(key_id.clone());
        }
    }

    // Key isn't present yet, so we need to add it
    let key_id = key.key_id().context(error::KeyId)?;
    ensure!(
        !root.keys.contains_key(&key_id),
        error::KeyDuplicate {
            key_id: hex::encode(&key_id)
        }
    );
    root.keys.insert(key_id.clone(), key);
    Ok(key_id)
}
