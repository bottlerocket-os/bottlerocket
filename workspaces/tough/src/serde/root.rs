use crate::error;
use crate::serde::decoded::{Decoded, Hex};
use crate::serde::key::Key;
use crate::serde::{Metadata, Role};
use chrono::{DateTime, Utc};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use snafu::{ensure, ResultExt};
use std::collections::BTreeMap;
use std::fmt;
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
    // An inner function that does actual key ID validation:
    // * fails if a key ID doesn't match its contents
    // * fails if there is a duplicate key ID
    fn visit_entry(
        keyid: Decoded<Hex>,
        key: Key,
        map: &mut BTreeMap<Decoded<Hex>, Key>,
    ) -> Result<(), error::Error> {
        let digest = Sha256::digest(&serde_json::to_vec(&key).context(error::JsonSerialization)?);
        ensure!(
            &keyid == digest.as_slice(),
            error::HashMismatch {
                calculated: hex::encode(digest),
                expected: hex::encode(&keyid),
            }
        );
        let keyid_hex = hex::encode(&keyid); // appease borrowck
        ensure!(
            map.insert(keyid, key).is_none(),
            error::DuplicateKeyId { keyid: keyid_hex }
        );
        Ok(())
    }

    // The rest of this is fitting the above function into serde and doing error type conversion.
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<Decoded<Hex>, Key>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some((keyid, key)) = access.next_entry()? {
                visit_entry(keyid, key, &mut map).map_err(M::Error::custom)?;
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(Visitor)
}
