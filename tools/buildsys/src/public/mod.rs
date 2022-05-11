/// Public types that can be used in Bottlerocket `build.rs` files and by other build and test
/// related tooling.
mod error;
mod variant;

pub use error::{Error, Result};
pub use variant::{Variant, DEFAULT_VARIANT_TYPE, DEFAULT_VARIANT_VERSION, VARIANT_ENV};
