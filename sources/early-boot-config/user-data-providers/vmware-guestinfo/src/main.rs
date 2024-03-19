/*!
# Introduction

User data provider binary used to fetch user data provided via VMWare guestinfo.
*/

use early_boot_config_provider::provider::{
    print_userdata_output, setup_provider_logging, UserDataProvider,
};
use std::process::ExitCode;
use vmware_guestinfo_user_data_provider::VmwareGuestinfo;

fn main() -> ExitCode {
    setup_provider_logging();
    print_userdata_output(VmwareGuestinfo.user_data())
}
