use early_boot_config_provider::provider::UserDataProvider;
use early_boot_config_provider::settings::SettingsJson;

pub struct VmwareGuestinfo;

impl UserDataProvider for VmwareGuestinfo {
    #[allow(dead_code)]
    fn user_data(&self) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
