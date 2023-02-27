use bottlerocket_release::BottlerocketRelease;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use semver::Version;
use snafu::ResultExt;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

pub mod error {
    use std::io;
    use std::path::PathBuf;

    use snafu::Snafu;

    /// Public error type for `libstorewolf`
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Unable to create directory at '{}': {}", path.display(), source))]
        DirectoryCreation { path: PathBuf, source: io::Error },

        #[snafu(display("Failed to create symlink at '{}': {}", path.display(), source))]
        LinkCreate { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to get OS version: {}", source))]
        ReleaseVersion { source: bottlerocket_release::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

/// Given a base path, create a brand new datastore with the appropriate
/// symlink structure for the desired datastore version.
///
/// If `version` is given, uses it, otherwise pulls version from bottlerocket-release.
///
/// An example setup for theoretical version 1.5:
///    /path/to/datastore/current
///    -> /path/to/datastore/v1
///    -> /path/to/datastore/v1.5
///    -> /path/to/datastore/v1.5.2
///    -> /path/to/datastore/v1.5.2_0123456789abcdef
///
/// Returns the path to the datastore (i.e. the last path in the above example).
pub fn create_new_datastore<P: AsRef<Path>>(
    base_path: P,
    version: Option<Version>,
) -> Result<PathBuf> {
    let version = match version {
        Some(v) => v,
        None => {
            let br = BottlerocketRelease::new().context(error::ReleaseVersionSnafu)?;
            br.version_id
        }
    };

    // Create random string to append to the end of the new datastore path
    let random_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // Build the various paths to which we'll symlink

    // /path/to/datastore/v1
    let major_version_filename = format!("v{}", version.major);
    let major_version_path = base_path.as_ref().join(&major_version_filename);

    // /path/to/datastore/v1.5
    let minor_version_filename = format!("v{}.{}", version.major, version.minor);
    let minor_version_path = base_path.as_ref().join(&minor_version_filename);

    // /path/to/datastore/v1.5.2
    let patch_version_filename = format!("v{}.{}.{}", version.major, version.minor, version.patch);
    let patch_version_path = base_path.as_ref().join(&patch_version_filename);

    // /path/to/datastore/v1.5_0123456789abcdef
    let data_store_filename = format!(
        "v{}.{}.{}_{}",
        version.major, version.minor, version.patch, random_id
    );
    let data_store_path = base_path.as_ref().join(&data_store_filename);

    // /path/to/datastore/current
    let current_path = base_path.as_ref().join("current");

    // Create the path to the datastore, i.e /path/to/datastore/v1.5_0123456789abcdef
    fs::create_dir_all(&data_store_path).context(error::DirectoryCreationSnafu {
        path: &base_path.as_ref(),
    })?;

    // Build our symlink chain (See example in docstring above)
    // /path/to/datastore/v1.5.2 -> v1.5.2_0123456789abcdef
    symlink(&data_store_filename, &patch_version_path).context(error::LinkCreateSnafu {
        path: &patch_version_path,
    })?;
    // /path/to/datastore/v1.5 -> v1.5.2
    symlink(&patch_version_filename, &minor_version_path).context(error::LinkCreateSnafu {
        path: &minor_version_path,
    })?;
    // /path/to/datastore/v1 -> v1.5
    symlink(&minor_version_filename, &major_version_path).context(error::LinkCreateSnafu {
        path: &major_version_path,
    })?;
    // /path/to/datastore/current -> v1
    symlink(&major_version_filename, &current_path).context(error::LinkCreateSnafu {
        path: &current_path,
    })?;
    Ok(data_store_path)
}
