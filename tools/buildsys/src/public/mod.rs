/// Public types that can be used in Bottlerocket `build.rs` files and by other build and test
/// related tooling.
pub(crate) mod error;
mod readme;
mod variant;

pub use error::{Error, Result};
pub use readme::{generate_readme, ReadmeSource};
pub use variant::{Variant, DEFAULT_VARIANT_TYPE, DEFAULT_VARIANT_VERSION, VARIANT_ENV};
