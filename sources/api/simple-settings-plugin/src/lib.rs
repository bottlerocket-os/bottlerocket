/*!
  This crate contains a simple settings plugin for unit tests.
*/

use bottlerocket_settings_plugin::SettingsPlugin;
use model_derive::model;

#[derive(SettingsPlugin)]
#[model(rename = "settings", impl_default = true)]
struct SimpleSettings {
    motd: settings_extension_motd::MotdV1,
    ntp: settings_extension_ntp::NtpSettingsV1,
}
