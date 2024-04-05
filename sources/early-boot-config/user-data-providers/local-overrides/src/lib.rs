/// Site-local overrides
use early_boot_config_provider::provider::{user_data_from_file, UserDataProvider};
use early_boot_config_provider::settings::SettingsJson;

const LOCAL_OVERRIDES: &str = "/local/user-data-overrides.toml";

pub struct LocalOverrides;

impl UserDataProvider for LocalOverrides {
    fn user_data(&self) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        user_data_from_file(LOCAL_OVERRIDES)
    }
}
