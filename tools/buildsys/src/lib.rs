/// Public types that can be used in Bottlerocket `build.rs` files and by other build and test
/// related tooling. See the `public` module directory.
mod public;

pub use public::{
    generate_readme, Error, ReadmeSource, Result, Variant, DEFAULT_VARIANT_TYPE,
    DEFAULT_VARIANT_VERSION, VARIANT_ENV,
};
