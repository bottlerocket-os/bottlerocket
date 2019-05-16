//! The deserialization module implements generic deserialization techniques that are particularly
//! useful for populating Rust structures from the datastore.

mod pairs;

pub use pairs::{from_map, from_map_with_prefix};

use serde::de;

/// Potential errors from deserialization.
#[derive(Debug, Error)]
pub enum DeserializationError {
    // This error variant is required to implement ser::Error for serde.
    #[error(msg_embedded, no_from, non_std)]
    /// Error during serialization
    Message(String),

    /// Error deserializing scalar value
    Json(serde_json::error::Error),
}

type Result<T> = std::result::Result<T, DeserializationError>;

impl de::Error for DeserializationError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeserializationError::Message(msg.to_string())
    }
}
