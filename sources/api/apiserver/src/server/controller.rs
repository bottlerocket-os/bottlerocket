//! The controller module maps between the datastore and the API interface, similar to the
//! controller in the MVC model.

use bottlerocket_release::BottlerocketRelease;
use serde::de::DeserializeOwned;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::server::error::{self, Result};
use actix_web::HttpResponse;
use datastore::deserialization::{from_map, from_map_with_prefix};
use datastore::serialization::to_pairs;
use datastore::{deserialize_scalar, Committed, DataStore, Key, KeyType, ScalarError, Value};
use model::{ConfigurationFiles, Services, Settings};
use num::FromPrimitive;
use std::os::unix::process::ExitStatusExt;
use thar_be_updates::error::TbuErrorStatus;

/// List the open transactions from the data store.
pub(crate) fn list_transactions<D>(datastore: &D) -> Result<HashSet<String>>
where
    D: DataStore,
{
    datastore.list_transactions().context(error::DataStore {
        op: "list_transactions",
    })
}

/// Build a Settings based on pending data in the datastore; the Settings will be empty if there
/// are no pending settings.
pub(crate) fn get_transaction<D, S>(datastore: &D, transaction: S) -> Result<Settings>
where
    D: DataStore,
    S: Into<String>,
{
    let pending = Committed::Pending {
        tx: transaction.into(),
    };
    get_prefix(datastore, &pending, "settings.", None)
        .map(|maybe_settings| maybe_settings.unwrap_or_else(Settings::default))
}

/// Deletes the transaction from the data store, removing any uncommitted settings under that
/// transaction name.
pub(crate) fn delete_transaction<D: DataStore>(
    datastore: &mut D,
    transaction: &str,
) -> Result<HashSet<Key>> {
    datastore
        .delete_transaction(transaction)
        .context(error::DataStore {
            op: "delete_pending",
        })
}

/// Build a Settings based on the data in the datastore.  Errors if no settings are found.
pub(crate) fn get_settings<D: DataStore>(datastore: &D, committed: &Committed) -> Result<Settings> {
    get_prefix(datastore, committed, "settings.", None)
        .transpose()
        // None is not OK here - we always have *some* settings
        .context(error::MissingData { prefix: "settings" })?
}

/// Build a Settings based on the data in the datastore that begins with the given prefix.
pub(crate) fn get_settings_prefix<D: DataStore, S: AsRef<str>>(
    datastore: &D,
    prefix: S,
    committed: &Committed,
) -> Result<Settings> {
    let prefix = "settings.".to_string() + prefix.as_ref();
    get_prefix(datastore, committed, &prefix, None)
        .transpose()
        // None is OK here - they could ask for a prefix we don't have
        .unwrap_or_else(|| Ok(Settings::default()))
}

// The "os" APIs don't deal with the data store at all, they just read a release field.
/// Build a BottlerocketRelease using the bottlerocket-release library.
pub(crate) fn get_os_info() -> Result<BottlerocketRelease> {
    BottlerocketRelease::new().context(error::ReleaseData)
}

/// Build a Services based on the data in the datastore.
pub(crate) fn get_services<D: DataStore>(datastore: &D) -> Result<Services> {
    get_prefix(
        datastore,
        &Committed::Live,
        "services.",
        Some("services".to_string()),
    )
    .transpose()
    // None is not OK here - we always have services
    .context(error::MissingData { prefix: "services" })?
}

/// Build a ConfigurationFiles based on the data in the datastore.
pub(crate) fn get_configuration_files<D: DataStore>(datastore: &D) -> Result<ConfigurationFiles> {
    get_prefix(
        datastore,
        &Committed::Live,
        "configuration-files",
        Some("configuration-files".to_string()),
    )
    .transpose()
    // None is not OK here - we always have configuration files
    .context(error::MissingData {
        prefix: "configuration-files",
    })?
}

/// Helper to get data from the datastore, starting with the given find_prefix, and deserialize it
/// into the desired type.  map_prefix should be the prefix to remove if you're deserializing into
/// a map; see docs on from_map_with_prefix.  Returns Err if we couldn't pull expected data;
/// returns Ok(None) if we found there were no populated keys.
fn get_prefix<D, T, S>(
    datastore: &D,
    committed: &Committed,
    find_prefix: S,
    map_prefix: Option<String>,
) -> Result<Option<T>>
where
    D: DataStore,
    T: DeserializeOwned,
    S: AsRef<str>,
{
    let find_prefix = find_prefix.as_ref();

    let data = datastore
        .get_prefix(find_prefix, committed)
        .with_context(|| error::DataStore {
            op: format!("get_prefix '{}' for {:?}", find_prefix, committed),
        })?;
    if data.is_empty() {
        return Ok(None);
    }

    from_map_with_prefix(map_prefix, &data).context(error::Deserialization { given: find_prefix })
}

/// Build a Settings based on the data in the datastore for the given keys.
pub(crate) fn get_settings_keys<D: DataStore>(
    datastore: &D,
    keys: &HashSet<&str>,
    committed: &Committed,
) -> Result<Settings> {
    let mut data = HashMap::new();
    for key_str in keys {
        trace!("Pulling value from datastore for key: {}", key_str);
        let key = Key::new(KeyType::Data, &key_str).context(error::NewKey {
            key_type: "data",
            name: *key_str,
        })?;
        let value = match datastore
            .get_key(&key, committed)
            .context(error::DataStore { op: "get_key" })?
        {
            Some(v) => v,
            // TODO: confirm we want to skip requested keys if not populated, or error
            None => continue,
        };
        data.insert(key, value);
    }

    let settings = from_map(&data).context(error::Deserialization {
        given: "given keys",
    })?;
    Ok(settings)
}

/// Build a collection of Service items with the given names using data from the datastore.
pub(crate) fn get_services_names<'a, D: DataStore>(
    datastore: &D,
    names: &'a HashSet<&str>,
    committed: &Committed,
) -> Result<Services> {
    get_map_from_prefix(datastore, "services.".to_string(), names, committed)
}

/// Build a collection of ConfigurationFile items with the given names using data from the
/// datastore.
pub(crate) fn get_configuration_files_names<D: DataStore>(
    datastore: &D,
    names: &HashSet<&str>,
    committed: &Committed,
) -> Result<ConfigurationFiles> {
    get_map_from_prefix(
        datastore,
        "configuration-files.".to_string(),
        names,
        committed,
    )
}

/// Helper to get data from the datastore for a collection of requested items under a given prefix.  For
/// example, a collection of Service items under "services" that have the requested names.
/// Returns Err if we couldn't pull expected data, including the case where a name was specified
/// for which we have no data.
fn get_map_from_prefix<D: DataStore, T>(
    datastore: &D,
    prefix: String,
    names: &HashSet<&str>,
    committed: &Committed,
) -> Result<HashMap<String, T>>
where
    T: DeserializeOwned,
{
    let mut result = HashMap::new();
    for &name in names {
        let item_prefix = prefix.clone() + name;

        let item_data = datastore
            .get_prefix(&item_prefix, committed)
            .with_context(|| error::DataStore {
                op: format!("get_prefix '{}' for {:?}", &item_prefix, committed),
            })?;

        ensure!(
            !item_data.is_empty(),
            error::ListKeys {
                requested: item_prefix
            }
        );

        let item = from_map_with_prefix(Some(item_prefix.clone()), &item_data)
            .context(error::Deserialization { given: item_prefix })?;
        result.insert(name.to_string(), item);
    }

    Ok(result)
}

/// Given a Settings, takes any Some values and updates them in the datastore.
pub(crate) fn set_settings<D: DataStore>(
    datastore: &mut D,
    settings: &Settings,
    transaction: &str,
) -> Result<()> {
    trace!("Serializing Settings to write to data store");
    let pairs = to_pairs(settings).context(error::DataStoreSerialization { given: "Settings" })?;
    let pending = Committed::Pending {
        tx: transaction.into(),
    };
    datastore
        .set_keys(&pairs, &pending)
        .context(error::DataStore { op: "set_keys" })
}

// This is not as nice as get_settings, which uses Serializer/Deserializer to properly use the
// data model and check types.
/// Gets the value of a metadata key for the requested list of data keys.
pub(crate) fn get_metadata_for_data_keys<D: DataStore, S: AsRef<str>>(
    datastore: &D,
    md_key_str: S,
    data_key_strs: &HashSet<&str>,
) -> Result<HashMap<String, Value>> {
    trace!("Getting metadata '{}'", md_key_str.as_ref());
    let md_key = Key::new(KeyType::Meta, md_key_str.as_ref()).context(error::NewKey {
        key_type: "meta",
        name: md_key_str.as_ref(),
    })?;

    let mut result = HashMap::new();
    for data_key_str in data_key_strs {
        trace!("Pulling metadata from datastore for key: {}", data_key_str);
        let data_key = Key::new(KeyType::Data, data_key_str).context(error::NewKey {
            key_type: "data",
            name: *data_key_str,
        })?;
        let value_str = match datastore.get_metadata(&md_key, &data_key) {
            Ok(Some(v)) => v,
            // TODO: confirm we want to skip requested keys if not populated, or error
            Ok(None) => continue,
            // May want to make it possible to receive an error if a metadata key doesn't
            // exist, but to start, we expect to request metadata for multiple keys and not all
            // of them will necessarily have the metadata.
            Err(_) => continue,
        };
        trace!("Deserializing scalar from metadata");
        let value: Value = deserialize_scalar::<_, ScalarError>(&value_str)
            .context(error::InvalidMetadata { key: md_key.name() })?;
        result.insert(data_key.to_string(), value);
    }

    Ok(result)
}

/// Gets the value of a metadata key everywhere it's found in the data store.  Returns a mapping
/// of data key to the metadata value associated with the requested key.
pub(crate) fn get_metadata_for_all_data_keys<D: DataStore, S: AsRef<str>>(
    datastore: &D,
    md_key_str: S,
) -> Result<HashMap<String, Value>> {
    trace!("Getting metadata '{}'", md_key_str.as_ref());
    let meta_map = datastore
        .get_metadata_prefix("", &Some(md_key_str))
        .context(error::DataStore {
            op: "get_metadata_prefix",
        })?;

    let mut result = HashMap::new();
    for (data_key, metadata) in meta_map {
        for (meta_key, value_str) in metadata {
            trace!("Deserializing scalar from metadata");
            let value: Value = deserialize_scalar::<_, ScalarError>(&value_str).context(
                error::InvalidMetadata {
                    key: meta_key.name(),
                },
            )?;
            result.insert(data_key.to_string(), value);
        }
    }
    Ok(result)
}

/// Makes live any pending settings in the datastore, returning the changed keys.
pub(crate) fn commit_transaction<D>(datastore: &mut D, transaction: &str) -> Result<HashSet<Key>>
where
    D: DataStore,
{
    datastore
        .commit_transaction(transaction)
        .context(error::DataStore { op: "commit" })
}

/// Launches the config applier to make appropriate changes to the system based on any settings
/// that have been committed.  Can be called after a commit, with the keys that changed in that
/// commit, or called on its own to reset configuration state with all known keys.
///
/// If `keys_limit` is Some, gives those keys to the applier so only changes relevant to those
/// keys are made.  Otherwise, tells the applier to apply changes for all known keys.
pub(crate) fn apply_changes<S>(keys_limit: Option<&HashSet<S>>) -> Result<()>
where
    S: AsRef<str>,
{
    if let Some(keys_limit) = keys_limit {
        let keys_limit: Vec<&str> = keys_limit.iter().map(|s| s.as_ref()).collect();
        // Prepare input to config applier; it uses the changed keys to update the right config
        trace!("Serializing the commit's changed keys: {:?}", keys_limit);
        let cmd_input =
            serde_json::to_string(&keys_limit).context(error::CommandSerialization {
                given: "commit's changed keys",
            })?;

        // Start config applier
        debug!("Launching thar-be-settings to apply changes");
        let mut cmd = Command::new("/usr/bin/thar-be-settings")
            .stdin(Stdio::piped())
            // FIXME where to send output?
            //.stdout()
            //.stderr()
            .spawn()
            .context(error::ConfigApplierStart)?;

        // Send changed keys to config applier
        trace!("Sending changed keys");
        cmd.stdin
            .as_mut()
            .context(error::ConfigApplierStdin)?
            .write_all(cmd_input.as_bytes())
            .context(error::ConfigApplierWrite)?;
    } else {
        // Start config applier
        debug!("Launching thar-be-settings to apply any and all changes");
        Command::new("/usr/bin/thar-be-settings")
            .arg("--all")
            // FIXME where to send output?
            //.stdout()
            //.stderr()
            .spawn()
            .context(error::ConfigApplierStart)?;
    }

    // Leave config applier to run in the background; we can't wait for it
    Ok(())
}

/// Dispatches an update command via `thar-be-updates`
pub(crate) fn dispatch_update_command(args: &[&str]) -> Result<HttpResponse> {
    let status = Command::new("/usr/bin/thar-be-updates")
        .args(args)
        .status()
        .context(error::UpdateDispatcher)?;
    if status.success() {
        return Ok(HttpResponse::NoContent().finish());
    }
    let exit_status = match status.code() {
        Some(code) => code,
        None => status.signal().unwrap_or(1),
    };
    let error_type = FromPrimitive::from_i32(exit_status);
    let error = match error_type {
        Some(TbuErrorStatus::UpdateLockHeld) => error::Error::UpdateLockHeld,
        Some(TbuErrorStatus::DisallowCommand) => error::Error::DisallowCommand,
        Some(TbuErrorStatus::UpdateDoesNotExist) => error::Error::UpdateDoesNotExist,
        Some(TbuErrorStatus::NoStagedImage) => error::Error::NoStagedImage,
        // other errors
        _ => error::Error::UpdateError,
    };
    Err(error)
}

#[cfg(test)]
mod test {
    use super::*;
    use datastore::memory::MemoryDataStore;
    use datastore::{Committed, DataStore, Key, KeyType};
    use maplit::{hashmap, hashset};
    use model::Service;
    use std::convert::TryInto;

    #[test]
    fn get_settings_works() {
        let mut ds = MemoryDataStore::new();
        // Set directly with data store
        ds.set_key(
            &Key::new(KeyType::Data, "settings.motd").unwrap(),
            "\"json string\"",
            &Committed::Live,
        )
        .unwrap();

        // Retrieve with helper
        let settings = get_settings(&ds, &Committed::Live).unwrap();
        assert_eq!(settings.motd, Some("json string".try_into().unwrap()));
    }

    #[test]
    fn get_settings_prefix_works() {
        let mut ds = MemoryDataStore::new();
        // Set directly with data store
        ds.set_key(
            &Key::new(KeyType::Data, "settings.motd").unwrap(),
            "\"json string\"",
            &Committed::Live,
        )
        .unwrap();

        // Retrieve with helper
        let settings = get_settings_prefix(&ds, "", &Committed::Live).unwrap();
        assert_eq!(settings.motd, Some("json string".try_into().unwrap()));

        let settings = get_settings_prefix(&ds, "mot", &Committed::Live).unwrap();
        assert_eq!(settings.motd, Some("json string".try_into().unwrap()));

        let settings = get_settings_prefix(&ds, "motdxxx", &Committed::Live).unwrap();
        assert_eq!(settings.motd, None);
    }

    #[test]
    fn get_settings_keys_works() {
        let mut ds = MemoryDataStore::new();
        // Set directly with data store
        ds.set_key(
            &Key::new(KeyType::Data, "settings.motd").unwrap(),
            "\"json string 1\"",
            &Committed::Live,
        )
        .unwrap();

        ds.set_key(
            &Key::new(KeyType::Data, "settings.ntp.time-servers").unwrap(),
            "\"json string 2\"",
            &Committed::Live,
        )
        .unwrap();

        // Retrieve with helper
        let settings =
            get_settings_keys(&ds, &hashset!("settings.motd"), &Committed::Live).unwrap();
        assert_eq!(settings.motd, Some("json string 1".try_into().unwrap()));
        assert_eq!(settings.ntp, None);
    }

    #[test]
    fn get_services_names_works() {
        let mut ds = MemoryDataStore::new();
        // Set directly with data store
        ds.set_key(
            &Key::new(KeyType::Data, "services.foo.configuration-files").unwrap(),
            "[\"file1\"]",
            &Committed::Live,
        )
        .unwrap();
        ds.set_key(
            &Key::new(KeyType::Data, "services.foo.restart-commands").unwrap(),
            "[\"echo hi\"]",
            &Committed::Live,
        )
        .unwrap();

        // Retrieve built service
        let names = hashset!("foo");
        let services = get_services_names(&ds, &names, &Committed::Live).unwrap();
        assert_eq!(
            services,
            hashmap!("foo".to_string() => Service {
                configuration_files: vec!["file1".try_into().unwrap()],
                restart_commands: vec!["echo hi".to_string()]
            })
        );
    }

    #[test]
    fn set_settings_works() {
        let mut settings = Settings::default();
        settings.motd = Some("tz".try_into().unwrap());

        // Set with helper
        let mut ds = MemoryDataStore::new();
        let tx = "test transaction";
        let pending = Committed::Pending { tx: tx.into() };
        set_settings(&mut ds, &settings, tx).unwrap();

        // Retrieve directly
        let key = Key::new(KeyType::Data, "settings.motd").unwrap();
        assert_eq!(
            Some("\"tz\"".to_string()),
            ds.get_key(&key, &pending).unwrap()
        );
    }

    #[test]
    fn get_metadata_keys_works() {
        let mut ds = MemoryDataStore::new();
        // Set directly with data store
        for data_key in &["abc", "def", "ghi"] {
            ds.set_metadata(
                &Key::new(KeyType::Meta, "my-meta").unwrap(),
                &Key::new(KeyType::Data, data_key).unwrap(),
                "\"json string\"",
            )
            .unwrap();
        }

        // We'll check a subset by specifying 2 of the 3 keys
        let expected = hashmap!(
            "abc".to_string() => "json string".into(),
            "def".to_string() => "json string".into(),
        );
        // Retrieve with helper
        let actual = get_metadata_for_data_keys(&ds, "my-meta", &hashset!("abc", "def")).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_metadata_all_works() {
        let mut ds = MemoryDataStore::new();
        // Set directly with data store
        for data_key in &["abc", "def"] {
            ds.set_metadata(
                &Key::new(KeyType::Meta, "my-meta").unwrap(),
                &Key::new(KeyType::Data, data_key).unwrap(),
                "\"json string\"",
            )
            .unwrap();
        }

        let expected = hashmap!(
            "abc".to_string() => "json string".into(),
            "def".to_string() => "json string".into(),
        );
        // Retrieve with helper
        let actual = get_metadata_for_all_data_keys(&ds, "my-meta").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn commit_works() {
        // Set directly with data store
        let mut ds = MemoryDataStore::new();
        let tx = "test transaction";
        let pending = Committed::Pending { tx: tx.into() };
        ds.set_key(
            &Key::new(KeyType::Data, "settings.motd").unwrap(),
            "\"json string\"",
            &pending,
        )
        .unwrap();

        // Confirm pending
        let settings = get_settings(&ds, &pending).unwrap();
        assert_eq!(settings.motd, Some("json string".try_into().unwrap()));
        // No live settings yet
        get_settings(&ds, &Committed::Live).unwrap_err();

        // Commit, pending -> live
        commit_transaction(&mut ds, tx).unwrap();

        // No more pending settings
        get_settings(&ds, &pending).unwrap_err();
        // Confirm live
        let settings = get_settings(&ds, &Committed::Live).unwrap();
        assert_eq!(settings.motd, Some("json string".try_into().unwrap()));
    }
}
