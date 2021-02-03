use crate::aws::client::build_client;
use log::info;
use pubsys_config::AwsConfig;
use rusoto_core::Region;
use rusoto_ec2::{DescribeImagesRequest, Ec2, Ec2Client};
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
    aws: &AwsConfig,
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
            error::MaxAttempts {
                id,
                max_attempts,
                region: region.name()
            }
        );

        let describe_request = DescribeImagesRequest {
            image_ids: Some(vec![id.to_string()]),
            ..Default::default()
        };
        // Use a new client each time so we have more confidence that different endpoints can see
        // the new AMI.
        let ec2_client =
            build_client::<Ec2Client>(&region, &sts_region, &aws).context(error::Client {
                client_type: "EC2",
                region: region.name(),
            })?;
        let describe_response =
            ec2_client
                .describe_images(describe_request)
                .await
                .context(error::DescribeImages {
                    region: region.name(),
                })?;
        // The response contains an Option<Vec<Image>>, so we have to check that we got a
        // list at all, and then that the list contains the ID in question.
        if let Some(images) = describe_response.images {
            let mut saw_it = false;
            for image in images {
                if let Some(ref found_id) = image.image_id {
                    if let Some(ref found_state) = image.state {
                        if id == found_id && state == found_state {
                            // Success; check if we have enough to declare victory.
                            saw_it = true;
                            successes += 1;
                            if successes >= successes_required {
                                info!("Found {} {} in {}", id, state, region.name());
                                return Ok(());
                            }
                            break;
                        }
                        // If the state shows us the AMI failed, we know we'll never hit the
                        // desired state.  (Unless they desired "error", which will be caught
                        // above.)
                        ensure!(
                            !["invalid", "deregistered", "failed", "error"]
                                .iter()
                                .any(|e| e == found_state),
                            error::State {
                                id,
                                state: found_state,
                                region: region.name()
                            }
                        );
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
                id,
                region.name(),
                state,
                attempts,
                max_attempts
            );
        }
        sleep(Duration::from_secs(seconds_between_attempts));
    }
}

mod error {
    use crate::aws;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Error creating {} client in {}: {}", client_type, region, source))]
        Client {
            client_type: String,
            region: String,
            source: aws::client::Error,
        },

        #[snafu(display("Failed to describe images in {}: {}", region, source))]
        DescribeImages {
            region: String,
            source: rusoto_core::RusotoError<rusoto_ec2::DescribeImagesError>,
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
