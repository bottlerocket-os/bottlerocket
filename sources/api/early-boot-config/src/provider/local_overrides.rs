/// Site-local overrides
use async_trait::async_trait;
use user_data_provider::provider::{user_data_from_file, UserDataProvider};
use user_data_provider::settings::SettingsJson;

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
