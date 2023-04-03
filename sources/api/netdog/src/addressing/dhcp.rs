use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Dhcp4ConfigV1 {
    DhcpEnabled(bool),
    WithOptions(Dhcp4OptionsV1),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Dhcp4OptionsV1 {
    pub(crate) enabled: bool,
    pub(crate) optional: Option<bool>,
    #[serde(rename = "route-metric")]
    pub(crate) route_metric: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Dhcp6ConfigV1 {
    DhcpEnabled(bool),
    WithOptions(Dhcp6OptionsV1),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Dhcp6OptionsV1 {
    pub(crate) enabled: bool,
    pub(crate) optional: Option<bool>,
}
