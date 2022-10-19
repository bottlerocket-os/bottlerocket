//! The local_file module provides a method for gathering userdata from local file

use super::SettingsJson;
use crate::compression::expand_file_maybe;
use snafu::ResultExt;
use std::path::Path;

pub(crate) const USER_DATA_FILE: &str = "/var/lib/bottlerocket/user-data.toml";

pub(crate) fn local_file_user_data(
) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
    if !Path::new(USER_DATA_FILE).exists() {
        return Ok(None);
    }
    info!("'{}' exists, using it", USER_DATA_FILE);

    // Read the file, decompressing it if compressed.
    let user_data_str = expand_file_maybe(USER_DATA_FILE).context(error::InputFileReadSnafu {
        path: USER_DATA_FILE,
    })?;

    if user_data_str.is_empty() {
        return Ok(None);
    }

    let json = SettingsJson::from_toml_str(&user_data_str, "user data").context(
        error::SettingsToJSONSnafu {
            from: USER_DATA_FILE,
        },
    )?;

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
