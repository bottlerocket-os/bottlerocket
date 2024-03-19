/*!
# Introduction

User data provider binary used to fetch the user data overrides provided in the file tree under `/local/user-data-overrides.toml`.
*/

use early_boot_config_provider::provider::{
    print_userdata_output, setup_provider_logging, UserDataProvider,
};
use local_overrides_user_data_provider::LocalOverrides;
use std::process::ExitCode;

fn main() -> ExitCode {
    setup_provider_logging();
    print_userdata_output(LocalOverrides.user_data())
}
