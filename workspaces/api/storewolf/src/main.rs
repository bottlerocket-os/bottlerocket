/*!
# Introduction

storewolf is a small program to create the filesystem datastore.

It creates the datastore at a provided path and populates any default
settings given in the defaults.toml file.
*/

use snafu::{OptionExt, ResultExt};
use std::path::Path;
use std::{env, process};

use apiserver::datastore::key::{Key, KeyType};
use apiserver::datastore::serialization::to_pairs;
use apiserver::datastore::{self, DataStore, FilesystemDataStore, ScalarError};
use apiserver::model;

#[macro_use]
extern crate log;

mod error {
    use apiserver::datastore::key::KeyType;
    use apiserver::datastore::{self, serialization, ScalarError};
    use snafu::Snafu;

    /// Potential errors during execution
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum StorewolfError {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("defaults.toml is not valid TOML: {}", source))]
        DefaultsFormatting { source: toml::de::Error },

        #[snafu(display("defaults.toml is not a TOML table"))]
        DefaultsNotTable {},

        #[snafu(display("defaults.toml's metadata is not a TOML list of Metadata"))]
        DefaultsMetadataNotTable { source: toml::de::Error },

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
    }
}

use error::StorewolfError;

type Result<T> = std::result::Result<T, StorewolfError>;

/// Creates a new FilesystemDataStore at the given path, with data and metadata coming from
/// defaults.toml at compile time.
fn populate_default_datastore<P: AsRef<Path>>(base_path: P) -> Result<()> {
    // Read and parse defaults
    let defaults_str = include_str!("../defaults.toml");
    let mut defaults_val: toml::Value =
        toml::from_str(defaults_str).context(error::DefaultsFormatting)?;

    // Check if we have metadata
    let table = defaults_val
        .as_table_mut()
        .context(error::DefaultsNotTable)?;
    let maybe_metadata_val = table.remove("metadata");

    // Write defaults to datastore
    debug!("Serializing defaults and writing to datastore");
    let defaults = to_pairs(&defaults_val).context(error::Serialization { given: "defaults" })?;
    let mut datastore = FilesystemDataStore::new(base_path);
    datastore
        .set_keys(&defaults, datastore::Committed::Live)
        .context(error::WriteKeys)?;

    // If we had metadata, write it out
    if let Some(metadata_val) = maybe_metadata_val {
        debug!("Serializing metadata and writing to datastore");
        let metadatas: Vec<model::Metadata> = metadata_val
            .try_into()
            .context(error::DefaultsMetadataNotTable)?;

        for metadata in metadatas {
            let model::Metadata { key, md, val } = metadata;
            let data_key = Key::new(KeyType::Data, &key).context(error::InvalidKey {
                key_type: KeyType::Data,
                key,
            })?;
            let md_key = Key::new(KeyType::Meta, &md).context(error::InvalidKey {
                key_type: KeyType::Meta,
                key: md,
            })?;
            let value = datastore::serialize_scalar::<_, ScalarError>(&val).with_context(|| {
                error::SerializeScalar {
                    given: format!("metadata value '{}'", val),
                }
            })?;

            datastore
                .set_metadata(&md_key, &data_key, value)
                .context(error::WriteMetadata)?;
        }
    }

    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    verbosity: usize,
    datastore_path: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
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
    let mut datastore_path = None;
    let mut verbosity = 2;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--verbose" => verbosity += 1,

            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path")),
                )
            }

            _ => usage(),
        }
    }

    Args {
        verbosity,
        datastore_path: datastore_path.unwrap_or_else(|| usage()),
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

    // Create default datastore if it doesn't exist
    if !Path::new(&args.datastore_path).exists() {
        info!("Creating datastore at: {}", &args.datastore_path);
        populate_default_datastore(&args.datastore_path)?;
        info!("Datastore created");
    }

    Ok(())
}
