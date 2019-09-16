#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod de;
pub mod decoded;
mod error;
mod iter;
pub mod key;
mod spki;
mod verify;

pub use crate::error::Error;

use crate::decoded::{Decoded, Hex};
use crate::iter::KeysIter;
use crate::key::Key;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_plain::{forward_display_to_serde, forward_from_str_to_serde};
use std::collections::HashMap;
use std::num::NonZeroU64;

/// A role type.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum RoleType {
    Root,
    Snapshot,
    Targets,
    Timestamp,
}

forward_display_to_serde!(RoleType);
forward_from_str_to_serde!(RoleType);

/// Common trait implemented by all roles.
pub trait Role {
    const TYPE: RoleType;

    fn expires(&self) -> DateTime<Utc>;
}

/// A signed metadata object.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Signed<T> {
    /// The role that is signed.
    pub signed: T,
    /// A list of signatures and their key IDs.
    pub signatures: Vec<Signature>,
}

/// A signature and the key ID that made it.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Signature {
    /// The key ID (listed in root.json) that made this signature.
    pub keyid: Decoded<Hex>,
    /// A hex-encoded signature of the canonical JSON form of a role.
    pub sig: Decoded<Hex>,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "_type")]
#[serde(rename = "root")]
pub struct Root {
    pub spec_version: String,
    pub consistent_snapshot: bool,
    pub version: NonZeroU64,
    pub expires: DateTime<Utc>,
    #[serde(deserialize_with = "de::deserialize_keys")]
    pub keys: HashMap<Decoded<Hex>, Key>,
    pub roles: HashMap<RoleType, RoleKeys>,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RoleKeys {
    pub keyids: Vec<Decoded<Hex>>,
    pub threshold: NonZeroU64,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

impl Root {
    pub fn keys(&self, role: RoleType) -> impl Iterator<Item = &Key> {
        KeysIter {
            keyids_iter: match self.roles.get(&role) {
                Some(role_keys) => role_keys.keyids.iter(),
                None => [].iter(),
            },
            keys: &self.keys,
        }
    }
}

impl Role for Root {
    const TYPE: RoleType = RoleType::Root;

    fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "_type")]
#[serde(rename = "snapshot")]
pub struct Snapshot {
    pub spec_version: String,
    pub version: NonZeroU64,
    pub expires: DateTime<Utc>,
    pub meta: HashMap<String, Meta>,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Meta {
    pub length: u64,
    pub hashes: Hashes,
    pub version: NonZeroU64,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Hashes {
    pub sha256: Decoded<Hex>,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

impl Role for Snapshot {
    const TYPE: RoleType = RoleType::Snapshot;

    fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// We do not handle delegation in this library.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "_type")]
#[serde(rename = "targets")]
pub struct Targets {
    pub spec_version: String,
    pub version: NonZeroU64,
    pub expires: DateTime<Utc>,
    pub targets: HashMap<String, Target>,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Target {
    pub length: u64,
    pub hashes: Hashes,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, Value>,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

impl Role for Targets {
    const TYPE: RoleType = RoleType::Targets;

    fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "_type")]
#[serde(rename = "timestamp")]
pub struct Timestamp {
    pub spec_version: String,
    pub version: NonZeroU64,
    pub expires: DateTime<Utc>,
    pub meta: HashMap<String, Meta>,

    /// Extra arguments found during deserialization.
    ///
    /// We must store these to correctly verify signatures for this object.
    ///
    /// If you're instantiating this struct, you should make this `HashMap::empty()`.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

impl Role for Timestamp {
    const TYPE: RoleType = RoleType::Timestamp;

    fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}
