/// Site-local defaults
use super::{user_data_from_file, UserDataProvider};
use crate::settings::SettingsJson;
use async_trait::async_trait;

const LOCAL_DEFAULTS_FILE: &str = "/local/user-data-defaults.toml";

pub struct LocalDefaults;

#[async_trait]
impl UserDataProvider for LocalDefaults {
    async fn user_data(&self) -> Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        user_data_from_file(LOCAL_DEFAULTS_FILE)
    }
}
