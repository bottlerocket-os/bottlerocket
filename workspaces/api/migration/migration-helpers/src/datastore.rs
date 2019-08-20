//! This module contains the functions that interact with the data store, retrieving data to
//! migrate and writing back migrated data.

use snafu::ResultExt;
use std::collections::HashMap;

use crate::{error, MigrationData, Result};
use apiserver::datastore::{deserialize_scalar, serialize_scalar, Committed, DataStore};

// To get input data from the existing data store, we use datastore methods, because we assume
// breaking changes in the basic data store API would be a major-version migration of the data
// store, and that would be handled separately.  This method is private to the crate, so we can
// reconsider as needed.
/// Retrieves data from the specified data store in a consistent format for easy modification.
pub(crate) fn get_input_data<D: DataStore>(
    datastore: &D,
    committed: Committed,
) -> Result<MigrationData> {
    let raw_data = datastore
        .get_prefix("", committed)
        .context(error::GetData { committed })?;

    // Deserialize values to Value so there's a consistent input type.  (We can't specify item
    // types because we'd have to know the model structure.)
    let mut data = HashMap::new();
    for (data_key, value) in raw_data.into_iter() {
        let value = deserialize_scalar(&value).context(error::Deserialize { input: value })?;
        data.insert(data_key, value);
    }

    // Metadata isn't committed, it goes live immediately, so we only populate the metadata
    // output for Committed::Live.
    let mut metadata = HashMap::new();
    if let Committed::Live = committed {
        let raw_metadata = datastore
            .get_metadata_prefix("", &None as &Option<&str>)
            .context(error::GetMetadata)?;
        for (data_key, meta_map) in raw_metadata.into_iter() {
            let data_entry = metadata.entry(data_key).or_insert_with(HashMap::new);
            for (metadata_key, value) in meta_map.into_iter() {
                let value =
                    deserialize_scalar(&value).context(error::Deserialize { input: value })?;
                data_entry.insert(metadata_key, value);
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
    committed: Committed,
) -> Result<()> {
    // Prepare serialized data
    let mut data = HashMap::new();
    for (data_key, raw_value) in &input.data {
        let value = serialize_scalar(raw_value).context(error::Serialize)?;
        data.insert(data_key, value);
    }

    // This is one of the rare cases where we want to set keys directly in the datastore:
    // * We're operating on a temporary copy of the datastore, so no concurrency issues
    // * We're either about to reboot or just have, and the settings applier will run afterward
    datastore
        .set_keys(&data, committed)
        .context(error::DataStoreWrite)?;

    // Set metadata in a loop (currently no batch API)
    for (data_key, meta_map) in &input.metadata {
        for (metadata_key, raw_value) in meta_map.into_iter() {
            let value = serialize_scalar(&raw_value).context(error::Serialize)?;
            datastore
                .set_metadata(&metadata_key, &data_key, value)
                .context(error::DataStoreWrite)?;
        }
    }

    Ok(())
}
