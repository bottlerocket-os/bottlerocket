use super::{snapshot::snapshot_from_image, AmiArgs};
use coldsnap::{SnapshotUploader, SnapshotWaiter};
use log::{debug, info, warn};
use rusoto_ebs::EbsClient;
use rusoto_ec2::{
    BlockDeviceMapping, DeleteSnapshotRequest, DescribeImagesRequest, EbsBlockDevice, Ec2,
    Ec2Client, Filter, RegisterImageRequest,
};
use snafu::{ensure, OptionExt, ResultExt};

const ROOT_DEVICE_NAME: &str = "/dev/xvda";
const DATA_DEVICE_NAME: &str = "/dev/xvdb";

// Features we assume/enable for the images.
const VIRT_TYPE: &str = "hvm";
const VOLUME_TYPE: &str = "gp2";
const SRIOV: &str = "simple";
const ENA: bool = true;

#[derive(Debug)]
pub(crate) struct RegisteredIds {
    pub(crate) image_id: String,
    pub(crate) snapshot_ids: Vec<String>,
}

/// Helper for `register_image`.  Inserts registered snapshot IDs into `cleanup_snapshot_ids` so
/// they can be cleaned up on failure if desired.
async fn _register_image(
    ami_args: &AmiArgs,
    region: &str,
    ebs_client: EbsClient,
    ec2_client: &Ec2Client,
    cleanup_snapshot_ids: &mut Vec<String>,
) -> Result<RegisteredIds> {
    debug!(
        "Uploading root and data images into EBS snapshots in {}",
        region
    );
    let uploader = SnapshotUploader::new(ebs_client);
    let root_snapshot = snapshot_from_image(
        &ami_args.root_image,
        &uploader,
        None,
        ami_args.no_progress,
    )
    .await
    .context(error::Snapshot {
        path: &ami_args.root_image,
        region,
    })?;
    cleanup_snapshot_ids.push(root_snapshot.clone());

    let data_snapshot = snapshot_from_image(
        &ami_args.data_image,
        &uploader,
        None,
        ami_args.no_progress,
    )
    .await
    .context(error::Snapshot {
        path: &ami_args.root_image,
        region,
    })?;
    cleanup_snapshot_ids.push(data_snapshot.clone());

    info!(
        "Waiting for root and data snapshots to become available in {}",
        region
    );
    let waiter = SnapshotWaiter::new(ec2_client.clone());
    waiter
        .wait(&root_snapshot, Default::default())
        .await
        .context(error::WaitSnapshot {
            snapshot_type: "root",
        })?;
    waiter
        .wait(&data_snapshot, Default::default())
        .await
        .context(error::WaitSnapshot {
            snapshot_type: "data",
        })?;

    // Prepare parameters for AMI registration request
    let root_bdm = BlockDeviceMapping {
        device_name: Some(ROOT_DEVICE_NAME.to_string()),
        ebs: Some(EbsBlockDevice {
            delete_on_termination: Some(true),
            snapshot_id: Some(root_snapshot.clone()),
            volume_type: Some(VOLUME_TYPE.to_string()),
            volume_size: ami_args.root_volume_size,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut data_bdm = root_bdm.clone();
    data_bdm.device_name = Some(DATA_DEVICE_NAME.to_string());
    if let Some(ebs) = data_bdm.ebs.as_mut() {
        ebs.snapshot_id = Some(data_snapshot.clone());
        ebs.volume_size = Some(ami_args.data_volume_size);
    }

    let register_request = RegisterImageRequest {
        architecture: Some(ami_args.arch.clone()),
        block_device_mappings: Some(vec![root_bdm, data_bdm]),
        description: ami_args.description.clone(),
        ena_support: Some(ENA),
        name: ami_args.name.clone(),
        root_device_name: Some(ROOT_DEVICE_NAME.to_string()),
        sriov_net_support: Some(SRIOV.to_string()),
        virtualization_type: Some(VIRT_TYPE.to_string()),
        ..Default::default()
    };

    info!("Making register image call in {}", region);
    let register_response = ec2_client
        .register_image(register_request)
        .await
        .context(error::RegisterImage { region })?;

    let image_id = register_response
        .image_id
        .context(error::MissingImageId { region })?;

    Ok(RegisteredIds {
        image_id,
        snapshot_ids: vec![root_snapshot, data_snapshot],
    })
}

/// Uploads the given images into snapshots and registers an AMI using them as its block device
/// mapping.  Deletes snapshots on failure.
pub(crate) async fn register_image(
    ami_args: &AmiArgs,
    region: &str,
    ebs_client: EbsClient,
    ec2_client: &Ec2Client,
) -> Result<RegisteredIds> {
    info!("Registering '{}' in {}", ami_args.name, region);
    let mut cleanup_snapshot_ids = Vec::new();
    let register_result = _register_image(
        ami_args,
        region,
        ebs_client,
        ec2_client,
        &mut cleanup_snapshot_ids,
    )
    .await;

    if let Err(_) = register_result {
        for snapshot_id in cleanup_snapshot_ids {
            let delete_request = DeleteSnapshotRequest {
                snapshot_id: snapshot_id.clone(),
                ..Default::default()
            };
            if let Err(e) = ec2_client.delete_snapshot(delete_request).await {
                warn!(
                    "While cleaning up, failed to delete snapshot {}: {}",
                    snapshot_id, e
                );
            }
        }
    }
    register_result
}

/// Queries EC2 for the given AMI name. If found, returns Ok(Some(id)), if not returns Ok(None).
pub(crate) async fn get_ami_id<S1, S2>(
    name: S1,
    arch: S2,
    region: &str,
    ec2_client: &Ec2Client,
) -> Result<Option<String>>
where
    S1: Into<String>,
    S2: Into<String>,
{
    let describe_request = DescribeImagesRequest {
        owners: Some(vec!["self".to_string()]),
        filters: Some(vec![
            Filter {
                name: Some("name".to_string()),
                values: Some(vec![name.into()]),
            },
            Filter {
                name: Some("architecture".to_string()),
                values: Some(vec![arch.into()]),
            },
            Filter {
                name: Some("image-type".to_string()),
                values: Some(vec!["machine".to_string()]),
            },
            Filter {
                name: Some("virtualization-type".to_string()),
                values: Some(vec![VIRT_TYPE.to_string()]),
            },
        ]),
        ..Default::default()
    };
    let describe_response = ec2_client
        .describe_images(describe_request)
        .await
        .context(error::DescribeImages { region })?;
    if let Some(mut images) = describe_response.images {
        if images.is_empty() {
            return Ok(None);
        }
        ensure!(
            images.len() == 1,
            error::MultipleImages {
                images: images
                    .into_iter()
                    .map(|i| i.image_id.unwrap_or_else(|| "<missing>".to_string()))
                    .collect::<Vec<_>>()
            }
        );
        let image = images.remove(0);
        // If there is an image but we couldn't find the ID of it, fail rather than returning None,
        // which would indicate no image.
        let id = image.image_id.context(error::MissingImageId { region })?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

mod error {
    use crate::aws::ami;
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Failed to describe images in {}: {}", region, source))]
        DescribeImages {
            region: String,
            source: rusoto_core::RusotoError<rusoto_ec2::DescribeImagesError>,
        },

        #[snafu(display("Image response in {} did not include image ID", region))]
        MissingImageId { region: String },

        #[snafu(display("DescribeImages with unique filters returned multiple results: {}", images.join(", ")))]
        MultipleImages { images: Vec<String> },

        #[snafu(display("Failed to register image in {}: {}", region, source))]
        RegisterImage {
            region: String,
            source: rusoto_core::RusotoError<rusoto_ec2::RegisterImageError>,
        },

        #[snafu(display("Failed to upload snapshot from {} in {}: {}", path.display(),region, source))]
        Snapshot {
            path: PathBuf,
            region: String,
            source: ami::snapshot::Error,
        },

        #[snafu(display("{} snapshot did not become available: {}", snapshot_type, source))]
        WaitSnapshot {
            snapshot_type: String,
            source: coldsnap::WaitError,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
