use super::validate_addressing;
use super::{error, Dhcp4ConfigV1, Dhcp6ConfigV1, Result, RouteV1, StaticConfigV1, Validate};
use crate::bonding::{
    ArpMonitoringConfigV1, BondModeV1, BondMonitoringConfigV1, MiiMonitoringConfigV1,
};
use crate::interface_id::InterfaceName;
use crate::net_config::devices::generate_addressing_validation;
use serde::Deserialize;
use snafu::ensure;

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct NetBondV1 {
    pub(crate) primary: Option<bool>,
    pub(crate) dhcp4: Option<Dhcp4ConfigV1>,
    pub(crate) dhcp6: Option<Dhcp6ConfigV1>,
    pub(crate) static4: Option<StaticConfigV1>,
    pub(crate) static6: Option<StaticConfigV1>,
    #[serde(rename = "route")]
    pub(crate) routes: Option<Vec<RouteV1>>,
    #[serde(rename = "kind")]
    _kind: BondKind,
    pub(crate) mode: BondModeV1,
    #[serde(rename = "min-links")]
    pub(crate) min_links: Option<usize>,
    #[serde(rename = "monitoring")]
    pub(crate) monitoring_config: BondMonitoringConfigV1,
    pub(crate) interfaces: Vec<InterfaceName>,
}

// Single variant enum only used to direct deserialization.  If the kind is not "Bond" or "bond",
// deserialization will fail.
#[derive(Debug, Deserialize, Clone)]
enum BondKind {
    #[serde(alias = "bond")]
    Bond,
}

generate_addressing_validation!(&NetBondV1);

impl Validate for NetBondV1 {
    fn validate(&self) -> Result<()> {
        validate_addressing(self)?;

        // TODO: We should move this and other validation logic into Deserialize when messaging
        // is better for enum failures https://github.com/serde-rs/serde/issues/2157
        let interfaces_count = self.interfaces.len();
        ensure!(
            interfaces_count > 0,
            error::InvalidNetConfigSnafu {
                reason: "bonds must have 1 or more interfaces specified"
            }
        );
        if let Some(min_links) = self.min_links {
            ensure!(
                min_links <= interfaces_count,
                error::InvalidNetConfigSnafu {
                    reason: "min-links is greater than number of interfaces configured"
                }
            )
        }
        // Validate monitoring configuration
        match &self.monitoring_config {
            BondMonitoringConfigV1::MiiMon(config) => config.validate()?,
            BondMonitoringConfigV1::ArpMon(config) => config.validate()?,
        }

        Ok(())
    }
}

impl Validate for MiiMonitoringConfigV1 {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.frequency > 0,
            error::InvalidNetConfigSnafu {
                reason: "miimon-frequency-ms of 0 disables Mii Monitoring, either set a value or configure Arp Monitoring"
            }
        );
        // updelay and downdelay should be a multiple of frequency, but will be rounded down
        // by the kernel, this ensures they are at least the size of frequency (non-zero)
        ensure!(
            self.frequency <= self.updelay && self.frequency <= self.downdelay,
            error::InvalidNetConfigSnafu {
                reason: "miimon-updelay-ms and miimon-downdelay-ms must be equal to or larger than miimon-frequency-ms"
            }
        );
        Ok(())
    }
}

impl Validate for ArpMonitoringConfigV1 {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.interval > 0,
            error::InvalidNetConfigSnafu {
                reason: "arpmon-interval-ms of 0 disables Arp Monitoring, either set a value or configure Mii Monitoring"
            }
        );
        // If using Arp Monitoring, 1-16 targets must be specified
        let targets_length: u32 = self.targets.len() as u32;
        ensure!(
            targets_length > 0 && targets_length <= 16,
            error::InvalidNetConfigSnafu {
                reason: "arpmon-targets must include between 1 and 16 targets"
            }
        );
        Ok(())
    }
}
