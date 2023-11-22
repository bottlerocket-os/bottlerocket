/// Site-local overrides
use super::{user_data_from_file, UserDataProvider};
use crate::settings::SettingsJson;
use async_trait::async_trait;

const LOCAL_OVERRIDES: &str = "/local/user-data-overrides.toml";

pub struct LocalOverrides;

#[async_trait]
impl UserDataProvider for LocalOverrides {
    async fn user_data(
        &self,
    ) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        user_data_from_file(LOCAL_OVERRIDES)
    }
}
