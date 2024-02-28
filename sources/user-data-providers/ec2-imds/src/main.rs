use ec2_imds_user_data_provider::Ec2Imds;
use std::process::ExitCode;
use user_data_provider::provider::{run_userdata_provider, setup_provider_logging};

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&Ec2Imds).await
}
