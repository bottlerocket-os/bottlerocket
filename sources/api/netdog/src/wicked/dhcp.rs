use crate::addressing::{Dhcp4ConfigV1, Dhcp4OptionsV1, Dhcp6ConfigV1, Dhcp6OptionsV1};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WickedDhcp4 {
    #[serde(rename = "$unflatten=enabled")]
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=route-priority")]
    route_priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=defer-timeout")]
    defer_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<AddrConfFlags>,
}

impl Default for WickedDhcp4 {
    fn default() -> Self {
        WickedDhcp4 {
            enabled: true,
            route_priority: None,
            defer_timeout: None,
            flags: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WickedDhcp6 {
    #[serde(rename = "$unflatten=enabled")]
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$unflatten=defer-timeout")]
    defer_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<AddrConfFlags>,
}

impl Default for WickedDhcp6 {
    fn default() -> Self {
        WickedDhcp6 {
            enabled: true,
            defer_timeout: None,
            flags: None,
        }
    }
}

// This is technically an enum, but considering we don't expose anything other than "optional" to
// the user, a struct makes handling tags much simpler.
#[derive(Default, Clone, Debug, Serialize, PartialEq)]
struct AddrConfFlags {
    #[serde(rename = "$unflatten=optional")]
    optional: (),
}

impl From<Dhcp4ConfigV1> for WickedDhcp4 {
    fn from(dhcp4: Dhcp4ConfigV1) -> Self {
        match dhcp4 {
            Dhcp4ConfigV1::DhcpEnabled(b) => WickedDhcp4 {
                enabled: b,
                ..Default::default()
            },
            Dhcp4ConfigV1::WithOptions(o) => WickedDhcp4::from(o),
        }
    }
}

impl From<Dhcp4OptionsV1> for WickedDhcp4 {
    fn from(options: Dhcp4OptionsV1) -> Self {
        let mut defer_timeout = None;
        let mut flags = None;

        if options.optional == Some(true) {
            defer_timeout = Some(1);
            flags = Some(AddrConfFlags::default());
        }

        WickedDhcp4 {
            enabled: options.enabled,
            route_priority: options.route_metric,
            defer_timeout,
            flags,
        }
    }
}

impl From<Dhcp6ConfigV1> for WickedDhcp6 {
    fn from(dhcp6: Dhcp6ConfigV1) -> Self {
        match dhcp6 {
            Dhcp6ConfigV1::DhcpEnabled(b) => WickedDhcp6 {
                enabled: b,
                ..Default::default()
            },
            Dhcp6ConfigV1::WithOptions(o) => WickedDhcp6::from(o),
        }
    }
}

impl From<Dhcp6OptionsV1> for WickedDhcp6 {
    fn from(options: Dhcp6OptionsV1) -> Self {
        let mut defer_timeout = None;
        let mut flags = None;

        if options.optional == Some(true) {
            defer_timeout = Some(1);
            flags = Some(AddrConfFlags::default());
        }

        WickedDhcp6 {
            enabled: options.enabled,
            defer_timeout,
            flags,
        }
    }
}
