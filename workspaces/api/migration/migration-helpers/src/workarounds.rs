use crate::args::Args;
use crate::{error, MigrationData, Result};
use apiserver::datastore::{Committed, DataStore, Key, KeyType};
use snafu::ResultExt;
use std::collections::HashSet;

/// Here we can fix known issues with migrated data, for example issues related to changes
/// in migration interface that we don't want the migrations to have to deal with.
pub(crate) fn fix_migrated_data<D: DataStore>(
    input: &MigrationData,
    output: &MigrationData,
    _source_datastore: &D,
    target_datastore: &mut D,
    committed: Committed,
    args: &Args,
) -> Result<()> {
    // If the source and target data store path are the same, we're using the old migrator
    // interface, and have to use a workaround to be able to delete keys.  They can't just be
    // removed from the MigrationData struct, because the old interface used the same data store
    // for input and output, and removing from MigrationData just means we won't write it out
    // again - but the file will still be there from the input.  We need to tell the data store
    // to remove it.
    if args.source_datastore == args.target_datastore {
        // Data keys first
        let old_keys: HashSet<_> = input.data.keys().collect();
        let new_keys: HashSet<_> = output.data.keys().collect();
        for removed_key_str in old_keys.difference(&new_keys) {
            // We need to make a Key from the key's name to fit the data store interface; we
            // don't use Key in MigrationData for the convenience of migration authors.
            let removed_key =
                Key::new(KeyType::Data, removed_key_str).context(error::InvalidKey {
                    key_type: KeyType::Data,
                    key: *removed_key_str,
                })?;
            target_datastore
                .unset_key(&removed_key, committed)
                .context(error::DataStoreRemove {
                    key: *removed_key_str,
                })?;
        }

        // Now the same thing for metadata
        for (data_key_str, old_metadata) in &input.metadata {
            let removed: HashSet<_> = if let Some(new_metadata) = output.metadata.get(data_key_str)
            {
                // Find which metadata keys the migration removed, for this data key
                let old_keys: HashSet<_> = old_metadata.keys().collect();
                let new_keys: HashSet<_> = new_metadata.keys().collect();
                old_keys.difference(&new_keys).map(|&s| s).collect()
            } else {
                // Migration output has no metadata for this data key, so it was all removed
                old_metadata.keys().collect()
            };

            for removed_meta_str in removed {
                let removed_meta =
                    Key::new(KeyType::Meta, removed_meta_str).context(error::InvalidKey {
                        key_type: KeyType::Meta,
                        key: removed_meta_str,
                    })?;
                let removed_data =
                    Key::new(KeyType::Data, data_key_str).context(error::InvalidKey {
                        key_type: KeyType::Data,
                        key: data_key_str,
                    })?;
                target_datastore
                    .unset_metadata(&removed_meta, &removed_data)
                    .context(error::DataStoreRemove {
                        key: removed_meta_str,
                    })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::fix_migrated_data;
    use crate::datastore::set_output_data;
    use crate::{Args, MigrationData, MigrationType};
    use apiserver::datastore::memory::MemoryDataStore;
    use apiserver::datastore::{Committed, DataStore, Key, KeyType};
    use maplit::hashmap;
    use serde_json::json;

    #[test]
    fn test_fix_migrated_data() {
        // Data/metadata starting with "remove" should be removed
        let input = MigrationData {
            data: hashmap!(
              "keepdata".into() => json!("hi"),
              "removedata".into() => json!("sup"),
            ),
            metadata: hashmap!(
                "keepdata".into() => hashmap!(
                    "keepmeta".into() => json!("howdy"),
                    "removemeta".into() => json!("yo"),
                ),
                "removedata".into() => hashmap!(
                    "keepmeta".into() => json!("hello"),
                    "removemeta".into() => json!("hiya"),
                ),
            ),
        };
        // This represents 'input' after a migration removes some data, so it should match the
        // data store after we call fix_migrated_data
        let expected = MigrationData {
            data: hashmap!(
              "keepdata".into() => json!("hi"),
            ),
            metadata: hashmap!(
                "keepdata".into() => hashmap!(
                    "keepmeta".into() => json!("howdy"),
                ),
                "removedata".into() => hashmap!(
                    "keepmeta".into() => json!("hello"),
                ),
            ),
        };

        // The point of the workaround is affecting the data store directly, so make test stores
        let mut source = MemoryDataStore::new();
        set_output_data(&mut source, &input, Committed::Live).unwrap();
        // To replicate old interface, the target data store starts with the input data, and
        // we're going to confirm that removed values are actually removed
        let mut target = MemoryDataStore::new();
        set_output_data(&mut target, &input, Committed::Live).unwrap();

        // Ensure values are there at the start
        let kept_data = Key::new(KeyType::Data, "keepdata").unwrap();
        let removed_data = Key::new(KeyType::Data, "removedata").unwrap();
        let kept_meta = Key::new(KeyType::Meta, "keepmeta").unwrap();
        let removed_meta = Key::new(KeyType::Meta, "removemeta").unwrap();
        assert_eq!(target.get_key(&kept_data, Committed::Live).unwrap(), Some("\"hi\"".into()));
        assert_eq!(target.get_key(&removed_data, Committed::Live).unwrap(), Some("\"sup\"".into()));
        assert_eq!(target.get_metadata(&kept_meta, &kept_data).unwrap(), Some("\"howdy\"".into()));
        assert_eq!(target.get_metadata(&kept_meta, &removed_data).unwrap(), Some("\"hello\"".into()));
        assert_eq!(target.get_metadata(&removed_meta, &kept_data).unwrap(), Some("\"yo\"".into()));
        assert_eq!(target.get_metadata(&removed_meta, &removed_data).unwrap(), Some("\"hiya\"".into()));

        // Same source and target, i.e. using old interface, so we should do our fix
        let args = Args {
            source_datastore: "same".into(),
            target_datastore: "same".into(),
            migration_type: MigrationType::Forward,
        };
        fix_migrated_data(
            &input,
            &expected,
            &source,
            &mut target,
            Committed::Live,
            &args,
        )
        .unwrap();

        // Ensure unaffected values were kept
        assert_eq!(target.get_key(&kept_data, Committed::Live).unwrap(), Some("\"hi\"".into()));
        assert_eq!(target.get_metadata(&kept_meta, &kept_data).unwrap(), Some("\"howdy\"".into()));
        assert_eq!(target.get_metadata(&kept_meta, &removed_data).unwrap(), Some("\"hello\"".into()));
        // Ensure removed values were removed
        assert_eq!(target.get_key(&removed_data, Committed::Live).unwrap(), None);
        assert_eq!(target.get_metadata(&removed_meta, &kept_data).unwrap(), None);
        assert_eq!(target.get_metadata(&removed_meta, &removed_data).unwrap(), None);
    }
}
