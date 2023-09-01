//! The metal module implements the `PlatformDataProvider` trait for gathering userdata on bare
//! metal.

use super::{PlatformDataProvider, SettingsJson};
use async_trait::async_trait;

use crate::provider::local_file;

pub(crate) struct MetalDataProvider;

#[async_trait]
impl PlatformDataProvider for MetalDataProvider {
    async fn platform_data(
        &self,
    ) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();

        // First read from any site-local defaults. It's unlikely that this file will exist, since
        // for bare metal provisioning these settings could just be written to the main user data
        // file, but this is consistent with other platforms.
        match local_file::user_data_defaults()? {
            Some(s) => output.push(s),
            None => info!(
                "No user data found via site defaults file: {}",
                local_file::USER_DATA_DEFAULTS_FILE
            ),
        }

        // This is the main file where we expect settings, so warn if they're not found.
        match local_file::user_data()? {
            Some(s) => output.push(s),
            None => warn!(
                "No user data found via local file: {}",
                local_file::USER_DATA_FILE
            ),
        }

        // Finally, apply any site-local overrides.
        match local_file::user_data_overrides()? {
            Some(s) => output.push(s),
            None => info!(
                "No user data found via site overrides file: {}",
                local_file::USER_DATA_OVERRIDES_FILE
            ),
        }

        Ok(output)
    }
}
