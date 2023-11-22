use early_boot_config::provider::{run_userdata_provider, setup_provider_logging, LocalOverrides};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&LocalOverrides).await
}
