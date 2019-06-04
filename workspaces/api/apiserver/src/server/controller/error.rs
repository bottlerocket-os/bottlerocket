use crate::datastore::{self, deserialization, serialization};
use snafu::Snafu;
use std::io;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Found no '{}' in datastore", prefix))]
    MissingData { prefix: String },

    #[snafu(display("Found no '{}' in datastore", requested))]
    ListKeys {
        requested: String,
    },

    #[snafu(display("Listed key '{}' not found on disk", key))]
    ListedKeyNotPresent {
        key: String,
    },

    #[snafu(display("Data store error during {}: {}", op, source))]
    DataStore {
        op: String,
        source: datastore::Error,
    },

    #[snafu(display("Error deserializing {}: {} ", given, source))]
    Deserialization {
        given: String,
        source: deserialization::Error,
    },

    #[snafu(display("Error serializing {}: {} ", given, source))]
    Serialization {
        given: String,
        source: serialization::Error,
    },

    #[snafu(display("Error serializing {} to JSON: {} ", given, source))]
    Json {
        given: String,
        source: serde_json::Error,
    },

    #[snafu(display("Unable to make {} key '{}': {}", key_type, name, source))]
    NewKey {
        key_type: String,
        name: String,
        source: datastore::Error,
    },

    #[snafu(display("Input is not valid JSON: {}", source))]
    InvalidJson { source: serde_json::Error },

    #[snafu(display("Metadata '{}' is not valid JSON: {}", key, source))]
    InvalidMetadata {
        key: String,
        source: serde_json::Error,
    },

    #[snafu(display("Input is not a JSON object"))]
    NotJsonObject {},

    #[snafu(display(r#"Settings input must either be formatted like {{"settings": {{"a": "b"}}}} or {{"a": "b"}}, where the {{"a": "b"}} mapping corresponds to valid settings."#))]
    NoSettings {},

    #[snafu(display(
        r#"Value inside {{"settings": x}} is not a valid Settings: {}"#,
        source
    ))]
    InvalidSettings { source: serde_json::Error },

    #[snafu(display("Unable to start config applier: {} ", source))]
    ConfigApplierStart { source: io::Error },

    #[snafu(display("Unable to use config applier, couldn't get stdin"))]
    ConfigApplierStdin {},

    #[snafu(display("Unable to send input to config applier: {}", source))]
    ConfigApplierWrite { source: io::Error },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
