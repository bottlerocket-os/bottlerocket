//! In-memory datastore for use in testing other modules.
//!
//! Mimics some of the decisions made for FilesystemDataStore, e.g. metadata being committed
//! immediately.

use super::{Committed, DataStore, DataStoreError, Key, KeyType, Result};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct MemoryDataStore {
    pending: HashMap<String, String>,
    live: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

impl MemoryDataStore {
    pub(crate) fn new() -> Self {
        Self {
            pending: HashMap::new(),
            live: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    fn dataset(&self, committed: Committed) -> &HashMap<String, String> {
        match committed {
            Committed::Live => &self.live,
            Committed::Pending => &self.pending,
        }
    }

    fn dataset_mut(&mut self, committed: Committed) -> &mut HashMap<String, String> {
        match committed {
            Committed::Live => &mut self.live,
            Committed::Pending => &mut self.pending,
        }
    }
}

impl MemoryDataStore {
    fn metadata_map_key(&self, metadata_key: &Key, data_key: &Key) -> String {
        data_key.to_string() + "/" + metadata_key
    }
}

impl DataStore for MemoryDataStore {
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: Committed,
    ) -> Result<HashSet<Key>> {
        let dataset = self.dataset(committed);
        let key_strs = dataset.keys().filter(|k| k.starts_with(prefix.as_ref()));
        let keys = key_strs.map(|s| {
            Key::new(KeyType::Data, s).expect(&format!(
                "Failed to make Key from key already in datastore: {}",
                s
            ))
        });
        Ok(keys.collect())
    }

    fn get_key(&self, key: &Key, committed: Committed) -> Result<Option<String>> {
        let map_key: &str = key.borrow();
        Ok(self.dataset(committed).get(map_key).cloned())
    }

    fn set_key<S: AsRef<str>>(&mut self, key: &Key, value: S, committed: Committed) -> Result<()> {
        self.dataset_mut(committed)
            .insert(key.to_string(), value.as_ref().to_owned());
        Ok(())
    }

    fn key_populated(&self, key: &Key, committed: Committed) -> Result<bool> {
        let map_key: &str = key.borrow();
        Ok(self.dataset(committed).contains_key(map_key))
    }

    fn get_metadata(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>> {
        let map_key = self.metadata_map_key(metadata_key, data_key);
        Ok(self.metadata.get(&map_key).cloned())
    }

    fn set_metadata<S: AsRef<str>>(
        &mut self,
        metadata_key: &Key,
        data_key: &Key,
        value: S,
    ) -> Result<()> {
        let map_key = self.metadata_map_key(metadata_key, data_key);
        self.metadata.insert(map_key, value.as_ref().to_owned());
        Ok(())
    }

    fn commit(&mut self) -> Result<HashSet<Key>> {
        // Find keys that have been changed
        let pending_keys = self.list_populated_keys("settings.", Committed::Pending)?;

        let mut pending = HashMap::new();
        for key_str in pending_keys.iter() {
            // We just listed keys, so the keys should be valid and data should exist.
            let key = Key::new(KeyType::Data, key_str)?;
            let data = self
                .get_key(&key, Committed::Pending)?
                .ok_or(DataStoreError::Internal(format!(
                    "Listed key not found on disk: {}",
                    key
                )))?;
            pending.insert(key_str, data);
        }

        // Apply changes to live
        self.set_keys(&pending, Committed::Live)?;

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
        assert_eq!(m.get_metadata(&mdkey, &k).unwrap(), Some(md.to_string()));
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
