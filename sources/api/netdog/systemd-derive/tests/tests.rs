use systemd_derive::{SystemdUnit, SystemdUnitSection};

#[derive(Debug, Default, SystemdUnit)]
struct NetworkConfig {
    r#match: Option<MatchSection>,
    network: Option<NetworkSection>,
    route: Vec<RouteSection>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Match")]
struct MatchSection {
    #[systemd(entry = "Name")]
    name: Option<String>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Network")]
struct NetworkSection {
    #[systemd(entry = "Address")]
    addresses: Vec<String>,
    #[systemd(entry = "DHCP")]
    dhcp: Option<String>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Route")]
struct RouteSection {
    #[systemd(entry = "Destination")]
    destination: Option<String>,
}

#[test]
fn empty() {
    let n = NetworkConfig {
        r#match: None,
        network: None,
        route: vec![],
    };

    assert_eq!(n.to_string(), "")
}

// Test all features: repeated entries and sections
#[test]
fn all_features() {
    let n = NetworkConfig {
        r#match: Some(MatchSection {
            name: Some("eno1".to_string()),
        }),
        network: Some(NetworkSection {
            addresses: vec!["1.2.3.4".to_string(), "2.3.4.5".to_string()],
            dhcp: Some("ipv4".to_string()),
        }),
        route: vec![
            RouteSection {
                destination: Some("10.0.0.1".to_string()),
            },
            RouteSection {
                destination: Some("11.0.0.1".to_string()),
            },
        ],
    };

    let expected = "[Match]
Name=eno1
[Network]
Address=1.2.3.4
Address=2.3.4.5
DHCP=ipv4
[Route]
Destination=10.0.0.1
[Route]
Destination=11.0.0.1
";

    assert_eq!(n.to_string(), expected)
}
