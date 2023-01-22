//! In-memory datastore for use in testing other modules.
//!
//! Mimics some of the decisions made for FilesystemDataStore, e.g. metadata being committed
//! immediately.

use std::collections::{HashMap, HashSet};

use super::{Committed, DataStore, Key, Result};

#[derive(Debug, Default)]
pub struct MemoryDataStore {
    // Transaction name -> (key -> data)
    pending: HashMap<String, HashMap<Key, String>>,
    // Committed (live) data.
    live: HashMap<Key, String>,
    // Map of data keys to their metadata, which in turn is a mapping of metadata keys to
    // arbitrary (string/serialized) values.
    metadata: HashMap<Key, HashMap<Key, String>>,
}

impl MemoryDataStore {
    pub fn new() -> Self {
        Default::default()
    }

    fn dataset(&self, committed: &Committed) -> Option<&HashMap<Key, String>> {
        match committed {
            Committed::Live => Some(&self.live),
            Committed::Pending { tx } => self.pending.get(tx),
        }
    }

    fn dataset_mut(&mut self, committed: &Committed) -> &mut HashMap<Key, String> {
        match committed {
            Committed::Live => &mut self.live,
            Committed::Pending { tx } => self.pending.entry(tx.clone()).or_default(),
        }
    }
}

impl DataStore for MemoryDataStore {
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: &Committed,
    ) -> Result<HashSet<Key>> {
        let empty = HashMap::new();
        let dataset = self.dataset(committed).unwrap_or(&empty);
        Ok(dataset
            .keys()
            // Make sure the data keys start with the given prefix.
            .filter(|k| k.name().starts_with(prefix.as_ref()))
            .cloned()
            .collect())
    }

    fn list_populated_metadata<S1, S2>(
        &self,
        prefix: S1,
        metadata_key_name: &Option<S2>,
    ) -> Result<HashMap<Key, HashSet<Key>>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let mut result = HashMap::new();
        for (data_key, meta_map) in self.metadata.iter() {
            // Confirm data key matches requested prefix.
            if !data_key.name().starts_with(prefix.as_ref()) {
                continue;
            }

            let mut meta_for_data = HashSet::new();
            for meta_key in meta_map.keys() {
                // Confirm metadata key matches requested name, if any.
                if let Some(name) = metadata_key_name {
                    if name.as_ref() != meta_key.name() {
                        continue;
                    }
                }
                meta_for_data.insert(meta_key.clone());
            }
            // Only add an entry for the data key if we found metadata.
            if !meta_for_data.is_empty() {
                result.insert(data_key.clone(), meta_for_data);
            }
        }

        Ok(result)
    }

    fn get_key(&self, key: &Key, committed: &Committed) -> Result<Option<String>> {
        let empty = HashMap::new();
        let dataset = self.dataset(committed).unwrap_or(&empty);
        Ok(dataset.get(key).cloned())
    }

    fn set_key<S: AsRef<str>>(&mut self, key: &Key, value: S, committed: &Committed) -> Result<()> {
        self.dataset_mut(committed)
            .insert(key.clone(), value.as_ref().to_owned());
        Ok(())
    }

    fn unset_key(&mut self, key: &Key, committed: &Committed) -> Result<()> {
        self.dataset_mut(committed).remove(key);
        Ok(())
    }

    fn key_populated(&self, key: &Key, committed: &Committed) -> Result<bool> {
        let empty = HashMap::new();
        let dataset = self.dataset(committed).unwrap_or(&empty);
        Ok(dataset.contains_key(key))
    }

    fn get_metadata_raw(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>> {
        let metadata_for_data = self.metadata.get(data_key);
        // If we have a metadata entry for this data key, then we can try fetching the requested
        // metadata key, otherwise we'll return early with Ok(None).
        let result = metadata_for_data.and_then(|m| m.get(metadata_key));
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

    fn unset_metadata(&mut self, metadata_key: &Key, data_key: &Key) -> Result<()> {
        // If we have any metadata for this data key, remove the given metadata key.
        if let Some(metadata_for_data) = self.metadata.get_mut(data_key) {
            metadata_for_data.remove(metadata_key);
        }
        Ok(())
    }

    fn commit_transaction<S>(&mut self, transaction: S) -> Result<HashSet<Key>>
    where
        S: Into<String> + AsRef<str>,
    {
        // Remove anything pending for this transaction
        if let Some(pending) = self.pending.remove(transaction.as_ref()) {
            // Apply pending changes to live
            self.set_keys(&pending, &Committed::Live)?;
            // Return keys that were committed
            Ok(pending.keys().cloned().collect())
        } else {
            Ok(HashSet::new())
        }
    }

    fn delete_transaction<S>(&mut self, transaction: S) -> Result<HashSet<Key>>
    where
        S: Into<String> + AsRef<str>,
    {
        // Remove anything pending for this transaction
        if let Some(pending) = self.pending.remove(transaction.as_ref()) {
            // Return the old pending keys
            Ok(pending.keys().cloned().collect())
        } else {
            Ok(HashSet::new())
        }
    }

    fn list_transactions(&self) -> Result<HashSet<String>> {
        Ok(self.pending.keys().cloned().collect())
    }
}

#[cfg(test)]
mod test {
    use super::super::{Committed, DataStore, Key, KeyType};
    use super::MemoryDataStore;
    use maplit::hashset;

    #[test]
    fn get_set_unset() {
        let mut m = MemoryDataStore::new();
        let k = Key::new(KeyType::Data, "memtest").unwrap();
        let v = "memvalue";
        m.set_key(&k, v, &Committed::Live).unwrap();
        assert_eq!(
            m.get_key(&k, &Committed::Live).unwrap(),
            Some(v.to_string())
        );

        let mdkey = Key::new(KeyType::Meta, "testmd").unwrap();
        let md = "mdval";
        m.set_metadata(&mdkey, &k, md).unwrap();
        assert_eq!(
            m.get_metadata_raw(&mdkey, &k).unwrap(),
            Some(md.to_string())
        );

        m.unset_metadata(&mdkey, &k).unwrap();
        assert_eq!(m.get_metadata_raw(&mdkey, &k).unwrap(), None);

        m.unset_key(&k, &Committed::Live).unwrap();
        assert_eq!(m.get_key(&k, &Committed::Live).unwrap(), None);
    }

    #[test]
    fn populated() {
        let mut m = MemoryDataStore::new();
        let k1 = Key::new(KeyType::Data, "memtest1").unwrap();
        let k2 = Key::new(KeyType::Data, "memtest2").unwrap();
        let v = "memvalue";
        m.set_key(&k1, v, &Committed::Live).unwrap();
        m.set_key(&k2, v, &Committed::Live).unwrap();

        assert!(m.key_populated(&k1, &Committed::Live).unwrap());
        assert!(m.key_populated(&k2, &Committed::Live).unwrap());
        assert_eq!(
            m.list_populated_keys("", &Committed::Live).unwrap(),
            hashset!(k1, k2),
        );

        let bad_key = Key::new(KeyType::Data, "memtest3").unwrap();
        assert!(!m.key_populated(&bad_key, &Committed::Live).unwrap());
    }

    #[test]
    fn commit() {
        let mut m = MemoryDataStore::new();
        let k = Key::new(KeyType::Data, "settings.a.b.c").unwrap();
        let v = "memvalue";
        let tx = "test transaction";
        let pending = Committed::Pending { tx: tx.into() };
        m.set_key(&k, v, &pending).unwrap();

        assert!(m.key_populated(&k, &pending).unwrap());
        assert!(!m.key_populated(&k, &Committed::Live).unwrap());
        m.commit_transaction(tx).unwrap();
        assert!(!m.key_populated(&k, &pending).unwrap());
        assert!(m.key_populated(&k, &Committed::Live).unwrap());
    }

    #[test]
    fn delete_transaction() {
        let mut m = MemoryDataStore::new();
        let k = Key::new(KeyType::Data, "settings.a.b.c").unwrap();
        let v = "memvalue";
        let tx = "test transaction";
        let pending = Committed::Pending { tx: tx.into() };
        m.set_key(&k, v, &pending).unwrap();

        // Set something in a different transaction to ensure it doesn't get deleted
        let k2 = Key::new(KeyType::Data, "settings.x.y.z").unwrap();
        let v2 = "memvalue 2";
        let tx2 = "test transaction 2";
        let pending2 = Committed::Pending { tx: tx2.into() };
        m.set_key(&k2, v2, &pending2).unwrap();

        assert!(m.key_populated(&k, &pending).unwrap());
        assert!(!m.key_populated(&k, &Committed::Live).unwrap());
        m.delete_transaction(tx).unwrap();
        assert!(!m.key_populated(&k, &pending).unwrap());
        assert!(!m.key_populated(&k, &Committed::Live).unwrap());

        // Assure other transactions were not deleted
        assert!(m.key_populated(&k2, &pending2).unwrap());
    }
}
