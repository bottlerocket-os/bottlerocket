use crate::error;
use crate::serde::decoded::{Decoded, Hex};
use crate::serde::key::Key;
use crate::serde::{Metadata, Role, Signed};
use chrono::{DateTime, Utc};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU64;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "_type")]
#[serde(rename = "root")]
pub(crate) struct Root {
    // Field ordering must be alphabetical so that it is sorted for Canonical JSON.
    pub(crate) consistent_snapshot: bool,
    pub(crate) expires: DateTime<Utc>,
    // BTreeMaps are used on purpose, because we re-serialize these fields as Canonical JSON to
    // verify the signature.
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

    pub(crate) fn verify_role<T: Metadata + Serialize>(
        &self,
        role: &Signed<T>,
    ) -> error::Result<()> {
        let role_keys = self
            .roles
            .get(&T::ROLE)
            .context(error::MissingRole { role: T::ROLE })?;
        let mut valid = 0;

        // TODO(iliana): actually implement Canonical JSON instead of just hoping that what we get
        // out of serde_json is Canonical JSON
        let data = serde_json::to_vec(&role.signed).context(error::JsonSerialization {
            what: format!("{} role", T::ROLE),
        })?;

        for signature in &role.signatures {
            if role_keys.keyids.contains(&signature.keyid) {
                if let Some(key) = self.keys.get(&signature.keyid) {
                    if key.verify(&data, &signature.sig) {
                        valid += 1;
                    }
                }
            }
        }

        ensure!(
            valid >= u64::from(role_keys.threshold),
            error::SignatureThreshold {
                role: T::ROLE,
                threshold: role_keys.threshold,
                valid,
            }
        );
        Ok(())
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
    // If this passes we insert the entry.
    fn validate_and_insert_entry(
        keyid: Decoded<Hex>,
        key: Key,
        map: &mut BTreeMap<Decoded<Hex>, Key>,
    ) -> Result<(), error::Error> {
        let digest =
            Sha256::digest(&serde_json::to_vec(&key).context(error::JsonSerialization {
                what: format!("key {}", keyid),
            })?);
        ensure!(
            &keyid == digest.as_slice(),
            error::HashMismatch {
                context: format!("key {}", keyid),
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
                validate_and_insert_entry(keyid, key, &mut map).map_err(M::Error::custom)?;
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(Visitor)
}

#[cfg(test)]
mod tests {
    use crate::serde::{root::Root, Signed};

    #[test]
    fn simple_rsa() {
        let root: Signed<Root> =
            serde_json::from_str(include_str!("../../tests/data/simple-rsa/root.json")).unwrap();
        root.signed.verify_role(&root).unwrap();
    }

    #[test]
    fn duplicate_keyid() {
        assert!(serde_json::from_str::<Signed<Root>>(include_str!(
            "../../tests/data/duplicate-keyid/root.json"
        ))
        .is_err());
    }

    #[test]
    fn no_root_json_signatures_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/no-root-json-signatures/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("missing signature should not verify");
    }

    #[test]
    fn invalid_root_json_signatures_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/invalid-root-json-signature/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("invalid (unauthentic) root signature should not verify");
    }

    #[test]
    fn expired_root_json_signature_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/expired-root-json-signature/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("expired root signature should not verify");
    }

    #[test]
    fn mismatched_root_json_keyids_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/mismatched-root-json-keyids/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("mismatched root role keyids (provided and signed) should not verify");
    }
}
