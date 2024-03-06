/// Local user data file
use async_trait::async_trait;
use early_boot_config_provider::provider::{user_data_from_file, UserDataProvider};
use early_boot_config_provider::settings::SettingsJson;

const LOCAL_USER_DATA: &str = "/var/lib/bottlerocket/user-data.toml";

pub struct LocalUserData;

#[async_trait]
impl UserDataProvider for LocalUserData {
    async fn user_data(&self) -> Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        user_data_from_file(LOCAL_USER_DATA)
    }
}
