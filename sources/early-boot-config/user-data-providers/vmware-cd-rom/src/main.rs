use early_boot_config_provider::provider::{
    print_userdata_output, setup_provider_logging, UserDataProvider,
};
use std::process::ExitCode;
use vmware_cd_rom_user_data_provider::VmwareCdRom;

fn main() -> ExitCode {
    setup_provider_logging();
    print_userdata_output(VmwareCdRom.user_data())
}
