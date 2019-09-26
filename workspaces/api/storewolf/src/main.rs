/*!
# Introduction

storewolf is a small program to create the filesystem datastore.

It creates the datastore at a provided path and populates any default
settings given in the defaults.toml file, unless they already exist.
*/
#![deny(rust_2018_idioms)]

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snafu::{OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::{env, fs, process};

use apiserver::datastore::key::{Key, KeyType};
use apiserver::datastore::serialization::{to_pairs, to_pairs_with_prefix};
use apiserver::datastore::{self, DataStore, FilesystemDataStore, ScalarError};
use apiserver::model;
use data_store_version::Version;

#[macro_use]
extern crate log;

// FIXME Get these from configuration in the future
const DATASTORE_VERSION_FILE: &str = "/usr/share/thar/data-store-version";

mod error {
    use std::io;
    use std::path::PathBuf;

    use apiserver::datastore::key::KeyType;
    use apiserver::datastore::{self, serialization, ScalarError};
    use data_store_version::error::Error as DataStoreVersionError;
    use snafu::Snafu;

    /// Potential errors during execution
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum StorewolfError {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Unable to clear pending settings: {}", source))]
        DeletePending { source: io::Error },

        #[snafu(display("Unable to create datastore at '{}': {}", path.display(), source))]
        DatastoreCreation { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to read datastore version from '{}': {}", path.display(), source))]
        DatastoreVersion {
            path: PathBuf,
            source: DataStoreVersionError,
        },

        #[snafu(display("defaults.toml is not valid TOML: {}", source))]
        DefaultsFormatting { source: toml::de::Error },

        #[snafu(display("defaults.toml is not a TOML table"))]
        DefaultsNotTable {},

        #[snafu(display("defaults.toml's settings is not a TOML table"))]
        DefaultSettingsNotTable {},

        #[snafu(display("defaults.toml's metadata is not a TOML list of Metadata"))]
        DefaultsMetadataNotTable { source: toml::de::Error },

        #[snafu(display("Error querying datstore for populated keys: {}", source))]
        QueryData { source: datastore::Error },

        #[snafu(display("Error querying datstore for populated metadata: {}", source))]
        QueryMetadata { source: datastore::Error },

        #[snafu(display("Error serializing {}: {} ", given, source))]
        Serialization {
            given: String,
            source: serialization::Error,
        },

        #[snafu(display("Error serializing scalar {}: {} ", given, source))]
        SerializeScalar { given: String, source: ScalarError },

        #[snafu(display("Unable to write keys to the datastore: {}", source))]
        WriteKeys { source: datastore::Error },

        #[snafu(display("Unable to create {:?} key '{}': {}", key_type, key, source))]
        InvalidKey {
            key_type: KeyType,
            key: String,
            source: datastore::Error,
        },

        #[snafu(display("Unable to write metadata to the datastore: {}", source))]
        WriteMetadata { source: datastore::Error },

        #[snafu(display("Failed to create symlink at '{}': {}", path.display(), source))]
        LinkCreate { path: PathBuf, source: io::Error },

        #[snafu(display("Data store link '{}' points to /", path.display()))]
        DataStoreLinkToRoot { path: PathBuf },
    }
}

use error::StorewolfError;

type Result<T> = std::result::Result<T, StorewolfError>;

/// Given a base path, create a brand new datastore with the appropriate
/// symlink structure.
///
/// An example setup for theoretical version 1.5:
///    /path/to/datastore/current
///    -> /path/to/datastore/v1
///    -> /path/to/datastore/v1.5
///    -> /path/to/datastore/v1.5_0123456789abcdef
fn create_new_datastore<P: AsRef<Path>>(base_path: P) -> Result<()> {
    // Get the datastore version from the version file
    let datastore_version =
        Version::from_file(&DATASTORE_VERSION_FILE).context(error::DatastoreVersion {
            path: &DATASTORE_VERSION_FILE,
        })?;
    // Create random string to append to the end of the new datastore path
    let random_id: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

    // Build the various paths to which we'll symlink

    // /path/to/datastore/v1.5_0123456789abcdef
    let data_store_filename = format!("{}_{}", datastore_version, random_id);
    let data_store_path = base_path.as_ref().join(&data_store_filename);

    // /path/to/datastore/v1
    let major_version_filename = format!("v{}", datastore_version.major);
    let major_version_path = base_path.as_ref().join(&major_version_filename);

    // /path/to/datastore/v1.5
    let minor_version_filename = format!("{}", datastore_version);
    let minor_version_path = base_path.as_ref().join(&minor_version_filename);

    // /path/to/datastore/current
    let current_path = base_path.as_ref().join("current");

    // Create the path to the datastore, i.e /path/to/datastore/v1.5_0123456789abcdef
    fs::create_dir_all(&data_store_path).context(error::DatastoreCreation {
        path: &base_path.as_ref(),
    })?;

    // Build our symlink chain (See example in docstring above)
    // /path/to/datastore/v1.5 -> v1.5_0123456789abcdef
    symlink(&data_store_filename, &minor_version_path).context(error::LinkCreate {
        path: &minor_version_path,
    })?;
    // /path/to/datastore/v1 -> v1.5
    symlink(&minor_version_filename, &major_version_path).context(error::LinkCreate {
        path: &major_version_path,
    })?;
    // /path/to/datastore/current -> v1
    symlink(&major_version_filename, &current_path).context(error::LinkCreate {
        path: &current_path,
    })?;
    Ok(())
}

/// Creates a new FilesystemDataStore at the given path, with data and metadata coming from
/// defaults.toml at compile time.
fn populate_default_datastore<P: AsRef<Path>>(base_path: P) -> Result<()> {
    // NOTE: Variables prefixed with "def" refer to values from defaults.
    //
    // Variables prefixed with "existing..." refer to values from the
    // existing datastore.

    // There's a chain of symlinks that point to the directory where data
    // actually lives. This is the start of the chain, whose name never
    // changes, so it can be used consistently by the rest of the OS.
    let datastore_path = base_path.as_ref().join("current");
    let mut datastore = FilesystemDataStore::new(&datastore_path);
    let mut existing_data = HashSet::new();
    let mut existing_metadata = HashMap::new();

    // If the "live" path of the datastore exists, query it for populated
    // meta/data.  Otherwise, create the datastore path.
    let live_path = &datastore_path.join("live");
    if live_path.exists() {
        debug!("Gathering existing data from the datastore");
        existing_metadata = datastore
            .list_populated_metadata("", &None as &Option<&str>)
            .context(error::QueryMetadata)?;
        existing_data = datastore
            .list_populated_keys("", datastore::Committed::Live)
            .context(error::QueryData)?;
    } else {
        info!("Creating datastore at: {}", &live_path.display());
        create_new_datastore(&base_path)?;
    }

    // Read and parse defaults
    let defaults_str = include_str!("../defaults.toml");
    let mut defaults_val: toml::Value =
        toml::from_str(defaults_str).context(error::DefaultsFormatting)?;

    // Check if we have metadata and settings. If so, pull them out
    // of `defaults_val`
    let table = defaults_val
        .as_table_mut()
        .context(error::DefaultsNotTable)?;
    let maybe_metadata_val = table.remove("metadata");
    let maybe_settings_val = table.remove("settings");

    // If there are default settings, write them to the datastore in Pending
    // state. This ensures the settings will go through a commit cycle when
    // first-boot services run, which will create config files for default
    // keys that require them.
    if let Some(def_settings_val) = maybe_settings_val {
        debug!("Serializing default settings and writing new ones to datastore");
        let def_settings_table = def_settings_val
            .as_table()
            .context(error::DefaultSettingsNotTable)?;

        // The default settings were removed from the "settings" key of the
        // defaults table above. We still need them under a "settings" key
        // before serializing so we have full dotted keys like
        // "settings.foo.bar" and not just "foo.bar". We use a HashMap
        // to rebuild the nested structure.
        let def_settings = to_pairs_with_prefix("settings".to_string(), &def_settings_table)
            .context(error::Serialization {
                given: "default settings",
            })?;

        // For each of the default settings, check if it exists in the
        // datastore. If not, add it to the map of settings to write
        let mut settings_to_write = HashMap::new();
        for (key, val) in def_settings {
            let setting = Key::new(KeyType::Data, &key).context(error::InvalidKey {
                key_type: KeyType::Data,
                key: key.to_string(),
            })?;
            if !existing_data.contains(&setting) {
                settings_to_write.insert(key, val);
            }
        }

        trace!(
            "Writing default settings to datastore: {:#?}",
            &settings_to_write
        );
        datastore
            .set_keys(&settings_to_write, datastore::Committed::Pending)
            .context(error::WriteKeys)?;
    }

    // If we have metadata, write it out to the datastore in Live state
    if let Some(def_metadata_val) = maybe_metadata_val {
        debug!("Serializing metadata and writing new keys to datastore");
        let def_metadatas: Vec<model::Metadata> = def_metadata_val
            .try_into()
            .context(error::DefaultsMetadataNotTable)?;

        // Before this transformation, `existing_metadata` is a
        // map of data key to set of metadata keys:
        // HashMap(dataKey => HashSet(metaKey)).
        //
        // To make comparison easier, we
        // flatten the map to a HashSet of tuples:
        // HashSet((dataKey, metaKey)).
        let existing_metadata: HashSet<(&Key, &Key)> = existing_metadata
            .iter()
            .flat_map(|data| data.1.iter().map(move |md_key| (data.0, md_key)))
            .collect();

        // For each of the default metadatas, check if it exists in the
        // datastore. If not, add it to the set of metadatas to write
        let mut metadata_to_write = HashSet::new();
        for def_metadata in def_metadatas {
            let model::Metadata { key, md, val } = def_metadata;
            let data_key = Key::new(KeyType::Data, &key).context(error::InvalidKey {
                key_type: KeyType::Data,
                key,
            })?;
            let md_key = Key::new(KeyType::Meta, &md).context(error::InvalidKey {
                key_type: KeyType::Meta,
                key: md,
            })?;

            // Put the `data_key` and `md_key` tuple into a variable so we
            // can more easily read the subsequent `contains()` call
            let def_metadata_keypair = (&data_key, &md_key);
            if !existing_metadata.contains(&def_metadata_keypair) {
                let value =
                    datastore::serialize_scalar::<_, ScalarError>(&val).with_context(|| {
                        error::SerializeScalar {
                            given: format!("metadata value '{}'", val),
                        }
                    })?;
                metadata_to_write.insert((md_key, data_key, value));
            }
        }

        trace!(
            "Writing default metadata to datastore: {:#?}",
            metadata_to_write
        );
        for metadata in metadata_to_write {
            let (md, key, val) = metadata;
            datastore
                .set_metadata(&md, &key, val)
                .context(error::WriteMetadata)?;
        }
    }

    // If any other defaults remain (configuration files, services, etc),
    // write them to the datastore in Live state
    debug!("Serializing other defaults and writing new ones to datastore");
    let defaults = to_pairs(&defaults_val).context(error::Serialization {
        given: "other defaults",
    })?;

    let mut other_defaults_to_write = HashMap::new();
    if !defaults.is_empty() {
        for (key, val) in defaults.iter() {
            let data_key = Key::new(KeyType::Data, &key).context(error::InvalidKey {
                key_type: KeyType::Data,
                key,
            })?;
            if !existing_data.contains(&data_key) {
                other_defaults_to_write.insert(key, val);
            }
        }

        trace!(
            "Writing other default data to datastore: {:#?}",
            &other_defaults_to_write
        );
        datastore
            .set_keys(&other_defaults_to_write, datastore::Committed::Live)
            .context(error::WriteKeys)?;
    }
    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    verbosity: usize,
    data_store_base_path: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --data-store-base-path PATH
            [ --verbose --verbose ... ]
        ",
        program_name
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut data_store_base_path = None;
    let mut verbosity = 2;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--verbose" => verbosity += 1,

            "--data-store-base-path" => {
                data_store_base_path = Some(iter.next().unwrap_or_else(|| {
                    usage_msg("Did not give argument to --data-store-base-path")
                }))
            }

            _ => usage(),
        }
    }

    Args {
        verbosity,
        data_store_base_path: data_store_base_path.unwrap_or_else(|| usage()),
    }
}

fn main() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TODO Fix this later when we decide our logging story
    // Start the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(args.verbosity)
        .color(stderrlog::ColorChoice::Never)
        .init()
        .context(error::Logger)?;

    info!("Storewolf started");

    // If anything exists in Pending state, delete it
    info!("Deleting pending settings");
    let pending_path = Path::new(&args.data_store_base_path)
        .join("current")
        .join("pending");
    if let Err(e) = fs::remove_dir_all(pending_path) {
        // If there are no pending settings, the directory won't exist.
        // Ignore the error in this case.
        if e.kind() != io::ErrorKind::NotFound {
            Err(e).context(error::DeletePending)?
        }
    }

    // Create the datastore if it doesn't exist
    info!("Populating datastore at: {}", &args.data_store_base_path);
    populate_default_datastore(&args.data_store_base_path)?;
    info!("Datastore populated");

    Ok(())
}
