use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::path::Path;

pub use derived_data::*;

lazy_static! {
    pub static ref FILESYSTEM_FILE_SUFFICES: std::vec::Vec<&'static str> = vec![
        "boot.ext4.lz4",
        "root.ext4.lz4",
        "root.ext4.lz4",
        "root.verity.lz4"
    ];
    pub static ref DISK_IMAGE_FILE_SUFFICES: std::vec::Vec<&'static str> =
        vec!["img.lz4", "data.img.lz4"];
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub version: version::Version,
    pub datastore_version: version::Version,
    pub migrations: Vec<Migration>,
}

impl Release {
    pub fn from_toml_file<P>(path: P) -> Result<Release, error::Error>
    where
        P: AsRef<Path> + Clone,
    {
        let release: Release = std::fs::read_to_string(&path)
            .context(error::FileReadError {
                path: std::path::PathBuf::from(path.as_ref().to_path_buf()),
            })
            .and_then(|c| toml::from_str(&c).context(error::TomlParseError {}))?;
        Ok(release)
    }

    pub fn as_build(
        &self,
        arch: String,
        variant: String,
        suffix: Option<String>,
    ) -> derived_data::Build {
        derived_data::Build {
            version: self.version.clone(),
            arch: arch,
            variant: variant,
            suffix: suffix,
        }
    }

    pub fn migration_names(&self) -> Vec<String> {
        self.migrations
            .iter()
            .map(|m| m.names.clone())
            .flatten()
            .collect()
    }

    pub fn migration_crates(&self) -> Result<Vec<String>, error::Error> {
        let (names, errors): (Vec<Result<_, _>>, Vec<Result<_, _>>) = self
            .migration_names()
            .into_iter()
            .map(|n| {
                // Convert migration_vXX_crate-name into crate-name
                n.rsplit("_")
                    .nth(0)
                    .map(|s| s.to_owned())
                    .context(error::MigrationNameFormatError { name: n.clone() })
            })
            .partition::<Vec<Result<String, error::Error>>, _>(|x| x.is_ok());
        ensure!(
            errors.len() == 0,
            error::MigrationNameFormatErrors {
                errors: errors
                    .into_iter()
                    .map(Result::unwrap_err)
                    .collect::<Vec<error::Error>>(),
            }
        );

        Ok(names.into_iter().map(Result::unwrap).collect())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Migration {
    pub from: version::Version,
    pub to: version::Version,
    pub names: Vec<String>,
}

pub mod error {
    use snafu::Snafu;

    #[derive(Snafu, Debug)]
    pub enum Error {
        #[snafu(visibility(pub(crate)))]
        #[snafu(display("unable to parse TOML: {}", source.to_string()))]
        TomlParseError { source: toml::de::Error },

        #[snafu(visibility(pub(crate)))]
        #[snafu(display("unable to parse TOML from file {:?}: {}", path, source.to_string()))]
        FileReadError {
            path: std::path::PathBuf,
            source: std::io::Error,
        },

        #[snafu(visibility(pub(crate)))]
        #[snafu(display("unable to determine crate name from migration name: {}", name))]
        MigrationNameFormatError { name: std::string::String },

        #[snafu(visibility(pub(crate)))]
        #[snafu(display("unable to determine crate name from migration names: {:?}", errors.iter().map(|x| x.to_string()).collect::<Vec<String>>()))]
        MigrationNameFormatErrors { errors: Vec<Error> },
    }
}

pub mod derived_data {
    use super::version;

    pub struct Build {
        pub(super) version: version::Version,
        pub(super) variant: String,
        pub(super) arch: String,
        pub(super) suffix: Option<String>,
    }

    impl Build {
        pub fn image_name(&self) -> String {
            let suffix = self
                .suffix
                .as_ref()
                .map_or_else(|| "".to_owned(), |s| "-".to_owned() + s.as_ref());
            format!(
                "thar-{arch}-{variant}-{version}{suffix}",
                arch = self.arch,
                variant = self.variant,
                version = self.version,
                suffix = suffix
            )
        }
    }
}

pub mod version {
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use snafu::ResultExt;
    use std::fmt::{self, Display};
    use std::str::FromStr;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct Version {
        string: String,
        semver: semver::Version,
    }

    impl Version {
        pub fn semver(&self) -> semver::Version {
            self.semver.clone()
        }
    }

    impl Display for Version {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.string)
        }
    }

    impl Serialize for Version {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.string)
        }
    }

    impl<'de> Deserialize<'de> for Version {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let mut s = String::deserialize(deserializer)?;
            // Append a .0 if we're looking at a X.Y not an X.Y.Z
            // version string.
            match s.matches(".").count() {
                0 => error::InvalidString { string: s.clone() }
                    .fail()
                    .map_err(de::Error::custom)?,
                1 => s.push_str(".0"),
                _ => (),
            }

            let sver = semver::Version::from_str(&s)
                .context(error::ParseError { string: s.clone() })
                .map_err(de::Error::custom)?;
            Ok(Self {
                string: s,
                semver: sver,
            })
        }
    }

    pub mod error {
        use snafu::Snafu;

        #[derive(Snafu, Debug)]
        pub enum Error {
            #[snafu(visibility(pub))]
            #[snafu(display("invalid version string {}", string))]
            InvalidString { string: String },

            #[snafu(visibility(pub))]
            #[snafu(display("error parsing version from string {}: {}", string, source.to_string()))]
            ParseError {
                string: String,
                source: semver::SemVerError,
            },
        }
    }
}
