use early_boot_config_provider::provider::{run_userdata_provider, setup_provider_logging};
use local_overrides_user_data_provider::LocalOverrides;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&LocalOverrides).await
}
