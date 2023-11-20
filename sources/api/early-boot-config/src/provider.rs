//! The provider module owns the `PlatformDataProvider` trait
mod ec2_identity_doc;
mod local_defaults;
mod local_file;
mod local_user_data;

use crate::compression::expand_file_maybe;
use crate::settings::SettingsJson;
use crate::LOG_LEVEL_ENV_VAR;
use async_trait::async_trait;
pub use ec2_identity_doc::Ec2IdentityDoc;
use env_logger::{Env, Target, WriteStyle};
pub use local_defaults::LocalDefaults;
pub use local_user_data::LocalUserData;
use snafu::ResultExt;
use std::path::Path;
use std::process::ExitCode;

#[cfg(variant_platform = "aws")]
mod aws;
#[cfg(variant_platform = "aws")]
pub use aws::AwsDataProvider as Platform;

#[cfg(variant_platform = "vmware")]
mod vmware;
#[cfg(variant_platform = "vmware")]
pub use vmware::VmwareDataProvider as Platform;

#[cfg(variant_platform = "metal")]
mod metal;
#[cfg(variant_platform = "metal")]
pub use metal::MetalDataProvider as Platform;

/// Support for new platforms can be added by implementing this trait.
#[async_trait]
pub trait PlatformDataProvider {
    /// You should return a list of SettingsJson, representing the settings changes you want to
    /// send to the API.
    ///
    /// This is a list so that handling multiple data sources within a platform can feel more
    /// natural; you can also send all changes in one entry if you like.
    async fn platform_data(
        &self,
    ) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>>;
}

/// Support for user data providers can be added by implementing this trait, and adding an
/// additional binary using the implementor and common functions below.
#[async_trait]
pub trait UserDataProvider {
    /// Optionally return a SettingsJson object if user data is found, representing the settings to
    /// send to the API.
    async fn user_data(
        &self,
    ) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>>;
}

/// Run a user data provider, returning the proper exit code and errors, and if successful,
/// printing its JSON to stdout.
pub async fn run_userdata_provider(provider: &impl UserDataProvider) -> ExitCode {
    let (exit_code, output) = match provider.user_data().await {
        Ok(Some(user_data)) => match serde_json::to_string(&user_data) {
            Ok(json) => (ExitCode::SUCCESS, json),
            Err(e) => (
                ExitCode::FAILURE,
                format!("Failed to serialize user data as JSON: {}", e),
            ),
        },
        Ok(None) => (ExitCode::SUCCESS, String::new()),
        Err(e) => (ExitCode::FAILURE, format!("{}", e)),
    };

    println!("{}", output);
    exit_code
}

/// Convenience function to set up logging for provider binaries.
///
/// Since provider binaries return their output to early-boot-config on stdout, we want to make
/// sure all logging happens to stderr.  For debugging purposes, the binaries' log level may be
/// configured via environment variable.
pub fn setup_provider_logging() {
    // Filter at info level by default unless configured via environment variable
    let log_level = Env::default().filter_or(LOG_LEVEL_ENV_VAR, "info");
    env_logger::Builder::from_env(log_level)
        .format_module_path(false)
        .target(Target::Stderr)
        .write_style(WriteStyle::Never)
        .init()
}

/// Read user data from a given path, decompressing if necessary
fn user_data_from_file<P: AsRef<Path>>(
    path: P,
) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
    let path = path.as_ref();

    if !path.exists() {
        info!("{} does not exist, not using it", path.display());
        return Ok(None);
    }
    info!("'{}' exists, using it", path.display());

    // Read the file, decompressing it if compressed.
    let user_data_str = expand_file_maybe(path).context(error::InputFileReadSnafu { path })?;

    if user_data_str.is_empty() {
        warn!("{} exists but is empty", path.display());
        return Ok(None);
    }

    trace!("Received user data: {}", user_data_str);
    let desc = format!("user data from {}", path.display());
    let json = SettingsJson::from_toml_str(&user_data_str, desc)
        .context(error::SettingsToJSONSnafu { from: path })?;

    Ok(Some(json))
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to serialize settings from {}: {}", from.display(), source))]
        SettingsToJSON {
            from: PathBuf,
            source: crate::settings::Error,
        },
    }
}
