use aws_sdk_ec2::{
    types::{ImageAttributeName, LaunchPermission},
    Client as Ec2Client,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

/// Returns the launch permissions for the given AMI
pub(crate) async fn get_launch_permissions(
    ec2_client: &Ec2Client,
    region: &str,
    ami_id: &str,
) -> Result<Vec<LaunchPermissionDef>> {
    let ec2_response = ec2_client
        .describe_image_attribute()
        .image_id(ami_id)
        .attribute(ImageAttributeName::LaunchPermission)
        .send()
        .await
        .context(error::DescribeImageAttributeSnafu {
            ami_id,
            region: region.to_string(),
        })?;

    let mut launch_permissions = vec![];

    let responses: Vec<LaunchPermission> =
        ec2_response.launch_permissions().unwrap_or(&[]).to_vec();
    for permission in responses {
        launch_permissions.push(LaunchPermissionDef::try_from(permission)?)
    }
    Ok(launch_permissions)
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LaunchPermissionDef {
    /// The name of the group
    Group(String),

    /// The Amazon Web Services account ID
    UserId(String),

    /// The ARN of an organization
    OrganizationArn(String),

    /// The ARN of an organizational unit
    OrganizationalUnitArn(String),
}

impl TryFrom<LaunchPermission> for LaunchPermissionDef {
    type Error = crate::aws::ami::launch_permissions::Error;

    fn try_from(launch_permission: LaunchPermission) -> std::result::Result<Self, Self::Error> {
        let LaunchPermission {
            group,
            user_id,
            organization_arn,
            organizational_unit_arn,
            ..
        } = launch_permission.clone();
        match (group, user_id, organization_arn, organizational_unit_arn) {
            (Some(group), None, None, None) => {
                Ok(LaunchPermissionDef::Group(group.as_str().to_string()))
            }
            (None, Some(user_id), None, None) => Ok(LaunchPermissionDef::UserId(user_id)),
            (None, None, Some(organization_arn), None) => {
                Ok(LaunchPermissionDef::OrganizationArn(organization_arn))
            }
            (None, None, None, Some(organizational_unit_arn)) => Ok(
                LaunchPermissionDef::OrganizationalUnitArn(organizational_unit_arn),
            ),
            _ => Err(Error::InvalidLaunchPermission { launch_permission }),
        }
    }
}

mod error {
    use aws_sdk_ec2::error::SdkError;
    use aws_sdk_ec2::operation::describe_image_attribute::DescribeImageAttributeError;
    use aws_sdk_ec2::types::LaunchPermission;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Error describing AMI {} in {}: {}", ami_id, region, source))]
        DescribeImageAttribute {
            ami_id: String,
            region: String,
            #[snafu(source(from(SdkError<DescribeImageAttributeError>, Box::new)))]
            source: Box<SdkError<DescribeImageAttributeError>>,
        },

        #[snafu(display("Invalid launch permission: {:?}", launch_permission))]
        InvalidLaunchPermission { launch_permission: LaunchPermission },
    }
}
pub(crate) use error::Error;

type Result<T> = std::result::Result<T, error::Error>;
