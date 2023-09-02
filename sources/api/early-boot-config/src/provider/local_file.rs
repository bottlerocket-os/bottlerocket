//! The local_file module provides a method for gathering userdata from local file

use super::SettingsJson;
use crate::compression::expand_file_maybe;
use snafu::ResultExt;
use std::path::Path;

pub(crate) const USER_DATA_FILE: &str = "/var/lib/bottlerocket/user-data.toml";
pub(crate) const USER_DATA_DEFAULTS_FILE: &str = "/local/user-data-defaults.toml";
pub(crate) const USER_DATA_OVERRIDES_FILE: &str = "/local/user-data-overrides.toml";

pub(crate) fn user_data() -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
    read_from_file(USER_DATA_FILE)
}

pub(crate) fn user_data_defaults(
) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
    read_from_file(USER_DATA_DEFAULTS_FILE)
}

pub(crate) fn user_data_overrides(
) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
    read_from_file(USER_DATA_OVERRIDES_FILE)
}

fn read_from_file(
    path: &str,
) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Ok(None);
    }
    info!("'{path}' exists, using it");

    // Read the file, decompressing it if compressed.
    let user_data_str = expand_file_maybe(path).context(error::InputFileReadSnafu { path })?;

    if user_data_str.is_empty() {
        return Ok(None);
    }

    let json = SettingsJson::from_toml_str(&user_data_str, "user data")
        .context(error::SettingsToJSONSnafu { from: path })?;

    Ok(Some(json))
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to serialize settings from {}: {}", from, source))]
        SettingsToJSON {
            from: String,
            source: crate::settings::Error,
        },
    }
}
