//! The local_file module implements the `PlatformDataProvider` trait for gathering userdata from
//! local file

use super::{PlatformDataProvider, SettingsJson};
use snafu::{OptionExt, ResultExt};
use std::fs;

pub(crate) struct LocalFileDataProvider;

impl LocalFileDataProvider {
    pub(crate) const USER_DATA_FILE: &'static str = "/etc/early-boot-config/user-data";
}

impl PlatformDataProvider for LocalFileDataProvider {
    fn platform_data(&self) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();
        info!("'{}' exists, using it", Self::USER_DATA_FILE);

        let user_data_str =
            fs::read_to_string(Self::USER_DATA_FILE).context(error::InputFileRead {
                path: Self::USER_DATA_FILE,
            })?;

        if user_data_str.is_empty() {
            return Ok(output);
        }

        // Remove outer "settings" layer before sending to API
        let mut val: toml::Value =
            toml::from_str(&user_data_str).context(error::TOMLUserDataParse)?;
        let table = val.as_table_mut().context(error::UserDataNotTomlTable)?;
        let inner = table
            .remove("settings")
            .context(error::UserDataMissingSettings)?;

        let json = SettingsJson::from_val(&inner, "user data").context(error::SettingsToJSON)?;
        output.push(json);

        Ok(output)
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("Error serializing TOML to JSON: {}", source))]
        SettingsToJSON { source: serde_json::error::Error },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("TOML data did not contain 'settings' section"))]
        UserDataMissingSettings,

        #[snafu(display("Data is not a TOML table"))]
        UserDataNotTomlTable,
    }
}
