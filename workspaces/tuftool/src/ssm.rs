#![cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]

use crate::error::{self, Result};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::ProfileProvider;
use rusoto_ssm::SsmClient;
use snafu::ResultExt;
use std::env;
use std::str::FromStr;

/// Builds an SSM client for a given profile name.
///
/// This **cannot** be called concurrently as it modifies environment variables (due to Rusoto's
/// inflexibility for determining the region given a profile name).
//
// A better explanation: we want to know what region to make SSM calls in based on ~/.aws/config,
// but `ProfileProvider::region` is an associated function, not a method; this means we can't tell
// it what profile to select the region for.
//
// However, `region` calls `ProfileProvider::default_profile_name`, which uses the `AWS_PROFILE`
// environment variable. So we set that :(
//
// This behavior should be better supported in `rusoto_credential`
// TODO(iliana): submit issue + PR upstream
pub(crate) fn build_client(profile: Option<&str>) -> Result<SsmClient> {
    Ok(if let Some(profile) = profile {
        let mut provider = ProfileProvider::new().context(error::RusotoCreds)?;
        provider.set_profile(profile);

        let profile_prev = env::var_os("AWS_PROFILE");
        env::set_var("AWS_PROFILE", profile);
        let region = ProfileProvider::region().context(error::RusotoCreds)?;
        match profile_prev {
            Some(v) => env::set_var("AWS_PROFILE", v),
            None => env::remove_var("AWS_PROFILE"),
        }

        SsmClient::new_with(
            HttpClient::new().context(error::RusotoTls)?,
            provider,
            match region {
                Some(region) => {
                    Region::from_str(&region).context(error::RusotoRegion { region })?
                }
                None => Region::default(),
            },
        )
    } else {
        SsmClient::new(Region::default())
    })
}
