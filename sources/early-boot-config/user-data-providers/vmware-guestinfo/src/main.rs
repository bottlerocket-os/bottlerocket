use early_boot_config_provider::provider::{run_userdata_provider, setup_provider_logging};
use std::process::ExitCode;
use vmware_guestinfo_user_data_provider::VmwareGuestinfo;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&VmwareGuestinfo).await
}
