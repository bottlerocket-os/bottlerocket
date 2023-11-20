/// Local user data file
use super::{user_data_from_file, UserDataProvider};
use crate::settings::SettingsJson;
use async_trait::async_trait;

const LOCAL_USER_DATA: &str = "/var/lib/bottlerocket/user-data.toml";

pub struct LocalUserData;

#[async_trait]
impl UserDataProvider for LocalUserData {
    async fn user_data(&self) -> Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        user_data_from_file(LOCAL_USER_DATA)
    }
}
