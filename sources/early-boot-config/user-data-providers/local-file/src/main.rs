use early_boot_config_provider::provider::{run_userdata_provider, setup_provider_logging};
use local_file_user_data_provider::LocalUserData;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&LocalUserData).await
}
