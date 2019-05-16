//! The server module owns the API surface and interfaces with the datastore.

mod controller;
#[macro_use]
mod rouille_ext;
mod router;

pub use router::handle_request;

use std::io;

use crate::datastore::deserialization::DeserializationError;
use crate::datastore::serialization::SerializationError;
use crate::datastore::DataStoreError;
use crate::IoErrorDetail;

/// Potential errors from the API surface.
#[derive(Debug, Error)]
pub(crate) enum ServerError {
    #[error(msg_embedded, no_from, non_std)]
    /// Server invariant violation
    Internal(String),

    /// Error in data store
    DataStore(DataStoreError),

    /// Error serializing request to datastore keys
    Serialization(SerializationError),

    /// JSON error interpreting value
    Json(serde_json::error::Error),

    /// Error populating response from data
    Deserialization(DeserializationError),

    #[error(msg_embedded, no_from, non_std)]
    /// User did not specify a required input
    MissingInput(String),

    #[error(msg_embedded, no_from, non_std)]
    /// User specified an invalid input
    InvalidInput(String),

    #[error(msg_embedded, no_from, non_std)]
    /// Error applying settings
    Io(IoErrorDetail),
}

type Result<T> = std::result::Result<T, ServerError>;

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        ServerError::Io(IoErrorDetail::new("".to_string(), err))
    }
}
