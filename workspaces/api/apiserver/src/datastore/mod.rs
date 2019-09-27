//! The datastore module contains the DataStore trait, which describes a key/value storage system
//! with metadata and simple transactions.
//!
//! There's also a common error type and some methods that implementations of DataStore should
//! generally share, like scalar serialization.
//!
//! We represent scalars -- the actual values stored under a datastore key -- using JSON, just to
//! have a convenient human-readable form.  (TOML doesn't allow raw scalars.  The JSON spec
//! doesn't seem to either, but this works, and the format is so simple for scalars that it could
//! be easily swapped out if needed.)

pub mod deserialization;
pub mod error;
pub mod filesystem;
pub mod key;
#[cfg(test)]
pub(crate) mod memory;
pub mod serialization;

pub use error::{Error, Result};
pub use filesystem::FilesystemDataStore;
pub use key::{Key, KeyType, KEY_SEPARATOR};

use serde::{Deserialize, Serialize};
use snafu::OptionExt;
use std::collections::{HashMap, HashSet};

/// Committed represents whether we want to look at pending (uncommitted) or live (committed) data
/// in the datastore.
#[derive(Debug, Copy, Clone)]
pub enum Committed {
    Pending,
    Live,
}

pub trait DataStore {
    /// Returns whether a key is present (has a value) in the datastore.
    fn key_populated(&self, key: &Key, committed: Committed) -> Result<bool>;
    /// Returns a list of the populated data keys in the datastore whose names start with the given
    /// prefix.
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: Committed,
    ) -> Result<HashSet<Key>>;
    /// Finds all metadata keys that are currently populated in the datastore whose data keys
    /// start with the given prefix.  If you specify metadata_key_name, only metadata keys with
    /// that name will be returned.
    ///
    /// Returns a mapping of the data keys to the set of populated metadata keys for each.
    fn list_populated_metadata<S1, S2>(
        &self,
        prefix: S1,
        metadata_key_name: &Option<S2>,
    ) -> Result<HashMap<Key, HashSet<Key>>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>;

    /// Retrieve the value for a single data key from the datastore.
    fn get_key(&self, key: &Key, committed: Committed) -> Result<Option<String>>;
    /// Set the value of a single data key in the datastore.
    fn set_key<S: AsRef<str>>(&mut self, key: &Key, value: S, committed: Committed) -> Result<()>;

    /// Retrieve the value for a single metadata key from the datastore.  Values will inherit from
    /// earlier in the tree, if more specific values are not found later.
    fn get_metadata(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>> {
        let mut result = Ok(None);
        let mut current_path = String::with_capacity(data_key.len());

        // Walk through segments of the data key in order, returning the last metadata we find
        for component in data_key.split(KEY_SEPARATOR) {
            if !current_path.is_empty() {
                current_path.push_str(KEY_SEPARATOR);
            }
            current_path.push_str(component);

            let data_key = Key::new(KeyType::Data, &current_path).unwrap_or_else(|_| {
                unreachable!("Prefix of Key failed to make Key: {}", current_path)
            });

            if let Some(md) = self.get_metadata_raw(metadata_key, &data_key)? {
                result = Ok(Some(md));
            }
        }
        result
    }

    /// Retrieve the value for a single metadata key from the datastore, without taking into
    /// account inheritance of metadata from earlier in the tree.
    fn get_metadata_raw(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>>;
    /// Set the value of a single metadata key in the datastore.
    fn set_metadata<S: AsRef<str>>(
        &mut self,
        metadata_key: &Key,
        data_key: &Key,
        value: S,
    ) -> Result<()>;

    /// Applies pending changes to the live datastore.  Returns the list of changed keys.
    fn commit(&mut self) -> Result<HashSet<Key>>;

    /// Set multiple data keys at once in the data store.
    ///
    /// Implementers can replace the default implementation if there's a faster way than setting
    /// each key individually.
    fn set_keys<S1, S2>(&mut self, pairs: &HashMap<S1, S2>, committed: Committed) -> Result<()>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        for (key_str, value) in pairs {
            trace!("Setting data key {}", key_str.as_ref());
            let key = Key::new(KeyType::Data, key_str)?;
            self.set_key(&key, value, committed)?;
        }
        Ok(())
    }

    /// Retrieves all keys starting with the given prefix, returning them in a Key -> value map.
    ///
    /// Can be followed up by a deserialize::from_map call to build a structure.
    fn get_prefix<S: AsRef<str>>(
        &self,
        find_prefix: S,
        committed: Committed,
    ) -> Result<HashMap<Key, String>> {
        let keys = self.list_populated_keys(&find_prefix, committed)?;
        trace!("Found populated keys: {:?}", keys);
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result = HashMap::new();
        for key in keys {
            // Already confirmed key via listing keys, so an error is more serious.
            trace!("Pulling value from datastore for key: {}", key);
            let value = self
                .get_key(&key, committed)?
                .context(error::ListedKeyNotPresent { key: key.as_ref() })?;

            result.insert(key, value);
        }
        Ok(result)
    }

    /// Retrieves all metadata for data keys starting with the given prefix.  If you specify
    /// metadata_key_name, only metadata keys with that name will be returned.  Returns a
    /// mapping of each data key to its metadata, where metadata is a mapping of metadata Key to
    /// string value.
    fn get_metadata_prefix<S1, S2>(
        &self,
        find_prefix: S1,
        metadata_key_name: &Option<S2>,
    ) -> Result<HashMap<Key, HashMap<Key, String>>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let meta_map = self.list_populated_metadata(&find_prefix, metadata_key_name)?;
        trace!("Found populated metadata: {:?}", meta_map);
        if meta_map.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result = HashMap::new();
        for (data_key, meta_keys) in meta_map {
            for meta_key in meta_keys {
                // If the user requested specific metadata, move to the next key unless it
                // matches.
                if let Some(name) = metadata_key_name {
                    if name.as_ref() != meta_key.as_ref() {
                        continue;
                    }
                }

                // Already confirmed key via listing keys, so an error is more serious.
                trace!(
                    "Pulling metadata '{}' from datastore for key: {}",
                    meta_key,
                    &data_key
                );
                let value = self.get_metadata(&meta_key, &data_key)?.context(
                    error::ListedMetaNotPresent {
                        meta_key: meta_key.as_ref(),
                        data_key: data_key.as_ref(),
                    },
                )?;

                // Insert a top-level map entry for the data key if we've found metadata.
                let data_entry = result.entry(data_key.clone()).or_insert_with(HashMap::new);

                data_entry.insert(meta_key, value);
            }
        }
        Ok(result)
    }
}

/////

// This section ties together serialization and deserialization of scalar values, so it's in the
// parent module of serialization and deserialization.

/// Concrete error type for scalar ser/de.
pub type ScalarError = serde_json::Error;

/// Serialize a given scalar value to the module-standard serialization format.
pub fn serialize_scalar<S, E>(scalar: &S) -> std::result::Result<String, E>
where
    S: Serialize,
    E: From<ScalarError>,
{
    serde_json::to_string(scalar).map_err(Into::into)
}

/// Deserialize a given scalar value from the module-standard serialization format.
pub fn deserialize_scalar<'de, D, E>(scalar: &'de str) -> std::result::Result<D, E>
where
    D: Deserialize<'de>,
    E: From<ScalarError>,
{
    serde_json::from_str(scalar).map_err(Into::into)
}

/// Serde Deserializer type matching the deserialize_scalar implementation.
type ScalarDeserializer<'de> = serde_json::Deserializer<serde_json::de::StrRead<'de>>;

/// Constructor for ScalarDeserializer.
fn deserializer_for_scalar(scalar: &str) -> ScalarDeserializer<'_> {
    serde_json::Deserializer::from_str(scalar)
}

/// Serde generic "Value" type representing a tree of deserialized values.  Should be able to hold
/// anything returned by the deserialization bits above.
pub type Value = serde_json::Value;

#[cfg(test)]
mod test {
    use super::memory::MemoryDataStore;
    use super::{Committed, DataStore, Key, KeyType};
    use maplit::hashmap;

    #[test]
    fn set_keys() {
        let mut m = MemoryDataStore::new();

        let k1 = Key::new(KeyType::Data, "memtest1").unwrap();
        let k2 = Key::new(KeyType::Data, "memtest2").unwrap();
        let v1 = "memvalue1".to_string();
        let v2 = "memvalue2".to_string();
        let data = hashmap!(
            &k1 => &v1,
            &k2 => &v2,
        );

        m.set_keys(&data, Committed::Pending).unwrap();

        assert_eq!(m.get_key(&k1, Committed::Pending).unwrap(), Some(v1));
        assert_eq!(m.get_key(&k2, Committed::Pending).unwrap(), Some(v2));
    }

    #[test]
    fn get_metadata_inheritance() {
        let mut m = MemoryDataStore::new();

        let meta = Key::new(KeyType::Meta, "mymeta").unwrap();
        let parent = Key::new(KeyType::Data, "a").unwrap();
        let grandchild = Key::new(KeyType::Data, "a.b.c").unwrap();

        // Set metadata on parent
        m.set_metadata(&meta, &parent, "value").unwrap();
        // Metadata shows up on grandchild...
        assert_eq!(
            m.get_metadata(&meta, &grandchild).unwrap(),
            Some("value".to_string())
        );
        // ...but only through inheritance, not directly.
        assert_eq!(m.get_metadata_raw(&meta, &grandchild).unwrap(), None);
    }

    #[test]
    fn get_prefix() {
        let mut m = MemoryDataStore::new();
        let data = hashmap!(
            Key::new(KeyType::Data, "x.1").unwrap() => "x1".to_string(),
            Key::new(KeyType::Data, "x.2").unwrap() => "x2".to_string(),
            Key::new(KeyType::Data, "y.3").unwrap() => "y3".to_string(),
        );
        m.set_keys(&data, Committed::Pending).unwrap();

        assert_eq!(
            m.get_prefix("x.", Committed::Pending).unwrap(),
            hashmap!(Key::new(KeyType::Data, "x.1").unwrap() => "x1".to_string(),
                     Key::new(KeyType::Data, "x.2").unwrap() => "x2".to_string())
        );
    }

    #[test]
    fn get_metadata_prefix() {
        let mut m = MemoryDataStore::new();

        // Build some data keys to which we can attach metadata; they don't actually have to be
        // set in the data store.
        let k1 = Key::new(KeyType::Data, "x.1").unwrap();
        let k2 = Key::new(KeyType::Data, "x.2").unwrap();
        let k3 = Key::new(KeyType::Data, "y.3").unwrap();

        // Set some metadata to check
        let mk1 = Key::new(KeyType::Meta, "metatest1").unwrap();
        let mk2 = Key::new(KeyType::Meta, "metatest2").unwrap();
        let mk3 = Key::new(KeyType::Meta, "metatest3").unwrap();
        m.set_metadata(&mk1, &k1, "41").unwrap();
        m.set_metadata(&mk2, &k2, "42").unwrap();
        m.set_metadata(&mk3, &k3, "43").unwrap();

        // Check all metadata
        assert_eq!(
            m.get_metadata_prefix("x.", &None as &Option<&str>).unwrap(),
            hashmap!(k1 => hashmap!(mk1 => "41".to_string()),
                     k2.clone() => hashmap!(mk2.clone() => "42".to_string()))
        );

        // Check metadata matching a given name
        assert_eq!(
            m.get_metadata_prefix("x.", &Some("metatest2")).unwrap(),
            hashmap!(k2 => hashmap!(mk2 => "42".to_string()))
        );
    }
}
