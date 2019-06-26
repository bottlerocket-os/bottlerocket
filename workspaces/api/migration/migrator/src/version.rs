//! This module handles versioning of data stores - detecting their versions, ordering versions,
//! determining whether a version change is forward or backward, etc.

use crate::error::{self, Result};
use lazy_static::lazy_static;
use regex::Regex;
use snafu::{OptionExt, ResultExt};
use std::cmp::Ordering;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// VersionComponent represents each integer segment of a version string.
pub(crate) type VersionComponent = u32;

lazy_static! {
    /// Regular expression that captures the entire version string (1.2 or v1.2) along with the
    /// major (1) and minor (2) separately.
    pub(crate) static ref VERSION_RE: Regex =
        Regex::new(r"(?P<version>v?(?P<major>[0-9]+)\.(?P<minor>[0-9]+))").unwrap();

    /// Regular expression that captures the version and ID from the name of a data store
    /// directory, e.g. matching "v1.5_0123456789abcdef" will let you retrieve version (v1.5),
    /// major (1), minor (5), and id (0123456789abcdef).
    pub(crate) static ref DATA_STORE_DIRECTORY_RE: Regex =
        Regex::new(&format!(r"^{}_(?P<id>.*)$", *VERSION_RE)).unwrap();

    /// Regular expression that will match migration file names and allow retrieving the
    /// version and name components.
    pub(crate) static ref MIGRATION_FILENAME_RE: Regex =
        Regex::new(&format!(r"^migrate_{}_(?P<name>[a-zA-Z0-9-]+)$", *VERSION_RE)).unwrap();
}

/// Version represents the version identifiers of our data store.
// Deriving Ord will check the fields in order, so as long as the more important fields (e.g.
// 'major') are listed first, it will compare versions as expected.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Version {
    /// The data store format version, or major for short.
    pub(crate) major: VersionComponent,
    /// The content format version, or minor for short.
    pub(crate) minor: VersionComponent,
}

impl FromStr for Version {
    type Err = error::Error;

    /// Parse a version string like "1.0" or "v1.0" into a Version.
    fn from_str(input: &str) -> Result<Self> {
        Self::from_str_with_re(input, &VERSION_RE)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

impl Version {
    #[allow(dead_code)]
    pub(crate) fn new(major: VersionComponent, minor: VersionComponent) -> Self {
        Self { major, minor }
    }

    /// Parses the input string into a Version, with a given Regex that is expected to provide
    /// "major" and "minor" captures.  Used to implement the pub(crate) entry points.
    fn from_str_with_re(input: &str, re: &Regex) -> Result<Self> {
        trace!("Parsing version from string: {}", input);

        let captures = re.captures(&input).context(error::InvalidVersion {
            given: input,
            re: re.as_str(),
        })?;

        let major_str = captures.name("major").context(error::Internal {
            msg: "Version matched regex but we don't have a 'major' capture",
        })?;
        let minor_str = captures.name("minor").context(error::Internal {
            msg: "Version matched regex but we don't have a 'minor' capture",
        })?;

        let major = major_str
            .as_str()
            .parse::<VersionComponent>()
            .with_context(|| error::InvalidVersionComponent {
                component: major_str.as_str(),
            })?;
        let minor = minor_str
            .as_str()
            .parse::<VersionComponent>()
            .with_context(|| error::InvalidVersionComponent {
                component: minor_str.as_str(),
            })?;

        trace!("Parsed major '{}' and minor '{}'", major, minor);
        Ok(Self { major, minor })
    }

    /// This pulls the version number out of the given datastore path.
    ///
    /// Returns Err if the given path isn't named like a versioned data store.
    ///
    /// Background: The data store path uses symlinks to represent versions and allow for easy
    /// version flips.  This function expects the target directory path.
    ///
    /// An example setup for version 1.5:
    ///    /path/to/datastore/current
    ///    -> /path/to/datastore/v1
    ///    -> /path/to/datastore/v1.5
    ///    -> /path/to/datastore/v1.5_0123456789abcdef
    pub(crate) fn from_datastore_path<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        trace!("Getting version from datastore path: {}", path.display());

        // Pull out the basename of the path, which contains the version
        let version_os_str = path
            .file_name()
            .context(error::DataStoreLinkToRoot { path: &path })?;
        let version_str = version_os_str
            .to_str()
            .context(error::DataStorePathNotUTF8 { path: &path })?;

        // Parse and return the version
        Self::from_str_with_re(version_str, &DATA_STORE_DIRECTORY_RE)
    }
}

/// Direction represents whether we're moving forward toward a newer version, or rolling back to
/// an older version.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Direction {
    Forward,
    Backward,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Forward => write!(f, "--forward"),
            Direction::Backward => write!(f, "--backward"),
        }
    }
}

impl Direction {
    /// Determines the migration direction, given the outgoing ("from') and incoming ("to")
    /// versions.
    pub(crate) fn from_versions(from: Version, to: Version) -> Option<Self> {
        match from.cmp(&to) {
            Ordering::Less => Some(Direction::Forward),
            Ordering::Greater => Some(Direction::Backward),
            Ordering::Equal => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Direction, Version};
    use std::str::FromStr;

    #[test]
    fn version_eq() {
        assert_eq!(Version::new(0, 0), Version::new(0, 0));
        assert_eq!(Version::new(1, 0), Version::new(1, 0));
        assert_eq!(Version::new(1, 1), Version::new(1, 1));

        assert_ne!(Version::new(0, 0), Version::new(0, 1));
        assert_ne!(Version::new(0, 1), Version::new(1, 0));
        assert_ne!(Version::new(1, 0), Version::new(0, 1));
    }

    #[test]
    fn version_ord() {
        assert!(Version::new(0, 1) > Version::new(0, 0));
        assert!(Version::new(1, 0) > Version::new(0, 99));
        assert!(Version::new(1, 1) > Version::new(1, 0));

        assert!(Version::new(0, 0) < Version::new(0, 1));
        assert!(Version::new(0, 99) < Version::new(1, 0));
        assert!(Version::new(1, 0) < Version::new(1, 1));
    }

    #[test]
    fn from_str() {
        assert_eq!(Version::from_str("0.1").unwrap(), Version::new(0, 1));
        assert_eq!(Version::from_str("1.0").unwrap(), Version::new(1, 0));
        assert_eq!(Version::from_str("2.3").unwrap(), Version::new(2, 3));

        assert_eq!(Version::from_str("v0.1").unwrap(), Version::new(0, 1));
        assert_eq!(Version::from_str("v1.0").unwrap(), Version::new(1, 0));
        assert_eq!(Version::from_str("v2.3").unwrap(), Version::new(2, 3));
    }

    #[test]
    fn fmt() {
        assert_eq!("v0.1", format!("{}", Version::new(0, 1)));
        assert_eq!("v1.0", format!("{}", Version::new(1, 0)));
        assert_eq!("v2.3", format!("{}", Version::new(2, 3)));
    }

    #[test]
    fn direction() {
        let v01 = Version::new(0, 1);
        let v02 = Version::new(0, 2);
        let v10 = Version::new(1, 0);

        assert_eq!(Direction::from_versions(v01, v02), Some(Direction::Forward));
        assert_eq!(
            Direction::from_versions(v02, v01),
            Some(Direction::Backward)
        );
        assert_eq!(Direction::from_versions(v01, v01), None);

        assert_eq!(Direction::from_versions(v02, v10), Some(Direction::Forward));
        assert_eq!(
            Direction::from_versions(v10, v02),
            Some(Direction::Backward)
        );
    }
}
