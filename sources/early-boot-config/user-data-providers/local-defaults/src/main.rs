/*!
# Introduction

User data provider binary used to fetch the default user data provided in the file tree under `/local/user-data-defaults.toml`.
*/

use early_boot_config_provider::provider::{
    print_userdata_output, setup_provider_logging, UserDataProvider,
};
use local_defaults_user_data_provider::LocalDefaults;
use std::process::ExitCode;

fn main() -> ExitCode {
    setup_provider_logging();
    print_userdata_output(LocalDefaults.user_data())
}
