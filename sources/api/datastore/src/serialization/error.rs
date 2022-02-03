use serde::ser;
use snafu::{IntoError, NoneError as NoSource, Snafu};

use crate::ScalarError;

/// Potential errors from serialization.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    // This error variant is required to implement ser::Error for serde.
    #[snafu(display("Error during serialization: {}", msg))]
    Message { msg: String },

    #[snafu(display("Serializer logic error: {}", msg))]
    Internal { msg: String },

    #[snafu(display("Error creating valid datastore key: {}", msg))]
    InvalidKey {
        // "msg" instead of just key name because we want to include the data store error
        // message, but can't have it as a "source" because that'd be circular
        msg: String,
    },

    #[snafu(display("Tried to output concrete value without prefix; value: {}", value))]
    MissingPrefix { value: String },

    #[snafu(display("Error serializing {}: {} ", given, source))]
    Serialization { given: String, source: ScalarError },

    #[snafu(display("Error deserializing {}: {} ", given, source))]
    Deserialization { given: String, source: ScalarError },

    #[snafu(display("'{}' not allowed by Serializer", typename))]
    InvalidType { typename: String },

    #[snafu(display("'{}' not allowed as map key", typename))]
    BadMapKey { typename: String },
}

pub type Result<T> = std::result::Result<T, Error>;

impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        MessageSnafu {
            msg: msg.to_string(),
        }
        .into_error(NoSource)
    }
}
