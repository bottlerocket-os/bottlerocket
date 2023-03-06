use aws_config::default_provider::credentials::default_provider;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::sts::AssumeRoleProvider;
use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_types::region::Region;
use pubsys_config::AwsConfig as PubsysAwsConfig;

/// Create an AWS client config using the given regions and pubsys config.
pub(crate) async fn build_client_config(
    region: &Region,
    sts_region: &Region,
    pubsys_aws_config: &PubsysAwsConfig,
) -> SdkConfig {
    let maybe_profile = pubsys_aws_config.profile.clone();
    let maybe_role = pubsys_aws_config.role.clone();
    let maybe_regional_role = pubsys_aws_config
        .region
        .get(region.as_ref())
        .and_then(|r| r.role.clone());
    let base_provider = base_provider(&maybe_profile).await;

    let config = match (&maybe_role, &maybe_regional_role) {
        (None, None) => aws_config::from_env().credentials_provider(base_provider),
        _ => {
            let assume_roles = maybe_role.iter().chain(maybe_regional_role.iter()).cloned();
            let provider =
                build_provider(sts_region, assume_roles.clone(), base_provider.clone()).await;
            aws_config::from_env().credentials_provider(provider)
        }
    };

    config.region(region.clone()).load().await
}

/// Chains credentials providers to assume the given roles in order.
/// The region given should be the one in which you want to talk to STS to get temporary
/// credentials, not the region in which you want to talk to a service endpoint like EC2.  This is
/// needed because you may be assuming a role in an opt-in region from an account that has not
/// opted-in to that region, and you need to get session credentials from an STS endpoint in a
/// region to which you have access in the base account
async fn build_provider(
    sts_region: &Region,
    assume_roles: impl Iterator<Item = String>,
    base_provider: SharedCredentialsProvider,
) -> SharedCredentialsProvider {
    let mut provider = base_provider;
    for assume_role in assume_roles {
        provider = SharedCredentialsProvider::new(
            AssumeRoleProvider::builder(assume_role)
                .region(sts_region.clone())
                .session_name("pubsys")
                .build(provider.clone()),
        )
    }
    provider
}

/// If the user specified a profile, use that, otherwise use the default
/// credentials mechanisms.
async fn base_provider(maybe_profile: &Option<String>) -> SharedCredentialsProvider {
    if let Some(profile) = maybe_profile {
        SharedCredentialsProvider::new(
            ProfileFileCredentialsProvider::builder()
                .profile_name(profile)
                .build(),
        )
    } else {
        SharedCredentialsProvider::new(default_provider().await)
    }
}
