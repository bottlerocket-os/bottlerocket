//! This module owns the Direction type used by the migrator to determine whether a migration
//! is moving forward to a new version or rolling back to a previous version.

use semver::Version;
use std::cmp::{Ord, Ordering};
use std::fmt;

/// Direction represents whether we're moving forward toward a newer version, or rolling back to
/// an older version.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Direction {
    Forward,
    Backward,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Forward => write!(f, "--forward"),
            Direction::Backward => write!(f, "--backward"),
        }
    }
}

impl Direction {
    /// Determines the migration direction, given the outgoing ("from') and incoming ("to")
    /// versions.
    pub(crate) fn from_versions(from: &Version, to: &Version) -> Option<Self> {
        match from.cmp(to) {
            Ordering::Less => Some(Direction::Forward),
            Ordering::Greater => Some(Direction::Backward),
            Ordering::Equal => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Direction;
    use semver::Version;

    #[test]
    fn direction() {
        let v01 = Version::new(0, 0, 1);
        let v02 = Version::new(0, 0, 2);
        let v10 = Version::new(0, 1, 0);

        assert_eq!(
            Direction::from_versions(&v01, &v02),
            Some(Direction::Forward)
        );
        assert_eq!(
            Direction::from_versions(&v02, &v01),
            Some(Direction::Backward)
        );
        assert_eq!(Direction::from_versions(&v01, &v01), None);

        assert_eq!(
            Direction::from_versions(&v02, &v10),
            Some(Direction::Forward)
        );
        assert_eq!(
            Direction::from_versions(&v10, &v02),
            Some(Direction::Backward)
        );
    }
}
