//! In-memory datastore for use in testing other modules.
//!
//! Mimics some of the decisions made for FilesystemDataStore, e.g. metadata being committed
//! immediately.

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use super::{Committed, DataStore, Key, KeyType, Result};

#[derive(Debug)]
pub(crate) struct MemoryDataStore {
    // Uncommitted (pending) data.
    pending: HashMap<Key, String>,
    // Committed (live) data.
    live: HashMap<Key, String>,
    // Map of data keys to their metadata, which in turn is a mapping of metadata keys to
    // arbitrary (string/serialized) values.
    metadata: HashMap<Key, HashMap<Key, String>>,
}

impl MemoryDataStore {
    pub(crate) fn new() -> Self {
        Self {
            pending: HashMap::new(),
            live: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    fn dataset(&self, committed: Committed) -> &HashMap<Key, String> {
        match committed {
            Committed::Live => &self.live,
            Committed::Pending => &self.pending,
        }
    }

    fn dataset_mut(&mut self, committed: Committed) -> &mut HashMap<Key, String> {
        match committed {
            Committed::Live => &mut self.live,
            Committed::Pending => &mut self.pending,
        }
    }
}

impl DataStore for MemoryDataStore {
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: Committed,
    ) -> Result<HashSet<Key>> {
        let dataset = self.dataset(committed);
        Ok(dataset
            .keys()
            // Make sure the data keys start with the given prefix.
            .filter(|k| k.starts_with(prefix.as_ref()))
            .cloned()
            .collect())
    }

    fn get_key(&self, key: &Key, committed: Committed) -> Result<Option<String>> {
        let map_key: &str = key.borrow();
        Ok(self.dataset(committed).get(map_key).cloned())
    }

    fn set_key<S: AsRef<str>>(&mut self, key: &Key, value: S, committed: Committed) -> Result<()> {
        self.dataset_mut(committed)
            .insert(key.clone(), value.as_ref().to_owned());
        Ok(())
    }

    fn key_populated(&self, key: &Key, committed: Committed) -> Result<bool> {
        let map_key: &str = key.borrow();
        Ok(self.dataset(committed).contains_key(map_key))
    }

    fn get_metadata_raw(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>> {
        let metadata_for_data = self.metadata.get(data_key.as_ref());
        // If we have a metadata entry for this data key, then we can try fetching the requested
        // metadata key, otherwise we'll return early with Ok(None).
        let result = metadata_for_data.and_then(|m| m.get(metadata_key.as_ref()));
        Ok(result.cloned())
    }

    fn set_metadata<S: AsRef<str>>(
        &mut self,
        metadata_key: &Key,
        data_key: &Key,
        value: S,
    ) -> Result<()> {
        // If we don't already have a metadata entry for this data key, insert one.
        let metadata_for_data = self
            .metadata
            // Clone data key because we want the HashMap key type to be Key, not &Key, and we
            // can't pass ownership because we only have a reference from our parameters.
            .entry(data_key.clone())
            .or_insert_with(HashMap::new);

        metadata_for_data.insert(metadata_key.clone(), value.as_ref().to_owned());
        Ok(())
    }

    fn commit(&mut self) -> Result<HashSet<Key>> {
        // Get data for changed keys
        let pending_data = self.get_prefix("settings.", Committed::Pending)?;

        // Turn String keys of pending data into Key keys, for return
        let try_pending_keys: Result<HashSet<Key>> = pending_data
            .keys()
            .map(|s| Key::new(KeyType::Data, s))
            .collect();
        let pending_keys = try_pending_keys?;

        // Apply changes to live
        self.set_keys(&pending_data, Committed::Live)?;

        // Remove pending
        self.pending = HashMap::new();

        Ok(pending_keys)
    }
}

#[cfg(test)]
mod test {
    use super::super::{Committed, DataStore, Key, KeyType};
    use super::MemoryDataStore;
    use maplit::hashset;

    #[test]
    fn get_set() {
        let mut m = MemoryDataStore::new();
        let k = Key::new(KeyType::Data, "memtest").unwrap();
        let v = "memvalue";
        m.set_key(&k, v, Committed::Live).unwrap();
        assert_eq!(m.get_key(&k, Committed::Live).unwrap(), Some(v.to_string()));

        let mdkey = Key::new(KeyType::Meta, "testmd").unwrap();
        let md = "mdval";
        m.set_metadata(&mdkey, &k, md).unwrap();
        assert_eq!(
            m.get_metadata_raw(&mdkey, &k).unwrap(),
            Some(md.to_string())
        );
    }

    #[test]
    fn populated() {
        let mut m = MemoryDataStore::new();
        let k1 = Key::new(KeyType::Data, "memtest1").unwrap();
        let k2 = Key::new(KeyType::Data, "memtest2").unwrap();
        let v = "memvalue";
        m.set_key(&k1, v, Committed::Live).unwrap();
        m.set_key(&k2, v, Committed::Live).unwrap();

        assert!(m.key_populated(&k1, Committed::Live).unwrap());
        assert!(m.key_populated(&k2, Committed::Live).unwrap());
        assert_eq!(
            m.list_populated_keys("", Committed::Live).unwrap(),
            hashset!(k1, k2),
        );

        let bad_key = Key::new(KeyType::Data, "memtest3").unwrap();
        assert!(!m.key_populated(&bad_key, Committed::Live).unwrap());
    }

    #[test]
    fn commit() {
        let mut m = MemoryDataStore::new();
        let k = Key::new(KeyType::Data, "settings.a.b.c").unwrap();
        let v = "memvalue";
        m.set_key(&k, v, Committed::Pending).unwrap();

        assert!(m.key_populated(&k, Committed::Pending).unwrap());
        assert!(!m.key_populated(&k, Committed::Live).unwrap());
        m.commit().unwrap();
        assert!(!m.key_populated(&k, Committed::Pending).unwrap());
        assert!(m.key_populated(&k, Committed::Live).unwrap());
    }
}
