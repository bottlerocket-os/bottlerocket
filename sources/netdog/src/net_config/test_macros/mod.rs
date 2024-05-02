//! The test_macros module contains macros meant to simplify testing across net config versions.
//!
//! Net configuration versions are typically additive, meaning new versions add support for new
//! features, while continuing support for previous features.  We want to ensure that we test all
//! applicable features for a version, without duplicating test code between version files and/or
//! test blocks.  The macros themselves are declarative and define a new module that contains all
//! the tests.  This allows us to use the same tests across all versions of net config, and get
//! nicely formatted `cargo test` output showing the net config version, module, and test.  This is
//! much nicer than looping over versions within a single test, and provides much better error
//! output for the user.
//!
//! Each macro typically has an associated directory inside the `test_data` folder that provides
//! templated net config files for parsing.
#[cfg(test)]
pub(super) mod basic;
#[cfg(test)]
pub(super) mod bonding;
#[cfg(test)]
pub(super) mod dhcp;
#[cfg(test)]
pub(super) mod static_address;
#[cfg(test)]
pub(super) mod vlan;

pub(super) use basic::basic_tests;
pub(super) use bonding::bonding_tests;
pub(super) use dhcp::dhcp_tests;
pub(super) use static_address::static_address_tests;
pub(super) use vlan::vlan_tests;

/// gen_boilerplate!() is a convenience macro meant to be used inside of test macros to generate
/// some generally useful boilerplate code.  It creates a `VERSION` constant in case the test
/// macros need it, and provides some convenience functions for gathering test data directories,
/// and rendering config templates in said directories.
///
/// The macro receives arguments for the net config version, as well as the directory where the
/// associated test files can be found.
macro_rules! gen_boilerplate {
    ($version:expr, $test_dir:expr) => {
        use handlebars::Handlebars;
        use serde::Serialize;
        use std::fs;
        use std::path::{Path, PathBuf};

        static VERSION: usize = $version;

        fn test_data() -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
        }

        fn net_config() -> PathBuf {
            test_data().join("net_config").join($test_dir)
        }

        #[derive(Serialize)]
        struct Context {
            version: usize,
        }

        fn render_config_template<P>(path: P) -> String
        where
            P: AsRef<Path>,
        {
            let path = path.as_ref();
            let path_str = fs::read_to_string(path).unwrap();

            let mut hb = Handlebars::new();
            hb.register_template_string("template", &path_str).unwrap();

            let context = Context { version: VERSION };
            hb.render("template", &context).unwrap()
        }
    };
}

pub(super) use gen_boilerplate;
