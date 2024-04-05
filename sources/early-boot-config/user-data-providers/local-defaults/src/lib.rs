/// Site-local defaults
use early_boot_config_provider::provider::{user_data_from_file, UserDataProvider};
use early_boot_config_provider::settings::SettingsJson;

const LOCAL_DEFAULTS_FILE: &str = "/local/user-data-defaults.toml";

pub struct LocalDefaults;

impl UserDataProvider for LocalDefaults {
    fn user_data(&self) -> Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        user_data_from_file(LOCAL_DEFAULTS_FILE)
    }
}
