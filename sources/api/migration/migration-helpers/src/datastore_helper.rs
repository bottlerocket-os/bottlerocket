//! This module contains the functions that interact with the data store, retrieving data to
//! migrate and writing back migrated data.

use bottlerocket_release::BottlerocketRelease;
use snafu::ResultExt;
use std::collections::HashMap;

use crate::{error, MigrationData, Result};
use datastore::{
    deserialize_scalar, serialization::to_pairs_with_prefix, serialize_scalar, Committed,
    DataStore, Key, KeyType,
};

// To get input data from the existing data store, we use datastore methods, because we assume
// breaking changes in the basic data store API would be a major-version migration of the data
// store, and that would be handled separately.  This method is private to the crate, so we can
// reconsider as needed.
/// Retrieves data from the specified data store in a consistent format for easy modification.
pub(crate) fn get_input_data<D: DataStore>(
    datastore: &D,
    committed: &Committed,
) -> Result<MigrationData> {
    let raw_data = datastore
        .get_prefix("", committed)
        .with_context(|_| error::GetDataSnafu {
            committed: committed.clone(),
        })?;

    let mut data = HashMap::new();
    for (data_key, value_str) in raw_data.into_iter() {
        // Store keys with just their name, rather than the full Key, so that migrations are easier
        // to write, and we don't tie migrations to any specific data store version.  Migrations
        // shouldn't need to link against data store code.
        let key_name = data_key.name();
        // Deserialize values to Value so there's a consistent input type.  (We can't specify item
        // types because we'd have to know the model structure.)
        let value =
            deserialize_scalar(&value_str).context(error::DeserializeSnafu { input: value_str })?;
        data.insert(key_name.clone(), value);
    }

    // We also want to make "os.*" values, like variant and arch, available to migrations.
    let release = BottlerocketRelease::new().context(error::BottlerocketReleaseSnafu)?;
    let os_pairs = to_pairs_with_prefix("os", &release).context(error::SerializeReleaseSnafu)?;
    for (data_key, value_str) in os_pairs.into_iter() {
        let value =
            deserialize_scalar(&value_str).context(error::DeserializeSnafu { input: value_str })?;
        data.insert(data_key.name().clone(), value);
    }

    // Metadata isn't committed, it goes live immediately, so we only populate the metadata
    // output for Committed::Live.
    let mut metadata = HashMap::new();
    if let Committed::Live = committed {
        let raw_metadata = datastore
            .get_metadata_prefix("", &None as &Option<&str>)
            .context(error::GetMetadataSnafu)?;
        for (data_key, meta_map) in raw_metadata.into_iter() {
            // See notes above about storing key Strings and Values.
            let data_key_name = data_key.name();
            let data_entry = metadata
                .entry(data_key_name.clone())
                .or_insert_with(HashMap::new);
            for (metadata_key, value_str) in meta_map.into_iter() {
                let metadata_key_name = metadata_key.name();
                let value = deserialize_scalar(&value_str)
                    .context(error::DeserializeSnafu { input: value_str })?;
                data_entry.insert(metadata_key_name.clone(), value);
            }
        }
    }

    Ok(MigrationData { data, metadata })
}

// Similar to get_input_data, we use datastore methods here; please read the comment on
// get_input_data.  This method is also private to the crate, so we can reconsider as needed.
/// Updates the given data store with the given (migrated) data.
pub(crate) fn set_output_data<D: DataStore>(
    datastore: &mut D,
    input: &MigrationData,
    committed: &Committed,
) -> Result<()> {
    // Prepare serialized data
    let mut data = HashMap::new();
    for (data_key_name, raw_value) in &input.data {
        // See notes above about storing key Strings and Values.
        let data_key = Key::new(KeyType::Data, data_key_name).context(error::InvalidKeySnafu {
            key_type: KeyType::Data,
            key: data_key_name,
        })?;
        let value = serialize_scalar(raw_value).context(error::SerializeSnafu)?;
        data.insert(data_key, value);
    }

    // This is one of the rare cases where we want to set keys directly in the datastore:
    // * We're operating on a temporary copy of the datastore, so no concurrency issues
    // * We're either about to reboot or just have, and the settings applier will run afterward
    datastore
        .set_keys(&data, committed)
        .context(error::DataStoreWriteSnafu)?;

    // Set metadata in a loop (currently no batch API)
    for (data_key_name, meta_map) in &input.metadata {
        let data_key = Key::new(KeyType::Data, data_key_name).context(error::InvalidKeySnafu {
            key_type: KeyType::Data,
            key: data_key_name,
        })?;
        for (metadata_key_name, raw_value) in meta_map.iter() {
            let metadata_key =
                Key::new(KeyType::Meta, metadata_key_name).context(error::InvalidKeySnafu {
                    key_type: KeyType::Meta,
                    key: metadata_key_name,
                })?;
            let value = serialize_scalar(&raw_value).context(error::SerializeSnafu)?;
            datastore
                .set_metadata(&metadata_key, &data_key, value)
                .context(error::DataStoreWriteSnafu)?;
        }
    }

    Ok(())
}
