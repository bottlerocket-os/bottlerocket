//! The nutanix module implements the `PlatformDataProvider` trait for gathering userdata on Nutanix
//! via Configdrive

use super::{PlatformDataProvider, SettingsJson};
use crate::compression::expand_file_maybe;
use async_trait::async_trait;
use serde_json::Value;
use snafu::ResultExt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;

use crate::provider::local_file::{local_file_user_data, USER_DATA_FILE};

pub(crate) struct NutanixDataProvider;

impl NutanixDataProvider {
    // This program expects that the CONFIGDRIVE is already mounted.  Mounting happens elsewhere in a
    // systemd unit file
    const CONFIGDRIVE_MOUNT: &'static str = "/media/cdrom/openstack/latest/";
    // A mounted CONFIGDRIVE contain a user-supplied file named `user-data`
    const USER_DATA_FILENAME: &'static str = "user_data";
    // A mounted CONFIGDRIVE contain a platform-supplied file named `meta_data.json`
    const META_DATA_FILENAME: &'static str = "meta_data.json";

    /// Read and decode meta data from file via mounted CONFIGDRIVE
    fn configdrive_meta_data() -> Result<Option<SettingsJson>> {
        info!("Attempting to retrieve hostname in meta data from mounted Configdrive");
        let meta_data_file = Path::new(Self::CONFIGDRIVE_MOUNT).join(Self::META_DATA_FILENAME);

        if !meta_data_file.exists() {
            return Ok(None);
        }

        info!("'{}' exists, using it", meta_data_file.display());

        // Read the content of metadata file
        let mut file = match File::open(meta_data_file) {
            Ok(file) => file,
            Err(_) => return Ok(None),
        };
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_err() {
            return Ok(None);
        }

        // Decode the json metadata file
        let v: Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        // Serch for hostname key
        let hostname = if let Some(hostname_value) = v["hostname"].as_str() {
            hostname_value
        } else {
            return Ok(None);
        };

        // Build hostname response
        let hostname_str = format!("[settings.network]\nhostname = \"{}\"", hostname);
        let json =
            SettingsJson::from_toml_str(&hostname_str, "hostname from meta data in Configdrive")
                .context(error::SettingsToJsonSnafu { from: hostname_str })?;

        Ok(Some(json))
    }

    /// Read and decode user data from file via mounted CONFIGDRIVE
    fn configdrive_user_data() -> Result<Option<SettingsJson>> {
        info!("Attempting to retrieve user data from mounted Configdrive");
        let user_data_file = Path::new(Self::CONFIGDRIVE_MOUNT).join(Self::USER_DATA_FILENAME);

        if !user_data_file.exists() {
            return Ok(None);
        }

        info!("'{}' exists, using it", user_data_file.display());
        let user_data_str = {
            // Read the file, decompressing it if compressed.
            expand_file_maybe(&user_data_file).context(error::InputFileReadSnafu {
                path: &user_data_file,
            })?
        };

        if user_data_str.is_empty() {
            return Ok(None);
        }

        let json = SettingsJson::from_toml_str(&user_data_str, "user data from Configdrive")
            .context(error::SettingsToJsonSnafu {
                from: user_data_file.display().to_string(),
            })?;

        Ok(Some(json))
    }
}

#[async_trait]
impl PlatformDataProvider for NutanixDataProvider {
    async fn platform_data(
        &self,
    ) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();

        // Attempt to read from local file first
        match local_file_user_data()? {
            Some(s) => output.push(s),
            None => warn!("No user data found via local file: {}", USER_DATA_FILE),
        }

        // Then look at the Configdrive for meta data hostname
        match Self::configdrive_meta_data()? {
            Some(s) => output.push(s),
            None => warn!("No hostname found in meta data via Configdrive"),
        }

        // Then look at the Configdrive for user data
        match Self::configdrive_user_data()? {
            Some(s) => output.push(s),
            None => warn!("No user data found via Configdrive"),
        }

        Ok(output)
    }
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
        SettingsToJson {
            from: String,
            source: crate::settings::Error,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
