macro_rules! vlan_tests {
    ($version:expr) => {
        mod vlan {
            use $crate::net_config::deserialize_config;
            use $crate::net_config::test_macros::gen_boilerplate;

            gen_boilerplate!($version, "vlan");

            #[test]
            fn ok_config() {
                let ok = net_config().join("net_config.toml");
                let rendered = render_config_template(ok);
                assert!(deserialize_config(&rendered).is_ok())
            }

            #[test]
            fn no_id() {
                let bad = net_config().join("no_id.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn out_of_bounds_id() {
                let bad = net_config().join("oob_id.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn missing_kind() {
                let bad = net_config().join("missing_kind.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn no_device() {
                let bad = net_config().join("no_device.toml");
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
            fn mac_in_device_field() {
                let bad = net_config().join("mac_in_device.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }
        }
    };
}
pub(crate) use vlan_tests;
