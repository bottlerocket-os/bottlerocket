macro_rules! bonding_tests {
    ($version:expr) => {
        mod bonding {
            use $crate::net_config::deserialize_config;
            use $crate::net_config::test_macros::gen_boilerplate;

            gen_boilerplate!($version, "bonding");

            #[test]
            fn ok_config() {
                let ok = net_config().join("net_config.toml");
                let rendered = render_config_template(ok);
                assert!(deserialize_config(&rendered).is_ok())
            }

            #[test]
            fn missing_kind() {
                let bad = net_config().join("missing_kind.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn no_monitoring() {
                let bad = net_config().join("no_monitoring.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn both_monitoring() {
                let bad = net_config().join("both_monitoring.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn no_interfaces() {
                let bad = net_config().join("no_interfaces.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn disabled_miimon() {
                let bad = net_config().join("disabled_miimon.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn disabled_arpmon() {
                let bad = net_config().join("disabled_arpmon.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn too_many_min_links() {
                let bad = net_config().join("too_many_min_links.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn arpmon_no_targets() {
                let bad = net_config().join("arpmon_no_targets.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn vlan_using_bond_interface() {
                let bad = net_config().join("vlan_using_bond.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn mac_as_identifier() {
                let bad = net_config().join("mac_as_identifier.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn mac_in_interfaces_list() {
                let bad = net_config().join("mac_in_interfaces.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }
        }
    };
}
pub(crate) use bonding_tests;
