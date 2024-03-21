/*!
# Introduction

User data provider binary used to generate user data from data in the EC2 instance identity document.

Currently used only to fetch the AWS region. Falls back to IMDS if the region is not found in the instance identity document.
*/

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
