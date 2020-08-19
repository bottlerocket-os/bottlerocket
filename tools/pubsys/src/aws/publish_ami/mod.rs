//! The publish_ami module owns the 'publish-ami' subcommand and controls the process of granting
//! and revoking public access to EC2 AMIs.

use crate::aws::{client::build_client, region_from_string};
use crate::config::InfraConfig;
use crate::Args;
use futures::future::{join, ready};
use futures::stream::{self, StreamExt};
use log::{debug, error, info, trace};
use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{
    DescribeImagesError, DescribeImagesRequest, DescribeImagesResult, Ec2, Ec2Client,
    ModifyImageAttributeError, ModifyImageAttributeRequest, ModifySnapshotAttributeError,
    ModifySnapshotAttributeRequest,
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
pub(crate) struct PublishArgs {
    /// Path to the JSON file containing regional AMI IDs to modify
    #[structopt(long)]
    ami_input: PathBuf,

    /// Comma-separated list of regions to publish in, overriding Infra.toml; given regions must be
    /// in the --ami-input file
    #[structopt(long, use_delimiter = true)]
    regions: Vec<String>,

    /// Make the AMIs public
    #[structopt(long, group = "mode")]
    make_public: bool,
    /// Make the AMIs private
    #[structopt(long, group = "mode")]
    make_private: bool,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, publish_args: &PublishArgs) -> Result<()> {
    let (operation, mode) = if publish_args.make_public {
        ("add".to_string(), "public")
    } else if publish_args.make_private {
        ("remove".to_string(), "private")
    } else {
        unreachable!("developer error: make-public and make-private not required/exclusive");
    };

    info!(
        "Using AMI data from path: {}",
        publish_args.ami_input.display()
    );
    let file = File::open(&publish_args.ami_input).context(error::File {
        op: "open",
        path: &publish_args.ami_input,
    })?;
    let mut ami_input: HashMap<String, String> =
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

    info!(
        "Using infra config from path: {}",
        args.infra_config_path.display()
    );
    let infra_config = InfraConfig::from_path(&args.infra_config_path).context(error::Config)?;
    trace!("Parsed infra config: {:?}", infra_config);

    let aws = infra_config.aws.unwrap_or_else(Default::default);

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let regions = if !publish_args.regions.is_empty() {
        publish_args.regions.clone()
    } else {
        aws.regions.clone().into()
    };
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
        let ami_id = ami_input
            .remove(&name)
            // This could only happen if someone removes the check above...
            .with_context(|| error::UnknownRegions {
                regions: vec![name.clone()],
            })?;
        let region = region_from_string(&name, &aws).context(error::ParseRegion)?;
        amis.insert(region, ami_id);
    }

    // We make a map storing our regional clients because they're used in a future and need to
    // live until the future is resolved.
    let mut ec2_clients = HashMap::with_capacity(amis.len());
    for region in amis.keys() {
        let ec2_client = build_client::<Ec2Client>(&region, &aws).context(error::Client {
            client_type: "EC2",
            region: region.name(),
        })?;
        ec2_clients.insert(region.clone(), ec2_client);
    }

    let snapshots = get_snapshots(&amis, &ec2_clients).await?;
    trace!("Found snapshots: {:?}", snapshots);

    info!("Updating snapshot permissions - making {}", mode);
    modify_snapshots(&snapshots, &ec2_clients, operation.clone()).await?;
    info!("Updating image permissions - making {}", mode);
    modify_images(&amis, &ec2_clients, operation.clone()).await?;

    Ok(())
}

/// Returns a regional mapping of snapshot IDs associated with the given AMIs.
async fn get_snapshots(
    amis: &HashMap<Region, String>,
    clients: &HashMap<Region, Ec2Client>,
) -> Result<HashMap<Region, Vec<String>>> {
    // Build requests for image information.
    let mut describe_requests = Vec::with_capacity(amis.len());
    for (region, image_id) in amis {
        let ec2_client = &clients[region];
        let describe_request = DescribeImagesRequest {
            image_ids: Some(vec![image_id.to_string()]),
            ..Default::default()
        };
        let describe_future = ec2_client.describe_images(describe_request);

        // Store the region and image ID so we can include it in errors
        let info_future = ready((region.clone(), image_id.clone()));
        describe_requests.push(join(info_future, describe_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(describe_requests).buffer_unordered(4);
    let describe_responses: Vec<(
        (Region, String),
        std::result::Result<DescribeImagesResult, RusotoError<DescribeImagesError>>,
    )> = request_stream.collect().await;

    // For each described image, get the snapshot IDs from the block device mappings.
    let mut snapshots = HashMap::with_capacity(amis.len());
    for ((region, image_id), describe_response) in describe_responses {
        // Get the image description, ensuring we only have one.
        let describe_response = describe_response.context(error::DescribeImages {
            region: region.name(),
        })?;
        let mut images = describe_response.images.context(error::MissingInResponse {
            request_type: "DescribeImages",
            missing: "images",
        })?;
        ensure!(
            !images.is_empty(),
            error::MissingImage {
                region: region.name(),
                image_id,
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
        snapshots.insert(region, snapshot_ids);
    }

    Ok(snapshots)
}

/// Modify snapshot attributes to make them public/private as requested.
async fn modify_snapshots(
    snapshots: &HashMap<Region, Vec<String>>,
    clients: &HashMap<Region, Ec2Client>,
    operation: String,
) -> Result<()> {
    // Build requests to modify snapshot attributes.
    let mut modify_snapshot_requests = Vec::new();
    for (region, snapshot_ids) in snapshots {
        for snapshot_id in snapshot_ids {
            let ec2_client = &clients[region];
            let modify_snapshot_request = ModifySnapshotAttributeRequest {
                attribute: Some("createVolumePermission".to_string()),
                group_names: Some(vec!["all".to_string()]),
                operation_type: Some(operation.clone()),
                snapshot_id: snapshot_id.clone(),
                ..Default::default()
            };
            let modify_snapshot_future =
                ec2_client.modify_snapshot_attribute(modify_snapshot_request);

            // Store the region and snapshot ID so we can include it in errors
            let info_future = ready((region.name().to_string(), snapshot_id.clone()));
            modify_snapshot_requests.push(join(info_future, modify_snapshot_future));
        }
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(modify_snapshot_requests).buffer_unordered(4);
    let modify_snapshot_responses: Vec<(
        (String, String),
        std::result::Result<(), RusotoError<ModifySnapshotAttributeError>>,
    )> = request_stream.collect().await;

    // Count up successes and failures so we can give a clear total in the final error message.
    let mut error_count = 0u16;
    let mut success_count = 0u16;
    for ((region, snapshot_id), modify_snapshot_response) in modify_snapshot_responses {
        match modify_snapshot_response {
            Ok(()) => {
                success_count += 1;
                debug!(
                    "Modified permissions of snapshot {} in {}",
                    snapshot_id, region,
                );
            }
            Err(e) => {
                error_count += 1;
                error!(
                    "Modifying permissions of {} in {} failed: {}",
                    snapshot_id, region, e
                );
            }
        }
    }

    ensure!(
        error_count == 0,
        error::ModifySnapshotAttribute {
            error_count,
            success_count,
        }
    );

    Ok(())
}

/// Modify image attributes to make them public/private as requested.
async fn modify_images(
    images: &HashMap<Region, String>,
    clients: &HashMap<Region, Ec2Client>,
    operation: String,
) -> Result<()> {
    // Build requests to modify image attributes.
    let mut modify_image_requests = Vec::new();
    for (region, image_id) in images {
        let ec2_client = &clients[region];
        let modify_image_request = ModifyImageAttributeRequest {
            attribute: Some("launchPermission".to_string()),
            user_groups: Some(vec!["all".to_string()]),
            operation_type: Some(operation.clone()),
            image_id: image_id.clone(),
            ..Default::default()
        };
        let modify_image_future = ec2_client.modify_image_attribute(modify_image_request);

        // Store the region and image ID so we can include it in errors
        let info_future = ready((region.name().to_string(), image_id.clone()));
        modify_image_requests.push(join(info_future, modify_image_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    let request_stream = stream::iter(modify_image_requests).buffer_unordered(4);
    let modify_image_responses: Vec<(
        (String, String),
        std::result::Result<(), RusotoError<ModifyImageAttributeError>>,
    )> = request_stream.collect().await;

    // Count up successes and failures so we can give a clear total in the final error message.
    let mut error_count = 0u16;
    let mut success_count = 0u16;
    for ((region, image_id), modify_image_response) in modify_image_responses {
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
        error::ModifyImageAttribute {
            error_count,
            success_count,
        }
    );

    Ok(())
}

mod error {
    use crate::aws;
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
            source: crate::config::Error,
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
            "Failed to modify permissions of {} of {} images",
            error_count, error_count + success_count,
        ))]
        ModifyImageAttribute {
            error_count: u16,
            success_count: u16,
        },

        #[snafu(display(
            "Failed to modify permissions of {} of {} snapshots",
            error_count, error_count + success_count,
        ))]
        ModifySnapshotAttribute {
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
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
