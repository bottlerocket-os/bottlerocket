macro_rules! static_address_tests {
    ($version:expr) => {
        mod static_address {
            use $crate::net_config::deserialize_config;
            use $crate::net_config::test_macros::gen_boilerplate;

            gen_boilerplate!($version, "static_address");

            #[test]
            fn ok_config() {
                let ok = net_config().join("net_config.toml");
                let rendered = render_config_template(ok);
                assert!(deserialize_config(&rendered).is_ok())
            }

            #[test]
            fn dhcp_and_static_addresses() {
                let ok = net_config().join("dhcp_and_static.toml");
                let rendered = render_config_template(ok);
                assert!(deserialize_config(&rendered).is_ok())
            }

            #[test]
            fn dhcp_and_routes() {
                let bad = net_config().join("dhcp_and_routes.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn no_dhcp_or_static() {
                let bad = net_config().join("no_dhcp_or_static.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn routes_no_addresses() {
                let bad = net_config().join("routes_no_addresses.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn invalid_static_config() {
                let bad = net_config().join("invalid_static_config.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn ipv6_in_static4() {
                let bad = net_config().join("ipv6_in_static4.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn ipv4_in_static6() {
                let bad = net_config().join("ipv4_in_static6.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }
        }
    };
}
pub(crate) use static_address_tests;
