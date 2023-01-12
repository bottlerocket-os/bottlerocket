//! The basic_tests macro contains tests that are applicable to all versions of network
//! configuration.  These tests ensure some basic parsing/validating works properly.  The tests
//! also ensure general network config rules are followed, such as a single primary interface is
//! defined, etc.
//!
//! The macro's only argument is the version of net config currently being tested.
macro_rules! basic_tests {
    ($version:expr) => {
        mod basic {
            use std::convert::TryFrom;
            use $crate::interface_id::{InterfaceId, InterfaceName};
            use $crate::net_config::deserialize_config;
            use $crate::net_config::test_macros::gen_boilerplate;

            gen_boilerplate!($version, "basic");

            #[test]
            fn invalid_version() {
                let bad = net_config().join("bad_version.toml");
                let bad_str = fs::read_to_string(bad).unwrap();
                assert!(deserialize_config(&bad_str).is_err())
            }

            #[test]
            fn ok_config() {
                let ok = net_config().join("net_config.toml");
                let rendered = render_config_template(ok);
                assert!(deserialize_config(&rendered).is_ok())
            }

            #[test]
            fn invalid_interface_config() {
                let bad = net_config().join("invalid_interface_config.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_err())
            }

            #[test]
            fn no_interfaces() {
                let bad = net_config().join("no_interfaces.toml");
                let rendered = render_config_template(bad);
                assert!(deserialize_config(&rendered).is_ok())
            }

            #[test]
            fn defined_primary_interface() {
                let ok_path = net_config().join("net_config.toml");
                let cfg = deserialize_config(&render_config_template(ok_path)).unwrap();

                let expected = InterfaceId::from(InterfaceName::try_from("eno2").unwrap());
                let actual = cfg.primary_interface().unwrap();
                assert_eq!(expected, actual)
            }

            #[test]
            fn undefined_primary_interface() {
                let ok_path = net_config().join("no_primary.toml");
                let cfg = deserialize_config(&render_config_template(ok_path)).unwrap();

                let expected = InterfaceId::from(InterfaceName::try_from("eno3").unwrap());
                let actual = cfg.primary_interface().unwrap();
                assert_eq!(expected, actual)
            }

            #[test]
            fn multiple_primary_interfaces() {
                let multiple = net_config().join("multiple_primary.toml");
                let rendered = render_config_template(multiple);
                assert!(deserialize_config(&rendered).is_err())
            }
        }
    };
}

pub(crate) use basic_tests;
