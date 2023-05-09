use super::{snapshot::snapshot_from_image, AmiArgs};
use aws_sdk_ebs::Client as EbsClient;
use aws_sdk_ec2::model::{
    ArchitectureValues, BlockDeviceMapping, EbsBlockDevice, Filter, VolumeType,
};
use aws_sdk_ec2::{Client as Ec2Client, Region};
use buildsys::manifest::{self, ImageFeature};
use coldsnap::{SnapshotUploader, SnapshotWaiter};
use log::{debug, info, warn};
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
    region: &Region,
    ebs_client: EbsClient,
    ec2_client: &Ec2Client,
    cleanup_snapshot_ids: &mut Vec<String>,
) -> Result<RegisteredIds> {
    let variant_manifest = manifest::ManifestInfo::new(&ami_args.variant_manifest).context(
        error::LoadVariantManifestSnafu {
            path: &ami_args.variant_manifest,
        },
    )?;

    let image_layout = variant_manifest
        .image_layout()
        .context(error::MissingImageLayoutSnafu {
            path: &ami_args.variant_manifest,
        })?;

    let (os_volume_size, data_volume_size) = image_layout.publish_image_sizes_gib();

    let uefi_data =
        std::fs::read_to_string(&ami_args.uefi_data).context(error::LoadUefiDataSnafu {
            path: &ami_args.uefi_data,
        })?;

    debug!("Uploading images into EBS snapshots in {}", region);
    let uploader = SnapshotUploader::new(ebs_client);
    let os_snapshot =
        snapshot_from_image(&ami_args.os_image, &uploader, None, ami_args.no_progress)
            .await
            .context(error::SnapshotSnafu {
                path: &ami_args.os_image,
                region: region.as_ref(),
            })?;
    cleanup_snapshot_ids.push(os_snapshot.clone());

    let mut data_snapshot = None;
    if let Some(data_image) = &ami_args.data_image {
        let snapshot = snapshot_from_image(data_image, &uploader, None, ami_args.no_progress)
            .await
            .context(error::SnapshotSnafu {
                path: &ami_args.os_image,
                region: region.as_ref(),
            })?;
        cleanup_snapshot_ids.push(snapshot.clone());
        data_snapshot = Some(snapshot);
    }

    info!("Waiting for snapshots to become available in {}", region);
    let waiter = SnapshotWaiter::new(ec2_client.clone());
    waiter
        .wait(&os_snapshot, Default::default())
        .await
        .context(error::WaitSnapshotSnafu {
            snapshot_type: "root",
        })?;

    if let Some(ref data_snapshot) = data_snapshot {
        waiter
            .wait(&data_snapshot, Default::default())
            .await
            .context(error::WaitSnapshotSnafu {
                snapshot_type: "data",
            })?;
    }

    // Prepare parameters for AMI registration request
    let os_bdm = BlockDeviceMapping::builder()
        .set_device_name(Some(ROOT_DEVICE_NAME.to_string()))
        .set_ebs(Some(
            EbsBlockDevice::builder()
                .set_delete_on_termination(Some(true))
                .set_snapshot_id(Some(os_snapshot.clone()))
                .set_volume_type(Some(VolumeType::from(VOLUME_TYPE)))
                .set_volume_size(Some(os_volume_size))
                .build(),
        ))
        .build();

    let mut data_bdm = None;
    if let Some(ref data_snapshot) = data_snapshot {
        let mut bdm = os_bdm.clone();
        bdm.device_name = Some(DATA_DEVICE_NAME.to_string());
        if let Some(ebs) = bdm.ebs.as_mut() {
            ebs.snapshot_id = Some(data_snapshot.clone());
            ebs.volume_size = Some(data_volume_size);
        }
        data_bdm = Some(bdm);
    }

    let mut block_device_mappings = vec![os_bdm];
    if let Some(data_bdm) = data_bdm {
        block_device_mappings.push(data_bdm);
    }

    let uefi_secure_boot_enabled = variant_manifest
        .image_features()
        .iter()
        .flatten()
        .any(|f| **f == ImageFeature::UefiSecureBoot);

    let (boot_mode, uefi_data) = if uefi_secure_boot_enabled {
        (Some("uefi-preferred".into()), Some(uefi_data))
    } else {
        (None, None)
    };

    info!("Making register image call in {}", region);
    let register_response = ec2_client
        .register_image()
        .set_architecture(Some(ami_args.arch.clone()))
        .set_block_device_mappings(Some(block_device_mappings))
        .set_boot_mode(boot_mode)
        .set_uefi_data(uefi_data)
        .set_description(ami_args.description.clone())
        .set_ena_support(Some(ENA))
        .set_name(Some(ami_args.name.clone()))
        .set_root_device_name(Some(ROOT_DEVICE_NAME.to_string()))
        .set_sriov_net_support(Some(SRIOV.to_string()))
        .set_virtualization_type(Some(VIRT_TYPE.to_string()))
        .send()
        .await
        .context(error::RegisterImageSnafu {
            region: region.as_ref(),
        })?;

    let image_id = register_response
        .image_id
        .context(error::MissingImageIdSnafu {
            region: region.as_ref(),
        })?;

    let mut snapshot_ids = vec![os_snapshot];
    if let Some(data_snapshot) = data_snapshot {
        snapshot_ids.push(data_snapshot);
    }

    Ok(RegisteredIds {
        image_id,
        snapshot_ids,
    })
}

/// Uploads the given images into snapshots and registers an AMI using them as its block device
/// mapping.  Deletes snapshots on failure.
pub(crate) async fn register_image(
    ami_args: &AmiArgs,
    region: &Region,
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

    if register_result.is_err() {
        for snapshot_id in cleanup_snapshot_ids {
            if let Err(e) = ec2_client
                .delete_snapshot()
                .set_snapshot_id(Some(snapshot_id.clone()))
                .send()
                .await
            {
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
pub(crate) async fn get_ami_id<S>(
    name: S,
    arch: &ArchitectureValues,
    region: &Region,
    ec2_client: &Ec2Client,
) -> Result<Option<String>>
where
    S: Into<String>,
{
    let describe_response = ec2_client
        .describe_images()
        .set_owners(Some(vec!["self".to_string()]))
        .set_filters(Some(vec![
            Filter::builder()
                .set_name(Some("name".to_string()))
                .set_values(Some(vec![name.into()]))
                .build(),
            Filter::builder()
                .set_name(Some("architecture".to_string()))
                .set_values(Some(vec![arch.as_ref().to_string()]))
                .build(),
            Filter::builder()
                .set_name(Some("image-type".to_string()))
                .set_values(Some(vec!["machine".to_string()]))
                .build(),
            Filter::builder()
                .set_name(Some("virtualization-type".to_string()))
                .set_values(Some(vec![VIRT_TYPE.to_string()]))
                .build(),
        ]))
        .send()
        .await
        .context(error::DescribeImagesSnafu {
            region: region.as_ref(),
        })?;
    if let Some(mut images) = describe_response.images {
        if images.is_empty() {
            return Ok(None);
        }
        ensure!(
            images.len() == 1,
            error::MultipleImagesSnafu {
                images: images
                    .into_iter()
                    .map(|i| i.image_id.unwrap_or_else(|| "<missing>".to_string()))
                    .collect::<Vec<_>>()
            }
        );
        let image = images.remove(0);
        // If there is an image but we couldn't find the ID of it, fail rather than returning None,
        // which would indicate no image.
        let id = image.image_id.context(error::MissingImageIdSnafu {
            region: region.as_ref(),
        })?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

mod error {
    use crate::aws::ami;
    use aws_sdk_ec2::error::{DescribeImagesError, RegisterImageError};
    use aws_sdk_ec2::types::SdkError;
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to describe images in {}: {}", region, source))]
        DescribeImages {
            region: String,
            source: SdkError<DescribeImagesError>,
        },

        #[snafu(display("Failed to load variant manifest from {}: {}", path.display(), source))]
        LoadVariantManifest {
            path: PathBuf,
            source: buildsys::manifest::Error,
        },

        #[snafu(display("Failed to load UEFI data from {}: {}", path.display(), source))]
        LoadUefiData {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Could not find image layout for {}", path.display()))]
        MissingImageLayout { path: PathBuf },

        #[snafu(display("Image response in {} did not include image ID", region))]
        MissingImageId { region: String },

        #[snafu(display("DescribeImages with unique filters returned multiple results: {}", images.join(", ")))]
        MultipleImages { images: Vec<String> },

        #[snafu(display("Failed to register image in {}: {}", region, source))]
        RegisterImage {
            region: String,
            source: SdkError<RegisterImageError>,
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
