use bottlerocket_settings_sdk::{BottlerocketSetting, LinearMigratorExtensionBuilder};
use settings_extension_ntp::NtpSettingsV1;
use std::process::ExitCode;

fn main() -> ExitCode {
    match LinearMigratorExtensionBuilder::with_name("ntp")
        .with_models(vec![BottlerocketSetting::<NtpSettingsV1>::model()])
        .build()
    {
        Ok(extension) => extension.run(),
        Err(e) => {
            println!("{}", e);
            ExitCode::FAILURE
        }
    }
}
