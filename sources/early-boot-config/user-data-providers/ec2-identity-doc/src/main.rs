use early_boot_config_provider::provider::{
    print_userdata_output, setup_provider_logging, AsyncUserDataProvider,
};
use ec2_identity_doc_user_data_provider::Ec2IdentityDoc;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    print_userdata_output(Ec2IdentityDoc.user_data().await)
}
