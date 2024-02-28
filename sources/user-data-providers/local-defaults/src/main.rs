use local_defaults_user_data_provider::LocalDefaults;
use std::process::ExitCode;
use user_data_provider::provider::{run_userdata_provider, setup_provider_logging};

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&LocalDefaults).await
}
