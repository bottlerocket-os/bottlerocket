use ec2_identity_doc_user_data_provider::Ec2IdentityDoc;
use std::process::ExitCode;
use user_data_provider::provider::{run_userdata_provider, setup_provider_logging};

#[tokio::main]
async fn main() -> ExitCode {
    setup_provider_logging();
    run_userdata_provider(&Ec2IdentityDoc).await
}
