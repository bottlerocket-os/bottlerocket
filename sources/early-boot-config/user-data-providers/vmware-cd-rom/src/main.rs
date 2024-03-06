use early_boot_config_provider::provider::{run_userdata_provider, setup_provider_logging};
use std::process::ExitCode;
use vmware_cd_rom_user_data_provider::VmwareCdRom;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&VmwareCdRom).await
}
