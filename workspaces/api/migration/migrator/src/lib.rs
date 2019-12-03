#![deny(rust_2018_idioms)]

use data_store_version::VERSION_RE;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regular expression that will match migration file names and allow retrieving the
    /// version and name components.
    pub static ref MIGRATION_FILENAME_RE: Regex =
        Regex::new(&format!(r"^migrate_{}_(?P<name>[a-zA-Z0-9-]+)$", *VERSION_RE)).unwrap();
}
