use crate::aws::client::build_client_config;
use aws_sdk_ec2::{config::Region, types::ImageState, Client as Ec2Client};
use log::info;
use pubsys_config::AwsConfig as PubsysAwsConfig;
use snafu::{ensure, ResultExt};
use std::thread::sleep;
use std::time::Duration;

/// Waits for the given AMI ID to reach the given state, requiring it be in that state for
/// `success_required` checks in a row.
pub(crate) async fn wait_for_ami(
    id: &str,
    region: &Region,
    sts_region: &Region,
    state: &str,
    successes_required: u8,
    pubsys_aws_config: &PubsysAwsConfig,
) -> Result<()> {
    let mut successes = 0;
    let max_attempts = 90;
    let mut attempts = 0;
    let seconds_between_attempts = 2;

    loop {
        attempts += 1;
        // Stop if we're over max, unless we're on a success streak, then give it some wiggle room.
        ensure!(
            (attempts - successes) <= max_attempts,
            error::MaxAttemptsSnafu {
                id,
                max_attempts,
                region: region.as_ref(),
            }
        );

        // Use a new client each time so we have more confidence that different endpoints can see
        // the new AMI.
        let client_config = build_client_config(region, sts_region, pubsys_aws_config).await;
        let ec2_client = Ec2Client::new(&client_config);
        let describe_response = ec2_client
            .describe_images()
            .set_image_ids(Some(vec![id.to_string()]))
            .send()
            .await
            .context(error::DescribeImagesSnafu {
                region: region.as_ref(),
            })?;

        // The response contains an Option<Vec<Image>>, so we have to check that we got a
        // list at all, and then that the list contains the ID in question.
        if let Some(images) = describe_response.images {
            let mut saw_it = false;
            for image in images {
                if let Some(found_id) = image.image_id {
                    if let Some(found_state) = image.state {
                        if id == found_id && ImageState::from(state) == found_state {
                            // Success; check if we have enough to declare victory.
                            saw_it = true;
                            successes += 1;
                            if successes >= successes_required {
                                info!("Found {} {} in {}", id, state, region);
                                return Ok(());
                            }
                            break;
                        }
                        // If the state shows us the AMI failed, we know we'll never hit the
                        // desired state.  (Unless they desired "error", which will be caught
                        // above.)
                        match &found_state {
                            ImageState::Invalid
                            | ImageState::Deregistered
                            | ImageState::Failed
                            | ImageState::Error => error::StateSnafu {
                                id,
                                state: found_state.as_ref(),
                                region: region.as_ref(),
                            }
                            .fail(),
                            _ => Ok(()),
                        }?;
                    }
                }
            }
            if !saw_it {
                // Did not find image in list; reset success count and try again (if we have spare attempts)
                successes = 0;
            }
        } else {
            // Did not receive list; reset success count and try again (if we have spare attempts)
            successes = 0;
        };

        if attempts % 5 == 1 {
            info!(
                "Waiting for {} in {} to be {}... (attempt {} of {})",
                id, region, state, attempts, max_attempts
            );
        }
        sleep(Duration::from_secs(seconds_between_attempts));
    }
}

mod error {
    use aws_sdk_ec2::error::SdkError;
    use aws_sdk_ec2::operation::describe_images::DescribeImagesError;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    #[allow(clippy::large_enum_variant)]
    pub(crate) enum Error {
        #[snafu(display("Failed to describe images in {}: {}", region, source))]
        DescribeImages {
            region: String,
            source: SdkError<DescribeImagesError>,
        },

        #[snafu(display(
            "Failed to reach desired state within {} attempts for {} in {}",
            max_attempts,
            id,
            region
        ))]
        MaxAttempts {
            max_attempts: u8,
            id: String,
            region: String,
        },

        #[snafu(display("Image '{}' went to '{}' state in {}", id, state, region))]
        State {
            id: String,
            state: String,
            region: String,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
