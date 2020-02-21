#![deny(rust_2018_idioms)]

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regular expression that will match migration file names and allow retrieving the
    /// version and name components.
    // Note: the version component is a simplified semver regex; we don't use any of the
    // extensions, just a simple x.y.z, so this isn't as strict as it could be.
    pub static ref MIGRATION_FILENAME_RE: Regex =
        Regex::new(r"(?x)^
                   migrate
                   _
                   v?  # optional 'v' prefix for humans
                   (?P<version>[0-9]+\.[0-9]+\.[0-9]+[0-9a-zA-Z+-]*)
                   _
                   (?P<name>[a-zA-Z0-9-]+)
                   $").unwrap();
}
