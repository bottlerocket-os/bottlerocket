//! The metal module implements the `PlatformDataProvider` trait for gathering userdata on bare
//! metal.

use super::{PlatformDataProvider, SettingsJson};
use async_trait::async_trait;

use crate::provider::local_file::{local_file_user_data, USER_DATA_FILE};

pub(crate) struct MetalDataProvider;

#[async_trait]
impl PlatformDataProvider for MetalDataProvider {
    async fn platform_data(
        &self,
    ) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();

        match local_file_user_data()? {
            Some(s) => output.push(s),
            None => warn!("No user data found via local file: {}", USER_DATA_FILE),
        }

        Ok(output)
    }
}
