//! The publish_ami module owns the 'publish-ami' subcommand and controls the process of granting
//! and revoking access to EC2 AMIs.

use crate::aws::ami::wait::{self, wait_for_ami};
use crate::aws::ami::Image;
use crate::aws::client::build_client;
use crate::aws::region_from_string;
use crate::Args;
use futures::future::{join, ready};
use futures::stream::{self, StreamExt};
use log::{debug, error, info, trace};
use pubsys_config::InfraConfig;
use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{
    DescribeImagesRequest, Ec2, Ec2Client, ModifyImageAttributeRequest,
    ModifySnapshotAttributeError, ModifySnapshotAttributeRequest,
};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::StructOpt;

/// Grants or revokes permissions to Bottlerocket AMIs
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
#[structopt(group = clap::ArgGroup::with_name("mode").required(true).multiple(false))]
#[structopt(group = clap::ArgGroup::with_name("who").required(true).multiple(true))]
pub(crate) struct PublishArgs {
    /// Path to the JSON file containing regional AMI IDs to modify
    #[structopt(long)]
    ami_input: PathBuf,

    /// Comma-separated list of regions to publish in, overriding Infra.toml; given regions must be
    /// in the --ami-input file
    #[structopt(long, use_delimiter = true)]
    regions: Vec<String>,

    /// Grant access to the given users/groups
    #[structopt(long, group = "mode")]
    grant: bool,
    /// Revoke access from the given users/groups
    #[structopt(long, group = "mode")]
    revoke: bool,

    /// User IDs to give/remove access
    #[structopt(long, use_delimiter = true, group = "who")]
    user_ids: Vec<String>,
    /// Group names to give/remove access
    #[structopt(long, use_delimiter = true, group = "who")]
    group_names: Vec<String>,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, publish_args: &PublishArgs) -> Result<()> {
    let (operation, description) = if publish_args.grant {
        ("add".to_string(), "granting access")
    } else if publish_args.revoke {
        ("remove".to_string(), "revoking access")
    } else {
        unreachable!("developer error: --grant and --revoke not required/exclusive");
    };

    info!(
        "Using AMI data from path: {}",
        publish_args.ami_input.display()
    );
    let file = File::open(&publish_args.ami_input).context(error::File {
        op: "open",
        path: &publish_args.ami_input,
    })?;
    let mut ami_input: HashMap<String, Image> =
        serde_json::from_reader(file).context(error::Deserialize {
            path: &publish_args.ami_input,
        })?;
    trace!("Parsed AMI input: {:?}", ami_input);

    // pubsys will not create a file if it did not create AMIs, so we should only have an empty
    // file if a user created one manually, and they shouldn't be creating an empty file.
    ensure!(
        !ami_input.is_empty(),
        error::Input {
            path: &publish_args.ami_input
        }
    );

    // If a lock file exists, use that, otherwise use Infra.toml or default
    let infra_config =
        InfraConfig::from_path_or_lock(&args.infra_config_path, true).context(error::Config)?;
    trace!("Using infra config: {:?}", infra_config);

    let aws = infra_config.aws.unwrap_or_else(Default::default);

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let regions = if !publish_args.regions.is_empty() {
        publish_args.regions.clone()
    } else {
        aws.regions.clone().into()
    };
    ensure!(
        !regions.is_empty(),
        error::MissingConfig {
            missing: "aws.regions"
        }
    );
    let base_region = region_from_string(&regions[0], &aws).context(error::ParseRegion)?;

    // Check that the requested regions are a subset of the regions we *could* publish from the AMI
    // input JSON.
    let requested_regions = HashSet::from_iter(regions.iter());
    let known_regions = HashSet::<&String>::from_iter(ami_input.keys());
    ensure!(
        requested_regions.is_subset(&known_regions),
        error::UnknownRegions {
            regions: requested_regions
                .difference(&known_regions)
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        }
    );

    // Parse region names, adding endpoints from InfraConfig if specified
    let mut amis = HashMap::with_capacity(regions.len());
    for name in regions {
        let image = ami_input
            .remove(&name)
            // This could only happen if someone removes the check above...
            .with_context(|| error::UnknownRegions {
                regions: vec![name.clone()],
            })?;
        let region = region_from_string(&name, &aws).context(error::ParseRegion)?;
        amis.insert(region, image);
    }

    // We make a map storing our regional clients because they're used in a future and need to
    // live until the future is resolved.
    let mut ec2_clients = HashMap::with_capacity(amis.len());
    for region in amis.keys() {
        let ec2_client =
            build_client::<Ec2Client>(&region, &base_region, &aws).context(error::Client {
                client_type: "EC2",
                region: region.name(),
            })?;
        ec2_clients.insert(region.clone(), ec2_client);
    }

    // If AMIs aren't in "available" state, we can get a DescribeImages response that includes
    // most of the data we need, but not snapshot IDs.
    info!("Waiting for AMIs to be available...");
    let mut wait_requests = Vec::with_capacity(amis.len());
    for (region, image) in &amis {
        let wait_future = wait_for_ami(&image.id, &region, &base_region, "available", 1, &aws);
        // Store the region and ID so we can include it in errors
        let info_future = ready((region.clone(), image.id.clone()));
        wait_requests.push(join(info_future, wait_future));
    }
    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(wait_requests).buffer_unordered(4);
    let wait_responses: Vec<((Region, String), std::result::Result<(), wait::Error>)> =
        request_stream.collect().await;

    // Make sure waits succeeded and AMIs are available.
    for ((region, image_id), wait_response) in wait_responses {
        wait_response.context(error::WaitAmi {
            id: &image_id,
            region: region.name(),
        })?;
    }

    let snapshots = get_regional_snapshots(&amis, &ec2_clients).await?;
    trace!("Found snapshots: {:?}", snapshots);

    info!("Updating snapshot permissions - {}", description);
    modify_regional_snapshots(
        Some(publish_args.user_ids.clone()),
        Some(publish_args.group_names.clone()),
        &operation,
        &snapshots,
        &ec2_clients,
    )
    .await?;

    info!("Updating image permissions - {}", description);
    let ami_ids = amis
        .into_iter()
        .map(|(region, image)| (region, image.id))
        .collect();
    modify_regional_images(
        Some(publish_args.user_ids.clone()),
        Some(publish_args.group_names.clone()),
        &operation,
        &ami_ids,
        &ec2_clients,
    )
    .await?;

    Ok(())
}

/// Returns the snapshot IDs associated with the given AMI.
pub(crate) async fn get_snapshots(
    image_id: &str,
    region: &Region,
    ec2_client: &Ec2Client,
) -> Result<Vec<String>> {
    let describe_request = DescribeImagesRequest {
        image_ids: Some(vec![image_id.to_string()]),
        ..Default::default()
    };
    let describe_response = ec2_client.describe_images(describe_request).await;
    let describe_response = describe_response.context(error::DescribeImages {
        region: region.name(),
    })?;

    // Get the image description, ensuring we only have one.
    let mut images = describe_response.images.context(error::MissingInResponse {
        request_type: "DescribeImages",
        missing: "images",
    })?;
    ensure!(
        !images.is_empty(),
        error::MissingImage {
            region: region.name(),
            image_id: image_id.to_string(),
        }
    );
    ensure!(
        images.len() == 1,
        error::MultipleImages {
            region: region.name(),
            images: images
                .into_iter()
                .map(|i| i.image_id.unwrap_or_else(|| "<missing>".to_string()))
                .collect::<Vec<_>>()
        }
    );
    let image = images.remove(0);

    // Look into the block device mappings for snapshots.
    let bdms = image
        .block_device_mappings
        .context(error::MissingInResponse {
            request_type: "DescribeImages",
            missing: "block_device_mappings",
        })?;
    ensure!(
        !bdms.is_empty(),
        error::MissingInResponse {
            request_type: "DescribeImages",
            missing: "non-empty block_device_mappings"
        }
    );
    let mut snapshot_ids = Vec::with_capacity(bdms.len());
    for bdm in bdms {
        let ebs = bdm.ebs.context(error::MissingInResponse {
            request_type: "DescribeImages",
            missing: "ebs in block_device_mappings",
        })?;
        let snapshot_id = ebs.snapshot_id.context(error::MissingInResponse {
            request_type: "DescribeImages",
            missing: "snapshot_id in block_device_mappings.ebs",
        })?;
        snapshot_ids.push(snapshot_id);
    }

    Ok(snapshot_ids)
}

/// Returns a regional mapping of snapshot IDs associated with the given AMIs.
async fn get_regional_snapshots(
    amis: &HashMap<Region, Image>,
    clients: &HashMap<Region, Ec2Client>,
) -> Result<HashMap<Region, Vec<String>>> {
    // Build requests for image information.
    let mut snapshots_requests = Vec::with_capacity(amis.len());
    for (region, image) in amis {
        let ec2_client = &clients[region];

        let snapshots_future = get_snapshots(&image.id, region, ec2_client);

        // Store the region so we can include it in errors
        let info_future = ready(region.clone());
        snapshots_requests.push(join(info_future, snapshots_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(snapshots_requests).buffer_unordered(4);
    let snapshots_responses: Vec<(Region, Result<Vec<String>>)> = request_stream.collect().await;

    // For each described image, get the snapshot IDs from the block device mappings.
    let mut snapshots = HashMap::with_capacity(amis.len());
    for (region, snapshot_ids) in snapshots_responses {
        let snapshot_ids = snapshot_ids?;
        snapshots.insert(region, snapshot_ids);
    }

    Ok(snapshots)
}

/// Modify createVolumePermission for the given users/groups on the given snapshots.  The
/// `operation` should be "add" or "remove" to allow/deny permission.
pub(crate) async fn modify_snapshots(
    user_ids: Option<Vec<String>>,
    group_names: Option<Vec<String>>,
    operation: &str,
    snapshot_ids: &[String],
    ec2_client: &Ec2Client,
    region: &Region,
) -> Result<()> {
    let mut requests = Vec::new();
    for snapshot_id in snapshot_ids {
        let request = ModifySnapshotAttributeRequest {
            attribute: Some("createVolumePermission".to_string()),
            user_ids: user_ids.clone(),
            group_names: group_names.clone(),
            operation_type: Some(operation.to_string()),
            snapshot_id: snapshot_id.clone(),
            ..Default::default()
        };
        let response_future = ec2_client.modify_snapshot_attribute(request);
        // Store the snapshot_id so we can include it in any errors
        let info_future = ready(snapshot_id.to_string());
        requests.push(join(info_future, response_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(requests).buffer_unordered(4);
    let responses: Vec<(
        String,
        std::result::Result<(), RusotoError<ModifySnapshotAttributeError>>,
    )> = request_stream.collect().await;

    for (snapshot_id, response) in responses {
        response.context(error::ModifyImageAttribute {
            snapshot_id,
            region: region.name(),
        })?
    }

    Ok(())
}

/// Modify createVolumePermission for the given users/groups, across all of the snapshots in the
/// given regional mapping.  The `operation` should be "add" or "remove" to allow/deny permission.
pub(crate) async fn modify_regional_snapshots(
    user_ids: Option<Vec<String>>,
    group_names: Option<Vec<String>>,
    operation: &str,
    snapshots: &HashMap<Region, Vec<String>>,
    clients: &HashMap<Region, Ec2Client>,
) -> Result<()> {
    // Build requests to modify snapshot attributes.
    let mut requests = Vec::new();
    for (region, snapshot_ids) in snapshots {
        let ec2_client = &clients[region];
        let modify_snapshot_future = modify_snapshots(
            user_ids.clone(),
            group_names.clone(),
            operation,
            snapshot_ids,
            ec2_client,
            region,
        );

        // Store the region and snapshot ID so we can include it in errors
        let info_future = ready((region.clone(), snapshot_ids.clone()));
        requests.push(join(info_future, modify_snapshot_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(requests).buffer_unordered(4);
    let responses: Vec<((Region, Vec<String>), Result<()>)> = request_stream.collect().await;

    // Count up successes and failures so we can give a clear total in the final error message.
    let mut error_count = 0u16;
    let mut success_count = 0u16;
    for ((region, snapshot_ids), response) in responses {
        match response {
            Ok(()) => {
                success_count += 1;
                debug!(
                    "Modified permissions in {} for snapshots [{}]",
                    region.name(),
                    snapshot_ids.join(", "),
                );
            }
            Err(e) => {
                error_count += 1;
                error!(
                    "Failed to modify permissions in {} for snapshots [{}]: {}",
                    region.name(),
                    snapshot_ids.join(", "),
                    e
                );
            }
        }
    }

    ensure!(
        error_count == 0,
        error::ModifySnapshotAttributes {
            error_count,
            success_count,
        }
    );

    Ok(())
}

/// Modify launchPermission for the given users/groups on the given images.  The `operation`
/// should be "add" or "remove" to allow/deny permission.
pub(crate) async fn modify_image(
    user_ids: Option<Vec<String>>,
    user_groups: Option<Vec<String>>,
    operation: &str,
    image_id: &str,
    ec2_client: &Ec2Client,
    region: &Region,
) -> Result<()> {
    // Build requests to modify image attributes.
    let modify_image_request = ModifyImageAttributeRequest {
        attribute: Some("launchPermission".to_string()),
        user_ids: user_ids.clone(),
        user_groups: user_groups.clone(),
        operation_type: Some(operation.to_string()),
        image_id: image_id.to_string(),
        ..Default::default()
    };
    ec2_client
        .modify_image_attribute(modify_image_request)
        .await
        .context(error::ModifyImageAttributes {
            image_id,
            region: region.name(),
        })
}

/// Modify launchPermission for the given users/groups, across all of the images in the given
/// regional mapping.  The `operation` should be "add" or "remove" to allow/deny permission.
pub(crate) async fn modify_regional_images(
    user_ids: Option<Vec<String>>,
    user_groups: Option<Vec<String>>,
    operation: &str,
    images: &HashMap<Region, String>,
    clients: &HashMap<Region, Ec2Client>,
) -> Result<()> {
    let mut requests = Vec::new();
    for (region, image_id) in images {
        let ec2_client = &clients[region];

        let modify_image_future = modify_image(
            user_ids.clone(),
            user_groups.clone(),
            operation,
            image_id,
            ec2_client,
            region,
        );

        // Store the region and image ID so we can include it in errors
        let info_future = ready((region.name().to_string(), image_id.clone()));
        requests.push(join(info_future, modify_image_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(requests).buffer_unordered(4);
    let responses: Vec<((String, String), Result<()>)> = request_stream.collect().await;

    // Count up successes and failures so we can give a clear total in the final error message.
    let mut error_count = 0u16;
    let mut success_count = 0u16;
    for ((region, image_id), modify_image_response) in responses {
        match modify_image_response {
            Ok(()) => {
                success_count += 1;
                info!("Modified permissions of image {} in {}", image_id, region,);
            }
            Err(e) => {
                error_count += 1;
                error!(
                    "Modifying permissions of {} in {} failed: {}",
                    image_id, region, e
                );
            }
        }
    }

    ensure!(
        error_count == 0,
        error::ModifyImagesAttributes {
            error_count,
            success_count,
        }
    );

    Ok(())
}

mod error {
    use crate::aws::{self, ami};
    use rusoto_core::RusotoError;
    use rusoto_ec2::{ModifyImageAttributeError, ModifySnapshotAttributeError};
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Error creating {} client in {}: {}", client_type, region, source))]
        Client {
            client_type: String,
            region: String,
            source: aws::client::Error,
        },

        #[snafu(display("Error reading config: {}", source))]
        Config {
            source: pubsys_config::Error,
        },

        #[snafu(display("Failed to describe images in {}: {}", region, source))]
        DescribeImages {
            region: String,
            source: rusoto_core::RusotoError<rusoto_ec2::DescribeImagesError>,
        },

        #[snafu(display("Failed to deserialize input from '{}': {}", path.display(), source))]
        Deserialize {
            path: PathBuf,
            source: serde_json::Error,
        },

        #[snafu(display("Failed to {} '{}': {}", op, path.display(), source))]
        File {
            op: String,
            path: PathBuf,
            source: io::Error,
        },

        #[snafu(display("Input '{}' is empty", path.display()))]
        Input {
            path: PathBuf,
        },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig {
            missing: String,
        },

        #[snafu(display("Failed to find given AMI ID {} in {}", image_id, region))]
        MissingImage {
            region: String,
            image_id: String,
        },

        #[snafu(display("Response to {} was missing {}", request_type, missing))]
        MissingInResponse {
            request_type: String,
            missing: String,
        },

        #[snafu(display(
            "Failed to modify permissions of {} in {}: {}",
            snapshot_id,
            region,
            source
        ))]
        ModifyImageAttribute {
            snapshot_id: String,
            region: String,
            source: RusotoError<ModifySnapshotAttributeError>,
        },

        #[snafu(display(
            "Failed to modify permissions of {} of {} images",
            error_count, error_count + success_count,
        ))]
        ModifyImagesAttributes {
            error_count: u16,
            success_count: u16,
        },

        #[snafu(display(
            "Failed to modify permissions of {} in {}: {}",
            image_id,
            region,
            source
        ))]
        ModifyImageAttributes {
            image_id: String,
            region: String,
            source: RusotoError<ModifyImageAttributeError>,
        },

        #[snafu(display(
            "Failed to modify permissions of {} of {} snapshots",
            error_count, error_count + success_count,
        ))]
        ModifySnapshotAttributes {
            error_count: u16,
            success_count: u16,
        },

        #[snafu(display("DescribeImages in {} with unique filters returned multiple results: {}", region, images.join(", ")))]
        MultipleImages {
            region: String,
            images: Vec<String>,
        },

        ParseRegion {
            source: crate::aws::Error,
        },

        #[snafu(display(
            "Given region(s) in Infra.toml / regions argument that are not in --ami-input file: {}",
            regions.join(", ")
        ))]
        UnknownRegions {
            regions: Vec<String>,
        },

        #[snafu(display("AMI '{}' in {} did not become available: {}", id, region, source))]
        WaitAmi {
            id: String,
            region: String,
            source: ami::wait::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
