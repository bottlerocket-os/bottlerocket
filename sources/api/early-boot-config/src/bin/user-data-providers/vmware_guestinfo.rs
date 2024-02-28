#[cfg(target_arch = "x86_64")]
use early_boot_config::provider::VmwareGuestinfo;
use std::process::ExitCode;
#[cfg(target_arch = "x86_64")]
use user_data_provider::provider::{run_userdata_provider, setup_provider_logging};

#[tokio::main]
async fn main() -> ExitCode {
    #[cfg(target_arch = "x86_64")]
    {
        setup_provider_logging();
        run_userdata_provider(&VmwareGuestinfo).await
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        println!("");
        ExitCode::SUCCESS
    }
}
