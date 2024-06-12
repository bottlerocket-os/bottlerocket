use bottlerocket_settings_sdk::{BottlerocketSetting, NullMigratorExtensionBuilder};
use settings_extension_container_runtime::ContainerRuntimeSettingsV1;
use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    match NullMigratorExtensionBuilder::with_name("container-runtime")
        .with_models(vec![
            BottlerocketSetting::<ContainerRuntimeSettingsV1>::model(),
        ])
        .build()
    {
        Ok(extension) => extension.run(),
        Err(e) => {
            println!("{}", e);
            ExitCode::FAILURE
        }
    }
}
