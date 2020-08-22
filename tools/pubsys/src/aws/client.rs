use crate::config::AwsConfig;
use async_trait::async_trait;
use rusoto_core::{request::DispatchSignedRequest, HttpClient, Region};
use rusoto_credential::{
    AutoRefreshingProvider, AwsCredentials, CredentialsError, DefaultCredentialsProvider,
    ProfileProvider, ProvideAwsCredentials,
};
use rusoto_ebs::EbsClient;
use rusoto_ec2::Ec2Client;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use snafu::ResultExt;

pub(crate) trait NewWith {
    fn new_with<P, D>(request_dispatcher: D, credentials_provider: P, region: Region) -> Self
    where
        P: ProvideAwsCredentials + Send + Sync + 'static,
        D: DispatchSignedRequest + Send + Sync + 'static;
}

impl NewWith for EbsClient {
    fn new_with<P, D>(request_dispatcher: D, credentials_provider: P, region: Region) -> Self
    where
        P: ProvideAwsCredentials + Send + Sync + 'static,
        D: DispatchSignedRequest + Send + Sync + 'static,
    {
        Self::new_with(request_dispatcher, credentials_provider, region)
    }
}

impl NewWith for Ec2Client {
    fn new_with<P, D>(request_dispatcher: D, credentials_provider: P, region: Region) -> Self
    where
        P: ProvideAwsCredentials + Send + Sync + 'static,
        D: DispatchSignedRequest + Send + Sync + 'static,
    {
        Self::new_with(request_dispatcher, credentials_provider, region)
    }
}

/// Create a rusoto client of the given type using the given region and configuration.
pub(crate) fn build_client<T: NewWith>(
    region: &Region,
    sts_region: &Region,
    aws: &AwsConfig,
) -> Result<T> {
    let maybe_regional_role = aws.region.get(region.name()).and_then(|r| r.role.clone());
    let assume_roles = aws.role.iter().chain(maybe_regional_role.iter()).cloned();
    let provider = build_provider(&sts_region, assume_roles.clone(), base_provider(&aws.profile)?)?;
    Ok(T::new_with(
        rusoto_core::HttpClient::new().context(error::HttpClient)?,
        provider,
        region.clone(),
    ))
}

/// Wrapper for trait object that implements ProvideAwsCredentials to simplify return values.
/// Might be able to remove if rusoto implements ProvideAwsCredentials for
/// Box<ProvideAwsCredentials>.
struct CredentialsProvider(Box<dyn ProvideAwsCredentials + Send + Sync + 'static>);
#[async_trait]
impl ProvideAwsCredentials for CredentialsProvider {
    async fn credentials(&self) -> std::result::Result<AwsCredentials, CredentialsError> {
        self.0.credentials().await
    }
}

/// Chains credentials providers to assume the given roles in order.
/// The region given should be the one in which you want to talk to STS to get temporary
/// credentials, not the region in which you want to talk to a service endpoint like EC2.  This is
/// needed because you may be assuming a role in an opt-in region from an account that has not
/// opted-in to that region, and you need to get session credentials from an STS endpoint in a
/// region to which you have access in the base account.
fn build_provider<P>(
    sts_region: &Region,
    assume_roles: impl Iterator<Item = String>,
    base_provider: P,
) -> Result<CredentialsProvider>
where
    P: ProvideAwsCredentials + Send + Sync + 'static,
{
    let mut provider = CredentialsProvider(Box::new(base_provider));
    for assume_role in assume_roles {
        let sts = StsClient::new_with(
            HttpClient::new().context(error::HttpClient)?,
            provider,
            sts_region.clone(),
        );
        let expiring_provider = StsAssumeRoleSessionCredentialsProvider::new(
            sts,
            assume_role,
            "pubsys".to_string(), // session name
            None,                 // external ID
            None,                 // session duration
            None,                 // scope down policy
            None,                 // MFA serial
        );
        provider = CredentialsProvider(Box::new(
            AutoRefreshingProvider::new(expiring_provider).context(error::Provider)?,
        ));
    }
    Ok(provider)
}

/// If the user specified a profile, have rusoto use that, otherwise use Rusoto's default
/// credentials mechanisms.
fn base_provider(maybe_profile: &Option<String>) -> Result<CredentialsProvider> {
    if let Some(profile) = maybe_profile {
        let mut p = ProfileProvider::new().context(error::Provider)?;
        p.set_profile(profile);
        Ok(CredentialsProvider(Box::new(p)))
    } else {
        Ok(CredentialsProvider(Box::new(
            DefaultCredentialsProvider::new().context(error::Provider)?,
        )))
    }
}

pub(crate) mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Failed to create HTTP client: {}", source))]
        HttpClient {
            source: rusoto_core::request::TlsError,
        },

        #[snafu(display("Failed to create AWS credentials provider: {}", source))]
        Provider {
            source: rusoto_credential::CredentialsError,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
