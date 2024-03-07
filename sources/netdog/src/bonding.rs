//! The bonding module contains the config structures specific to network bonds.

use serde::Deserialize;
use std::net::IpAddr;

// Currently only mode 1 (active-backup) is supported but eventually 0-6 could be added
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BondModeV1 {
    ActiveBackup,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum BondMonitoringConfigV1 {
    MiiMon(MiiMonitoringConfigV1),
    ArpMon(ArpMonitoringConfigV1),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MiiMonitoringConfigV1 {
    #[serde(rename = "miimon-frequency-ms")]
    pub(crate) frequency: u32,
    #[serde(rename = "miimon-updelay-ms")]
    pub(crate) updelay: u32,
    #[serde(rename = "miimon-downdelay-ms")]
    pub(crate) downdelay: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArpMonitoringConfigV1 {
    #[serde(rename = "arpmon-interval-ms")]
    pub(crate) interval: u32,
    #[serde(rename = "arpmon-validate")]
    pub(crate) validate: ArpValidateV1,
    #[serde(rename = "arpmon-targets")]
    pub(crate) targets: Vec<IpAddr>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ArpValidateV1 {
    Active,
    All,
    Backup,
    None,
}
