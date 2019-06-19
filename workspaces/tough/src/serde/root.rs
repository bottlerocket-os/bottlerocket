use crate::error;
use crate::serde::decoded::{Decoded, Hex};
use crate::serde::key::Key;
use crate::serde::{Metadata, Role};
use chrono::{DateTime, Utc};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use snafu::ResultExt;
use std::collections::BTreeMap;
use std::num::NonZeroU64;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "_type")]
#[serde(rename = "root")]
pub(crate) struct Root {
    pub(crate) consistent_snapshot: bool,
    pub(crate) expires: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_keys")]
    pub(crate) keys: BTreeMap<Decoded<Hex>, Key>,
    pub(crate) roles: BTreeMap<Role, RoleKeys>,
    pub(crate) spec_version: String,
    pub(crate) version: NonZeroU64,
}

impl Root {
    pub(crate) fn keys(&self, role: Role) -> Vec<Key> {
        let keyids = match self.roles.get(&role) {
            Some(role_keys) => &role_keys.keyids,
            None => return Vec::new(),
        };
        keyids
            .iter()
            .filter_map(|keyid| self.keys.get(keyid).cloned())
            .collect()
    }
}

impl Metadata for Root {
    const ROLE: Role = Role::Root;

    fn expires(&self) -> &DateTime<Utc> {
        &self.expires
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RoleKeys {
    pub(crate) keyids: Vec<Decoded<Hex>>,
    pub(crate) threshold: NonZeroU64,
}

/// Validates the key ID for each key during deserialization and fails if any don't match.
fn deserialize_keys<'de, D>(deserializer: D) -> Result<BTreeMap<Decoded<Hex>, Key>, D::Error>
where
    D: Deserializer<'de>,
{
    let keys: BTreeMap<Decoded<Hex>, Key> = BTreeMap::deserialize(deserializer)?;
    for (keyid, key) in &keys {
        let digest = Sha256::digest(
            &serde_json::to_vec(key)
                .context(error::JsonSerialization)
                .map_err(D::Error::custom)?,
        );
        if keyid != digest.as_slice() {
            error::HashMismatch {
                calculated: hex::encode(digest),
                expected: hex::encode(keyid),
            }
            .fail()
            .map_err(D::Error::custom)?;
        }
    }
    Ok(keys)
}
