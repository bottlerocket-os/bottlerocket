/*!
  This crate contains constants shared across multiple Bottlerocket crates
*/

// Shared API settings
pub const API_SOCKET: &str = "/run/api.sock";
pub const API_SETTINGS_URI: &str = "/settings";
pub const API_SETTINGS_GENERATORS_URI: &str = "/metadata/setting-generators";

// Shared transaction used by boot time services
pub const LAUNCH_TRANSACTION: &str = "bottlerocket-launch";

// Shared binaries' locations
pub const SYSTEMCTL_BIN: &str = "/bin/systemctl";
pub const HOST_CTR_BIN: &str = "/bin/host-ctr";
