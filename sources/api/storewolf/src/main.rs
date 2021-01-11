/*!
# Introduction

storewolf creates the filesystem datastore used by the API system.

It creates the datastore at a provided path and populates any default
settings given in the defaults.toml file, unless they already exist.
*/
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use semver::Version;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs, process};
use toml::{map::Entry, Value};

use datastore::key::{Key, KeyType};
use datastore::serialization::{to_pairs, to_pairs_with_prefix};
use datastore::{self, DataStore, FilesystemDataStore, ScalarError};
use model::modeled_types::SingleLineString;

// FIXME Get these from configuration in the future
// Shared transaction used by boot-time services.
const TRANSACTION: &str = "bottlerocket-launch";

mod error {
    use std::io;
    use std::path::PathBuf;

    use datastore::key::KeyType;
    use datastore::{self, serialization, ScalarError};
    use model::modeled_types::error::Error as ModeledTypesError;
    use snafu::Snafu;

    /// Potential errors during execution
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum StorewolfError {
        #[snafu(display("Unable to clear pending transactions: {}", source))]
        DeletePending { source: io::Error },

        #[snafu(display("Unable to create datastore: {}", source))]
        DatastoreCreation { source: storewolf::error::Error },

        #[snafu(display("{} is not valid TOML: {}", file, source))]
        DefaultsFormatting {
            file: String,
            source: toml::de::Error,
        },

        #[snafu(display("defaults.toml is not a TOML table"))]
        DefaultsNotTable {},

        #[snafu(display("defaults.toml's settings is not a TOML table"))]
        DefaultSettingsNotTable {},

        #[snafu(display("defaults.toml's metadata is not a TOML list of Metadata"))]
        DefaultsMetadataNotTable { source: toml::de::Error },

        #[snafu(display("defaults.toml's metadata has unexpected types"))]
        DefaultsMetadataUnexpectedFormat {},

        #[snafu(display("defaults.toml data types do not match types defined in current variant's override-defaults.toml"))]
        DefaultsVariantDoesNotMatch {},

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

        #[snafu(display("Data store link '{}' points to /", path.display()))]
        DataStoreLinkToRoot { path: PathBuf },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Internal error: {}", msg))]
        Internal { msg: String },

        #[snafu(display("Keys can't contain newlines: {}", source))]
        SingleLineString { source: ModeledTypesError },
    }
}

use error::StorewolfError;
use storewolf::create_new_datastore;

type Result<T> = std::result::Result<T, StorewolfError>;

/// Convert the generic toml::Value representing metadata into a
/// Vec<Metadata> that can be used to write the metadata to the datastore.
// The input to this function is a toml::Value that represents the metadata
// read from defaults.toml. This table is structured like so:
//
// Table({"settings": Table({"foo": Table({"affected-services": Array([ ... ])})})})
//
// This function will convert the above table to a Vec<model::Metadata>,
// validating the types and structure. The resulting Vec looks like:
//
// [
//   Metadata {key: "settings.motd", md: "affected-services", val: Array([ ... ])},
//   Metadata { ... },
// ]
fn parse_metadata_toml(md_toml_val: toml::Value) -> Result<Vec<model::Metadata>> {
    debug!("Parsing metadata toml");
    let mut def_metadatas: Vec<model::Metadata> = Vec::new();

    // Do a breadth-first search of the toml::Value table.
    // Create a Vec of tuples to keep track of where we have visited in the
    // toml::Value data structure. The first value in the tuple is the key
    // (represented as a Vec of key segments), the second is the toml::Value
    // associated with that key. It ends up looking like:
    // [
    //   (
    //     ["settings", "motd"],
    //     toml::Value
    //   ),
    //   ...
    // ]
    // For each key/value of the table we visit, match on the value of the
    // table. If it's another table, we add it to the list to process its
    // contents. If it is an array or string, we can construct a
    // model::Metadata, and add it to the Vec of model::Metadata to be
    // returned from the function.

    // Start at the root of the tree.
    let mut to_process = vec![(Vec::new(), md_toml_val)];

    while !to_process.is_empty() {
        let (mut path, toml_value) = to_process.pop().unwrap();
        trace!("Current metadata table path: {:#?}", &path);

        match toml_value {
            // A table means there is more processing to do. Add the current
            // key and value to the Vec to be processed further.
            toml::Value::Table(table) => {
                for (key, val) in table {
                    trace!("Found table for key '{}'", &key);
                    let mut path = path.clone();
                    path.push(key.to_string());
                    to_process.push((path, val));
                }
            }

            // An array or string means we're ready to create a model::Metadata
            val @ toml::Value::Array(_) | val @ toml::Value::String(_) => {
                // Get the metadata key from the end of the path
                let md_key = path.pop().context(error::Internal {
                    msg: "parse_metadata_toml found empty 'path' in the to_process vec - is 'metadata' not a Table?",
                })?;

                // Make sure that the path contains more than 1 item, i.e. ["settings", "motd"]
                ensure!(
                    path.len() >= 1,
                    error::Internal {
                        msg: format!(
                            "Cannot create empty metadata data key - is root not a Table?"
                        )
                    }
                );
                let data_key = path.join(".");

                trace!(
                    "Found metadata key '{}' for data key '{}'",
                    &md_key,
                    &data_key
                );

                // Ensure the metadata/data keys don't contain newline chars
                let md = SingleLineString::try_from(md_key).context(error::SingleLineString)?;
                let key = SingleLineString::try_from(data_key).context(error::SingleLineString)?;

                // Create the Metadata struct
                def_metadatas.push(model::Metadata { key, md, val })
            }

            // We don't recognize any other values yet, something is awry
            _ => return error::DefaultsMetadataUnexpectedFormat {}.fail(),
        };
    }
    Ok(def_metadatas)
}

/// This modifies the first given toml Value by inserting any values from the second Value.
///
/// This is done recursively.  Any time a scalar or array is seen, the left side is set to the
/// right side.  Any time a table is seen, we iterate through the keys of the tables; if the left
/// side does not have the key from the right side, it's inserted, otherwise we recursively merge
/// the values in each table for that key.
///
/// If at any point in the recursion the data types of the two values does not match, we error.
fn merge_values<'a>(merge_into: &'a mut Value, merge_from: &'a Value) -> Result<()> {
    // If the types of left and right don't match, we have inconsistent models, and shouldn't try
    // to merge them.
    ensure!(
        merge_into.same_type(&merge_from),
        error::DefaultsVariantDoesNotMatch
    );

    match merge_from {
        // If we see a scalar, we replace the left with the right.  We treat arrays like scalars so
        // behavior is clear - no question about whether we're appending right onto left, etc.
        Value::String(_)
        | Value::Integer(_)
        | Value::Float(_)
        | Value::Boolean(_)
        | Value::Datetime(_)
        | Value::Array(_) => *merge_into = merge_from.clone(),

        // If we see a table, we recursively merge each key.
        Value::Table(t2) => {
            // We know the other side is a table because of the `ensure` above.
            let t1 = merge_into.as_table_mut().unwrap();
            for (k2, v2) in t2.iter() {
                // Check if the left has the same key as the right.
                match t1.entry(k2) {
                    // If not, we can just insert the value.
                    Entry::Vacant(e) => {
                        e.insert(v2.clone());
                    }
                    // If so, we need to recursively merge; we don't want to replace an entire
                    // table, for example, because the left may have some distinct inner keys.
                    Entry::Occupied(ref mut e) => {
                        merge_values(e.get_mut(), v2)?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Creates a new FilesystemDataStore at the given path, with data and metadata coming from
/// defaults.toml at compile time.
fn populate_default_datastore<P: AsRef<Path>>(
    base_path: P,
    version: Option<Version>,
) -> Result<()> {
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
            .list_populated_keys("", &datastore::Committed::Live)
            .context(error::QueryData)?;
    } else {
        info!("Creating datastore at: {}", &live_path.display());
        create_new_datastore(&base_path, version).context(error::DatastoreCreation)?;
    }

    // Read and parse shared defaults
    let defaults_str = include_str!("../../../models/defaults.toml");
    let mut defaults_val: toml::Value =
        toml::from_str(defaults_str).context(error::DefaultsFormatting {
            file: "defaults.toml",
        })?;

    // Merge in any defaults for the current variant
    let variant_defaults_str =
        include_str!("../../../models/src/variant/current/override-defaults.toml");
    let variant_defaults_val: toml::Value =
        toml::from_str(variant_defaults_str).context(error::DefaultsFormatting {
            file: "override_defaults.toml",
        })?;
    merge_values(&mut defaults_val, &variant_defaults_val)?;

    // Check if we have metadata and settings. If so, pull them out
    // of `shared_defaults_val`
    let table = defaults_val
        .as_table_mut()
        .context(error::DefaultsNotTable)?;
    let maybe_metadata_val = table.remove("metadata");
    let maybe_settings_val = table.remove("settings");

    // If there are default settings, write them to the datastore in the shared pending
    // transaction. This ensures the settings will go through a commit cycle when first-boot
    // services run, which will create config files for default keys that require them.
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
        let def_settings = to_pairs_with_prefix("settings", &def_settings_table).context(
            error::Serialization {
                given: "default settings",
            },
        )?;

        // For each of the default settings, check if it exists in the
        // datastore. If not, add it to the map of settings to write
        let mut settings_to_write = HashMap::new();
        for (key, val) in def_settings {
            if !existing_data.contains(&key) {
                settings_to_write.insert(key, val);
            }
        }

        trace!(
            "Writing default settings to datastore: {:#?}",
            &settings_to_write
        );
        let pending = datastore::Committed::Pending {
            tx: TRANSACTION.to_string(),
        };
        datastore
            .set_keys(&settings_to_write, &pending)
            .context(error::WriteKeys)?;
    }

    // If we have metadata, write it out to the datastore in Live state
    if let Some(def_metadata_val) = maybe_metadata_val {
        debug!("Serializing metadata and writing new keys to datastore");
        // Create a Vec<Metadata> from the metadata toml::Value
        let def_metadatas = parse_metadata_toml(def_metadata_val)?;

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
        for (key, val) in defaults {
            if !existing_data.contains(&key) {
                other_defaults_to_write.insert(key, val);
            }
        }

        trace!(
            "Writing other default data to datastore: {:#?}",
            &other_defaults_to_write
        );
        datastore
            .set_keys(&other_defaults_to_write, &datastore::Committed::Live)
            .context(error::WriteKeys)?;
    }
    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    data_store_base_path: String,
    log_level: LevelFilter,
    version: Option<Version>,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --data-store-base-path PATH
            [ --version X.Y.Z ]
            [ --log-level trace|debug|info|warn|error ]

        If --version is not given, the version will be pulled from /etc/os-release.
        This is used to set up versioned symlinks in the data store base path.
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
    let mut log_level = None;
    let mut version = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--data-store-base-path" => {
                data_store_base_path = Some(iter.next().unwrap_or_else(|| {
                    usage_msg("Did not give argument to --data-store-base-path")
                }))
            }

            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }

            "--version" => {
                let version_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --version"));
                version = Some(
                    Version::from_str(&version_str)
                        .unwrap_or_else(|e| usage_msg(format!("Invalid version: {}", e))),
                );
            }

            _ => usage(),
        }
    }

    Args {
        data_store_base_path: data_store_base_path.unwrap_or_else(|| usage()),
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        version,
    }
}

fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    info!("Storewolf started");

    info!("Deleting pending transactions");
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
    populate_default_datastore(&args.data_store_base_path, args.version)?;
    info!("Datastore populated");

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use super::merge_values;
    use toml::toml;

    #[test]
    fn merge() {
        let mut left = toml! {
            top1 = "left top1"
            top2 = "left top2"
            [settings.inner]
            inner_setting1 = "left inner_setting1"
            inner_setting2 = "left inner_setting2"
        };
        let right = toml! {
            top1 = "right top1"
            [settings]
            setting = "right setting"
            [settings.inner]
            inner_setting1 = "right inner_setting1"
            inner_setting3 = "right inner_setting3"
        };
        // Can't comment inside this toml, unfortunately.
        // "top1" is being overwritten from right.
        // "top2" is only in the left and remains.
        // "setting" is only in the right side.
        // "inner" tests that recursion works; inner_setting1 is replaced, 2 is untouched, and
        // 3 is new.
        let expected = toml! {
            top1 = "right top1"
            top2 = "left top2"
            [settings]
            setting = "right setting"
            [settings.inner]
            inner_setting1 = "right inner_setting1"
            inner_setting2 = "left inner_setting2"
            inner_setting3 = "right inner_setting3"
        };
        merge_values(&mut left, &right).unwrap();
        assert_eq!(left, expected);
    }
}
