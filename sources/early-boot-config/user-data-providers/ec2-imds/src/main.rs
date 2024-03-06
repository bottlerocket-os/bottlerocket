use early_boot_config_provider::provider::{
    print_userdata_output, setup_provider_logging, AsyncUserDataProvider,
};
use ec2_imds_user_data_provider::Ec2Imds;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    print_userdata_output(Ec2Imds.user_data().await)
}
