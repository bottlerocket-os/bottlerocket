use early_boot_config::provider::{run_userdata_provider, setup_provider_logging, Ec2IdentityDoc};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&Ec2IdentityDoc).await
}
