#[cfg(target_arch = "x86_64")]
use early_boot_config::provider::{run_userdata_provider, setup_provider_logging, VmwareGuestinfo};
use std::process::ExitCode;

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
