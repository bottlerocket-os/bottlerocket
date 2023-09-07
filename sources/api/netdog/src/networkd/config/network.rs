use super::private::{
    Bond, BondWorker, CanHaveVlans, Device, Interface, NotBonded, Vlan, VlanLink,
};
use super::CONFIG_FILE_PREFIX;
use crate::addressing::{Dhcp4ConfigV1, Dhcp6ConfigV1, RouteTo, RouteV1, StaticConfigV1};
use crate::interface_id::InterfaceId;
use crate::interface_id::{InterfaceName, MacAddress};
use crate::networkd::{error, Result};
use ipnet::IpNet;
use lazy_static::lazy_static;
use snafu::{OptionExt, ResultExt};
use std::fmt::Display;
use std::fs;
use std::marker::PhantomData;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use systemd_derive::{SystemdUnit, SystemdUnitSection};

lazy_static! {
    static ref DEFAULT_ROUTE_IPV4: IpNet = "0.0.0.0/0".parse().unwrap();
    static ref DEFAULT_ROUTE_IPV6: IpNet = "::/0".parse().unwrap();
}

#[derive(Debug, Default, SystemdUnit)]
pub(crate) struct NetworkConfig {
    r#match: Option<MatchSection>,
    link: Option<LinkSection>,
    network: Option<NetworkSection>,
    route: Vec<RouteSection>,
    dhcp4: Option<Dhcp4Section>,
    dhcp6: Option<Dhcp6Section>,
    ipv6_accept_ra: Option<Ipv6AcceptRaSection>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Match")]
struct MatchSection {
    #[systemd(entry = "Name")]
    name: Option<InterfaceName>,
    #[systemd(entry = "PermanentMACAddress")]
    permanent_mac_address: Vec<MacAddress>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Link")]
struct LinkSection {
    #[systemd(entry = "RequiredForOnline")]
    required: Option<bool>,
    #[systemd(entry = "RequiredFamilyForOnline")]
    required_family: Option<RequiredFamily>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Network")]
struct NetworkSection {
    #[systemd(entry = "Address")]
    addresses: Vec<IpNet>,
    #[systemd(entry = "Bond")]
    bond: Option<InterfaceName>,
    #[systemd(entry = "ConfigureWithoutCarrier")]
    configure_wo_carrier: Option<bool>,
    #[systemd(entry = "DHCP")]
    dhcp: Option<DhcpBool>,
    #[systemd(entry = "IPv6AcceptRA")]
    ipv6_accept_ra: Option<bool>,
    #[systemd(entry = "IPv6DuplicateAddressDetection")]
    ipv6_duplicate_address_detection: Option<i32>,
    #[systemd(entry = "LinkLocalAddressing")]
    link_local_addressing: Option<DhcpBool>,
    #[systemd(entry = "PrimarySlave")]
    primary_bond_worker: Option<bool>,
    #[systemd(entry = "VLAN")]
    vlan: Vec<InterfaceName>,
    #[systemd(entry = "KeepConfiguration")]
    keep_configuration: Option<KeepConfiguration>,
    #[systemd(entry = "BindCarrier")]
    bind_carrier: Vec<InterfaceName>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Route")]
struct RouteSection {
    #[systemd(entry = "Destination")]
    destination: Option<IpNet>,
    #[systemd(entry = "Gateway")]
    gateway: Option<IpAddr>,
    #[systemd(entry = "Metric")]
    metric: Option<u32>,
    #[systemd(entry = "PreferredSource")]
    preferred_source: Option<IpAddr>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "DHCPv4")]
struct Dhcp4Section {
    #[systemd(entry = "RouteMetric")]
    metric: Option<u32>,
    #[systemd(entry = "UseDNS")]
    use_dns: Option<bool>,
    #[systemd(entry = "UseDomains")]
    use_domains: Option<bool>,
    #[systemd(entry = "UseMTU")]
    use_mtu: Option<bool>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "DHCPv6")]
struct Dhcp6Section {
    #[systemd(entry = "UseDNS")]
    use_dns: Option<bool>,
    #[systemd(entry = "UseDomains")]
    use_domains: Option<bool>,
    #[systemd(entry = "WithoutRA")]
    without_ra: Option<WithoutRa>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "IPv6AcceptRA")]
struct Ipv6AcceptRaSection {
    #[systemd(entry = "UseDNS")]
    use_dns: Option<bool>,
    #[systemd(entry = "UseDomains")]
    use_domains: Option<bool>,
    #[systemd(entry = "UseMTU")]
    use_mtu: Option<bool>,
}

// The `Any` variant isn't currently used, but is valid
#[allow(dead_code)]
#[derive(Debug)]
enum RequiredFamily {
    Any,
    Both,
    Ipv4,
    Ipv6,
}

impl Display for RequiredFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredFamily::Any => write!(f, "any"),
            RequiredFamily::Both => write!(f, "both"),
            RequiredFamily::Ipv4 => write!(f, "ipv4"),
            RequiredFamily::Ipv6 => write!(f, "ipv6"),
        }
    }
}

#[derive(Debug)]
enum DhcpBool {
    Ipv4,
    Ipv6,
    No,
    Yes,
}

impl Display for DhcpBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DhcpBool::Ipv4 => write!(f, "ipv4"),
            DhcpBool::Ipv6 => write!(f, "ipv6"),
            DhcpBool::No => write!(f, "no"),
            DhcpBool::Yes => write!(f, "yes"),
        }
    }
}

// Only the `Solicit` variant is currently used.
#[allow(dead_code)]
#[derive(Debug)]
enum WithoutRa {
    No,
    Solicit,
    InformationRequest,
}

impl Display for WithoutRa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WithoutRa::No => write!(f, "no"),
            WithoutRa::Solicit => write!(f, "solicit"),
            WithoutRa::InformationRequest => write!(f, "information-request"),
        }
    }
}

// Only the `Dhcp` variant is currently used.
#[derive(Debug)]
#[allow(dead_code)]
enum KeepConfiguration {
    Yes,
    No,
    Dhcp,
    DhcpOnStop,
    Static,
}

impl Display for KeepConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeepConfiguration::Yes => write!(f, "yes"),
            KeepConfiguration::No => write!(f, "no"),
            KeepConfiguration::Dhcp => write!(f, "dhcp"),
            KeepConfiguration::DhcpOnStop => write!(f, "dhcp-on-stop"),
            KeepConfiguration::Static => write!(f, "static"),
        }
    }
}

impl NetworkConfig {
    const FILE_EXT: &str = "network";

    fn new_with_name(name: InterfaceName) -> Self {
        Self {
            r#match: Some(MatchSection {
                name: Some(name),
                permanent_mac_address: Vec::default(),
            }),
            ..Default::default()
        }
    }

    fn new_with_mac_address(mac: MacAddress) -> Self {
        Self {
            r#match: Some(MatchSection {
                name: None,
                permanent_mac_address: vec![mac],
            }),
            ..Default::default()
        }
    }

    /// The name of the device that corresponds to this config
    // This method is useful but is only used in tests so far
    #[cfg(test)]
    pub(crate) fn name(&self) -> Option<InterfaceId> {
        let maybe_name = self.r#match.as_ref().and_then(|m| m.name.as_ref());
        let maybe_mac = self
            .r#match
            .as_ref()
            .and_then(|m| m.permanent_mac_address.first());

        match (maybe_name, maybe_mac) {
            (Some(name), _) => Some(InterfaceId::from(name.clone())),
            (None, Some(mac)) => Some(InterfaceId::from(mac.clone())),
            (None, None) => None,
        }
    }

    /// Add config to accept IPv6 router advertisements
    // TODO: expose a network config option for this
    pub(crate) fn accept_ra(&mut self) {
        self.network_mut().ipv6_accept_ra = Some(true);
        self.dhcp6_mut().without_ra = Some(WithoutRa::Solicit);
    }

    /// Add config to disable IPv6 duplicate address detection
    // TODO: expose a network config option for this
    pub(crate) fn disable_dad(&mut self) {
        self.network_mut().ipv6_duplicate_address_detection = Some(0)
    }

    /// Write the config to the proper directory with the proper prefix and file extention
    pub(crate) fn write_config_file<P: AsRef<Path>>(&self, config_dir: P) -> Result<()> {
        let cfg_path = self.config_path(config_dir)?;

        fs::write(&cfg_path, self.to_string()).context(error::NetworkDConfigWriteSnafu {
            what: "network config",
            path: cfg_path,
        })
    }

    /// Build the proper prefixed path for the config file
    fn config_path<P: AsRef<Path>>(&self, config_dir: P) -> Result<PathBuf> {
        let match_section = self
            .r#match
            .as_ref()
            .context(error::ConfigMissingNameSnafu {
                what: "network config".to_string(),
            })?;

        // Choose the device name for the filename if it exists, otherwise use the MAC
        let device_name = match (
            &match_section.name,
            match_section.permanent_mac_address.first(),
        ) {
            (Some(name), _) => name.to_string(),
            (None, Some(mac)) => mac.to_string().replace(':', ""),
            (None, None) => {
                return error::ConfigMissingNameSnafu {
                    what: "network_config".to_string(),
                }
                .fail();
            }
        };

        let filename = format!("{}{}", CONFIG_FILE_PREFIX, device_name);
        let mut cfg_path = Path::new(config_dir.as_ref()).join(filename);
        cfg_path.set_extension(Self::FILE_EXT);
        Ok(cfg_path)
    }

    // The following methods are private and primarily meant for use by the NetworkBuilder.  They
    // are convenience methods to access the referenced structs (which are `Option`s) since they
    // may need to be accessed in multiple places during the builder's construction process. (And
    // no one wants to call `get_or_insert_with()` everywhere)
    fn link_mut(&mut self) -> &mut LinkSection {
        self.link.get_or_insert_with(LinkSection::default)
    }

    fn network_mut(&mut self) -> &mut NetworkSection {
        self.network.get_or_insert_with(NetworkSection::default)
    }

    fn dhcp4_mut(&mut self) -> &mut Dhcp4Section {
        self.dhcp4.get_or_insert_with(Dhcp4Section::default)
    }

    fn dhcp6_mut(&mut self) -> &mut Dhcp6Section {
        self.dhcp6.get_or_insert_with(Dhcp6Section::default)
    }

    fn ipv6_accept_ra_mut(&mut self) -> &mut Ipv6AcceptRaSection {
        self.ipv6_accept_ra
            .get_or_insert_with(Ipv6AcceptRaSection::default)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
//
/// The builder for `NetworkConfig`.
//
// Why a builder?  Great question.  As you can see below, some logic is involved to translate
// config struct fields to a valid NetworkConfig.  Since NetworkConfig will be created by multiple
// devices (interfaces, bonds and VLANs to start), it makes sense to centralize that logic to avoid
// duplication/mistakes.  Using a builder means type parameters can be used to limit available
// methods based on the device being created.  Putting the type parameter on the builder and not
// NetworkConfig avoids proliferating the type parameter everywhere NetworkConfig may be used.
#[derive(Debug)]
pub(crate) struct NetworkBuilder<T: Device> {
    network: NetworkConfig,
    spooky: PhantomData<T>,
}

impl<T: Device> NetworkBuilder<T> {
    pub(crate) fn build(self) -> NetworkConfig {
        self.network
    }
}

impl NetworkBuilder<Interface> {
    /// Create a new .network config for an interface; not meant for an interface that will be a
    /// bond worker.  See `.new_bond_worker()` for that case.
    pub(crate) fn new_interface<I>(id: I) -> Self
    where
        I: Into<InterfaceId>,
    {
        let network = match id.into() {
            InterfaceId::Name(n) => NetworkConfig::new_with_name(n),
            InterfaceId::MacAddress(m) => NetworkConfig::new_with_mac_address(m),
        };

        Self {
            network,
            spooky: PhantomData,
        }
    }
}

impl NetworkBuilder<Bond> {
    // Create a new .network config for a network bond
    pub(crate) fn new_bond(name: InterfaceName) -> Self {
        let mut network = NetworkConfig::new_with_name(name);
        // Bonds should be brought up without waiting for a carrier
        network.network_mut().configure_wo_carrier = Some(true);

        Self {
            network,
            spooky: PhantomData,
        }
    }

    /// Bind workers to bond for carrier detection
    pub(crate) fn with_bind_carrier(&mut self, interfaces: Vec<InterfaceName>) {
        self.network.network_mut().bind_carrier = interfaces;
    }
}

impl NetworkBuilder<BondWorker> {
    /// Create a new .network config for an interface meant to be bound to a bond
    pub(crate) fn new_bond_worker(name: InterfaceName) -> Self {
        let mut network = NetworkConfig::new_with_name(name);
        // Disable all address autoconfig for bond workers
        network.network_mut().link_local_addressing = Some(DhcpBool::No);

        Self {
            network,
            spooky: PhantomData,
        }
    }

    // Add the bond this worker is bound to
    pub(crate) fn bound_to_bond(&mut self, bond: InterfaceName) {
        self.network.network_mut().bond = Some(bond);
    }

    // Make this bond worker the primary
    pub(crate) fn primary_bond_worker(&mut self) {
        self.network.network_mut().primary_bond_worker = Some(true)
    }
}

impl NetworkBuilder<Vlan> {
    // Create a new .network config for a VLAN
    pub(crate) fn new_vlan(name: InterfaceName) -> Self {
        let mut network = NetworkConfig::new_with_name(name);
        // VLANs should be brought up without waiting for a carrier
        network.network_mut().configure_wo_carrier = Some(true);

        Self {
            network,
            spooky: PhantomData,
        }
    }
}

impl NetworkBuilder<VlanLink> {
    pub(crate) fn new_vlan_link<I>(id: I) -> Self
    where
        I: Into<InterfaceId>,
    {
        let mut network = match id.into() {
            InterfaceId::Name(n) => NetworkConfig::new_with_name(n),
            InterfaceId::MacAddress(m) => NetworkConfig::new_with_mac_address(m),
        };
        // Disable all address autoconfig for vlan links
        network.network_mut().link_local_addressing = Some(DhcpBool::No);
        network.network_mut().ipv6_accept_ra = Some(false);

        Self {
            network,
            spooky: PhantomData,
        }
    }
}

// The following methods are meant only for devices able to be members of VLANs
impl<T> NetworkBuilder<T>
where
    T: CanHaveVlans + Device,
{
    /// Add multiple VLANs
    pub(crate) fn with_vlans(&mut self, vlans: Vec<InterfaceName>) {
        for vlan in vlans {
            self.with_vlan(vlan)
        }
    }

    /// Add a single VLAN
    pub(crate) fn with_vlan(&mut self, vlan: InterfaceName) {
        self.network.network_mut().vlan.push(vlan)
    }
}

// The following methods are meant only for devices not bound to a bond
impl<T> NetworkBuilder<T>
where
    T: NotBonded + Device,
{
    /// Add DHCP4 and/or DHCP6 configuration.  If neither exist, this is a no-op
    /// These options are somewhat intertwined depending on a protocol being optional, etc.
    //
    // The builder ingests dhcp4/6 options and processes them immediately, rather than storing them
    // and processing them during the build() method.  This is intentional as DHCP options are only
    // valid for devices not bound to a bond (and potentially more in the future).
    pub(crate) fn with_dhcp(&mut self, dhcp4: Option<Dhcp4ConfigV1>, dhcp6: Option<Dhcp6ConfigV1>) {
        match (dhcp4, dhcp6) {
            (Some(dhcp4), Some(dhcp6)) => self.with_dhcp_impl(dhcp4, dhcp6),
            (Some(dhcp4), None) => self.with_dhcp4(dhcp4),
            (None, Some(dhcp6)) => self.with_dhcp6(dhcp6),
            (None, None) => (),
        }
    }

    /// Private helper for adding both DHCP4 and DHCP6 configuration since the options are
    /// intertwined
    fn with_dhcp_impl(&mut self, dhcp4: Dhcp4ConfigV1, dhcp6: Dhcp6ConfigV1) {
        self.network.network_mut().dhcp =
            match (Self::dhcp4_enabled(&dhcp4), Self::dhcp6_enabled(&dhcp6)) {
                (true, true) => Some(DhcpBool::Yes),
                (true, false) => Some(DhcpBool::Ipv4),
                (false, true) => Some(DhcpBool::Ipv6),
                (false, false) => Some(DhcpBool::No),
            };

        let link = self.network.link_mut();
        match (Self::dhcp4_required(&dhcp4), Self::dhcp6_required(&dhcp6)) {
            (true, true) => {
                link.required = Some(true);
                link.required_family = Some(RequiredFamily::Both);
            }
            (true, false) => {
                link.required = Some(true);
                link.required_family = Some(RequiredFamily::Ipv4);
            }
            (false, true) => {
                link.required = Some(true);
                link.required_family = Some(RequiredFamily::Ipv6);
            }
            (false, false) => link.required = Some(false),
        }

        let dhcp4_is_enabled = Self::dhcp4_enabled(&dhcp4);
        let dhcp6_is_enabled = Self::dhcp6_enabled(&dhcp6);
        let dhcp_is_enabled = dhcp4_is_enabled || dhcp6_is_enabled;

        if dhcp4_is_enabled {
            let dhcp4_s = self.network.dhcp4_mut();
            dhcp4_s.metric = Self::dhcp4_metric(&dhcp4);
            dhcp4_s.use_mtu = Some(true);
        }

        if dhcp6_is_enabled {
            let ipv6_accept_ra_s = self.network.ipv6_accept_ra_mut();
            ipv6_accept_ra_s.use_mtu = Some(true);
        }

        if dhcp_is_enabled {
            self.network.network_mut().keep_configuration = Some(KeepConfiguration::Dhcp);
        }
    }

    /// Private helper for adding DHCP4 config
    fn with_dhcp4(&mut self, dhcp4: Dhcp4ConfigV1) {
        self.network.network_mut().dhcp = match Self::dhcp4_enabled(&dhcp4) {
            true => Some(DhcpBool::Ipv4),
            false => Some(DhcpBool::No),
        };

        self.network.link_mut().required = Some(Self::dhcp4_required(&dhcp4));

        if Self::dhcp4_enabled(&dhcp4) {
            let dhcp = self.network.dhcp4_mut();
            dhcp.metric = Self::dhcp4_metric(&dhcp4);
            dhcp.use_mtu = Some(true);
            self.network.network_mut().keep_configuration = Some(KeepConfiguration::Dhcp);
        }
    }

    /// Private helper for adding DHCP6 config
    fn with_dhcp6(&mut self, dhcp6: Dhcp6ConfigV1) {
        self.network.network_mut().dhcp = match Self::dhcp6_enabled(&dhcp6) {
            true => Some(DhcpBool::Ipv6),
            false => Some(DhcpBool::No),
        };

        self.network.link_mut().required = Some(Self::dhcp6_required(&dhcp6));

        if Self::dhcp6_enabled(&dhcp6) {
            let accept_ra = self.network.ipv6_accept_ra_mut();
            accept_ra.use_mtu = Some(true);
            self.network.network_mut().keep_configuration = Some(KeepConfiguration::Dhcp);
        }
    }

    /// Add static address configuration
    pub(crate) fn with_static_config(&mut self, static_config: StaticConfigV1) {
        self.network
            .network_mut()
            .addresses
            .append(&mut static_config.addresses.into_iter().collect())
    }

    /// Add multiple static routes
    pub(crate) fn with_routes(&mut self, routes: Vec<RouteV1>) {
        for route in routes {
            self.with_route(route)
        }
    }

    /// Add a single static route
    pub(crate) fn with_route(&mut self, route: RouteV1) {
        let destination = match route.to {
            RouteTo::DefaultRoute => match route.via.or(route.from) {
                Some(IpAddr::V4(_)) => Some(*DEFAULT_ROUTE_IPV4),
                Some(IpAddr::V6(_)) => Some(*DEFAULT_ROUTE_IPV6),
                // If no gateway or from is given, assume the ipv4 default
                None => Some(*DEFAULT_ROUTE_IPV4),
            },
            RouteTo::Ip(ip) => Some(ip),
        };

        // Each route gets its own RouteSection
        let route_section = RouteSection {
            destination,
            gateway: route.via,
            metric: route.route_metric,
            preferred_source: route.from,
        };

        self.network.route.push(route_section)
    }

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
    // The following helper methods on the `DhcpXConfig` structs exist to conveniently parse out
    // the required information.  Since this is the only place we parse these values, and will only
    // ever be parsing them from the latest version of DHCP structs, it doesn't really make sense
    // to implement them on the DhcpXConfig structs themselves. (Not to mention all the repeated
    // dead code that would exist as new versions were built)
    fn dhcp4_enabled(dhcp4: &Dhcp4ConfigV1) -> bool {
        match dhcp4 {
            Dhcp4ConfigV1::DhcpEnabled(b) => *b,
            Dhcp4ConfigV1::WithOptions(o) => o.enabled,
        }
    }

    fn dhcp4_required(dhcp4: &Dhcp4ConfigV1) -> bool {
        match dhcp4 {
            // Assume enabled == required
            Dhcp4ConfigV1::DhcpEnabled(enabled) => *enabled,
            // If "optional" isn't set, assume DHCP is required.
            // If optional==true, DHCP is NOT required
            Dhcp4ConfigV1::WithOptions(o) => o.optional.map_or(true, |b| !b),
        }
    }

    fn dhcp4_metric(dhcp4: &Dhcp4ConfigV1) -> Option<u32> {
        match dhcp4 {
            Dhcp4ConfigV1::DhcpEnabled(_) => None,
            Dhcp4ConfigV1::WithOptions(o) => o.route_metric,
        }
    }

    fn dhcp6_enabled(dhcp6: &Dhcp6ConfigV1) -> bool {
        match dhcp6 {
            Dhcp6ConfigV1::DhcpEnabled(b) => *b,
            Dhcp6ConfigV1::WithOptions(o) => o.enabled,
        }
    }

    fn dhcp6_required(dhcp6: &Dhcp6ConfigV1) -> bool {
        match dhcp6 {
            // Assume enabled == required
            Dhcp6ConfigV1::DhcpEnabled(enabled) => *enabled,
            // If "optional" isn't set, assume DHCP is required
            // If optional==true, DHCP is NOT required
            Dhcp6ConfigV1::WithOptions(o) => o.optional.map_or(true, |b| !b),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::networkd::config::tests::{test_data, TestDevices, BUILDER_DATA};
    use crate::networkd::devices::{NetworkDBond, NetworkDInterface, NetworkDVlan};

    const FAKE_TEST_DIR: &str = "testdir";

    fn network_path(name: String) -> PathBuf {
        test_data()
            .join("network")
            .join(format!("{}.network", name))
    }

    fn network_from_interface(iface: NetworkDInterface) -> NetworkConfig {
        let mut network = NetworkBuilder::new_interface(iface.name);
        network.with_dhcp(iface.dhcp4, iface.dhcp6);
        if let Some(s) = iface.static4 {
            network.with_static_config(s)
        }
        if let Some(s) = iface.static6 {
            network.with_static_config(s)
        }
        if let Some(r) = iface.routes {
            network.with_routes(r)
        }
        network.build()
    }

    fn network_from_vlan(vlan: NetworkDVlan) -> NetworkConfig {
        let mut network = NetworkBuilder::new_vlan(vlan.name);
        network.with_dhcp(vlan.dhcp4, vlan.dhcp6);
        if let Some(s) = vlan.static4 {
            network.with_static_config(s)
        }
        if let Some(s) = vlan.static6 {
            network.with_static_config(s)
        }
        if let Some(r) = vlan.routes {
            network.with_routes(r)
        }
        network.build()
    }

    fn network_from_bond(bond: NetworkDBond) -> NetworkConfig {
        let mut network = NetworkBuilder::new_bond(bond.name.clone());
        network.with_dhcp(bond.dhcp4, bond.dhcp6);
        if let Some(s) = bond.static4 {
            network.with_static_config(s)
        }
        if let Some(s) = bond.static6 {
            network.with_static_config(s)
        }
        if let Some(r) = bond.routes {
            network.with_routes(r)
        }
        network.with_bind_carrier(bond.interfaces);
        network.build()
    }

    #[test]
    fn interface_network_builder() {
        let devices = toml::from_str::<TestDevices>(BUILDER_DATA).unwrap();

        for interface in devices.interface {
            let expected_filename = network_path(interface.name.to_string().replace(':', ""));
            let expected = fs::read_to_string(expected_filename).unwrap();
            let got = network_from_interface(interface).to_string();

            assert_eq!(expected, got)
        }
    }

    #[test]
    fn vlan_network_builder() {
        let devices = toml::from_str::<TestDevices>(BUILDER_DATA).unwrap();

        for vlan in devices.vlan {
            let expected_filename = network_path(vlan.name.to_string());
            let expected = fs::read_to_string(expected_filename).unwrap();
            let got = network_from_vlan(vlan).to_string();

            assert_eq!(expected, got)
        }
    }

    #[test]
    fn bond_network_builder() {
        let devices = toml::from_str::<TestDevices>(BUILDER_DATA).unwrap();

        for bond in devices.bond {
            let expected_filename = network_path(bond.name.to_string());
            let expected = fs::read_to_string(expected_filename).unwrap();
            let got = network_from_bond(bond).to_string();

            assert_eq!(expected, got)
        }
    }

    #[test]
    fn bond_worker_network_builder() {
        let devices = toml::from_str::<TestDevices>(BUILDER_DATA).unwrap();

        // Validate the first interface gets the Primary bit added and the second doesn't.  Worker
        // config is identical so validating the first set keeps us from creating a bunch of
        // redundant identical files
        let bond = devices.bond.first().unwrap();
        for (index, worker) in bond.interfaces.iter().enumerate() {
            let mut network = NetworkBuilder::new_bond_worker(worker.clone());
            network.bound_to_bond(bond.name.clone());
            if index == 0 {
                network.primary_bond_worker();
            }

            let expected_filename = network_path(worker.to_string());
            let expected = fs::read_to_string(expected_filename).unwrap();
            let got = network.build().to_string();
            assert_eq!(expected, got)
        }
    }

    #[test]
    fn config_path_empty() {
        let n = NetworkConfig::default();
        assert!(n.config_path(FAKE_TEST_DIR).is_err())
    }

    #[test]
    fn config_path_name() {
        let filename = format!("{}foo", CONFIG_FILE_PREFIX);
        let mut expected = Path::new(FAKE_TEST_DIR).join(filename);
        expected.set_extension(NetworkConfig::FILE_EXT);

        let network = NetworkConfig::new_with_name(InterfaceName::try_from("foo").unwrap());

        assert_eq!(expected, network.config_path(FAKE_TEST_DIR).unwrap())
    }

    #[test]
    fn config_path_mac() {
        let filename = format!("{}f874a4d53264", CONFIG_FILE_PREFIX);
        let mut expected = Path::new(FAKE_TEST_DIR).join(filename);
        expected.set_extension(NetworkConfig::FILE_EXT);

        let network = NetworkConfig::new_with_mac_address(
            MacAddress::try_from("f8:74:a4:d5:32:64".to_string()).unwrap(),
        );

        assert_eq!(expected, network.config_path(FAKE_TEST_DIR).unwrap())
    }

    #[test]
    fn config_path_name_before_mac() {
        let filename = format!("{}foo", CONFIG_FILE_PREFIX);
        let mut expected = Path::new(FAKE_TEST_DIR).join(filename);
        expected.set_extension(NetworkConfig::FILE_EXT);

        let network = NetworkConfig {
            r#match: Some(MatchSection {
                name: Some(InterfaceName::try_from("foo").unwrap()),
                permanent_mac_address: vec![MacAddress::try_from("f8:74:a4:d5:32:64").unwrap()],
            }),
            ..Default::default()
        };

        assert_eq!(expected, network.config_path(FAKE_TEST_DIR).unwrap())
    }
}
