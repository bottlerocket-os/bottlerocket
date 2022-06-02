//! The provider module owns the `PlatformDataProvider` trait

use crate::settings::SettingsJson;
use async_trait::async_trait;

mod local_file;

#[cfg(variant_platform = "aws")]
mod aws;
#[cfg(variant_platform = "aws")]
pub(crate) use aws::AwsDataProvider as Platform;

#[cfg(variant_platform = "vmware")]
mod vmware;
#[cfg(variant_platform = "vmware")]
pub(crate) use vmware::VmwareDataProvider as Platform;

#[cfg(variant_platform = "metal")]
mod metal;
#[cfg(variant_platform = "metal")]
pub(crate) use metal::MetalDataProvider as Platform;

/// Support for new platforms can be added by implementing this trait.
#[async_trait]
pub(crate) trait PlatformDataProvider {
    /// You should return a list of SettingsJson, representing the settings changes you want to
    /// send to the API.
    ///
    /// This is a list so that handling multiple data sources within a platform can feel more
    /// natural; you can also send all changes in one entry if you like.
    async fn platform_data(
        &self,
    ) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>>;
}
