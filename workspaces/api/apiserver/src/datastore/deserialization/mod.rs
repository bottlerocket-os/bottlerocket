//! The deserialization module implements generic deserialization techniques that are particularly
//! useful for populating Rust structures from the datastore.

mod error;
mod pairs;

pub use error::{Error, Result};
pub use pairs::{from_map, from_map_with_prefix};
