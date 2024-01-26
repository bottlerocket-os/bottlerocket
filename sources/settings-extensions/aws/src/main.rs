use bottlerocket_settings_sdk::{BottlerocketSetting, NullMigratorExtensionBuilder};
use settings_extension_aws::AwsSettingsV1;
use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    match NullMigratorExtensionBuilder::with_name("aws")
        .with_models(vec![BottlerocketSetting::<AwsSettingsV1>::model()])
        .build()
    {
        Ok(extension) => extension.run(),
        Err(e) => {
            println!("{}", e);
            ExitCode::FAILURE
        }
    }
}
