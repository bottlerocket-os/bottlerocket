//! The dhcp_tests macro contains tests pertaining to DHCP.  These tests are applicable to all
//! versions of network configuration.
//!
//! The macro's only argument is the version of net config currently being tested.
macro_rules! dhcp_tests {
    ($version:expr) => {
        mod dhcp {
            use $crate::net_config::deserialize_config;
            use $crate::net_config::test_macros::gen_boilerplate;

            gen_boilerplate!($version, "dhcp");

            #[test]
            fn invalid_dhcp4_options() {
                let bad = net_config().join("invalid_dhcp4_options.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn invalid_dhcp6_options() {
                let bad = net_config().join("invalid_dhcp6_options.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn invalid_dhcp_config() {
                let bad = net_config().join("invalid_dhcp_config.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn dhcp4_missing_enable() {
                let bad = net_config().join("dhcp4_missing_enabled.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn dhcp6_missing_enable() {
                let bad = net_config().join("dhcp6_missing_enabled.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }
        }
    };
}

pub(crate) use dhcp_tests;
