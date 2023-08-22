//! The ami module owns the describing of images in EC2.

use aws_sdk_ec2::{config::Region, types::Image, Client as Ec2Client};
use futures::future::{join, ready};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;

use crate::aws::ami::launch_permissions::{get_launch_permissions, LaunchPermissionDef};

/// Wrapper structure for the `ImageDef` struct, used during deserialization
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ImageData {
    Image(ImageDef),
    ImageList(Vec<ImageDef>),
}

impl ImageData {
    pub(crate) fn images(&self) -> Vec<ImageDef> {
        match self {
            ImageData::Image(image) => vec![image.to_owned()],
            ImageData::ImageList(images) => images.to_owned(),
        }
    }
}

/// Structure of the EC2 image fields that should be validated
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct ImageDef {
    /// The ID of the EC2 image
    pub(crate) id: String,

    /// The name of the EC2 image
    pub(crate) name: String,

    /// Whether or not the EC2 image is public
    #[serde(default)]
    pub(crate) public: bool,

    /// The launch permissions for the EC2 image.
    pub(crate) launch_permissions: Option<Vec<LaunchPermissionDef>>,

    /// Whether or not the EC2 image supports Elastic Network Adapter
    #[serde(default = "default_ena_support")]
    pub(crate) ena_support: bool,

    /// The level of the EC2 image's Single Root I/O Virtualization support
    #[serde(default = "default_sriov_net_support")]
    pub(crate) sriov_net_support: String,
}

fn default_ena_support() -> bool {
    true
}

fn default_sriov_net_support() -> String {
    "simple".to_string()
}

impl From<(Image, Option<Vec<LaunchPermissionDef>>)> for ImageDef {
    fn from(args: (Image, Option<Vec<LaunchPermissionDef>>)) -> Self {
        Self {
            id: args.0.image_id().unwrap_or_default().to_string(),
            name: args.0.name().unwrap_or_default().to_string(),
            public: args.0.public().unwrap_or_default(),
            launch_permissions: args.1,
            ena_support: args.0.ena_support().unwrap_or_default(),
            sriov_net_support: args.0.sriov_net_support().unwrap_or_default().to_string(),
        }
    }
}

/// Fetches all images whose IDs are keys in `expected_images`. The map `expected_image_public` is
/// used to determine if the launch permissions for the image should be fetched (only if the image is not
/// public). The return value is a HashMap of Region to a Result, which is `Ok` if the request for
/// that region was successful and `Err` if not. The Result contains a HashMap of `image_id` to
/// `ImageDef`.
pub(crate) async fn describe_images<'a>(
    clients: &'a HashMap<Region, Ec2Client>,
    expected_images: &HashMap<Region, Vec<ImageDef>>,
) -> HashMap<&'a Region, Result<HashMap<String, ImageDef>>> {
    // Build requests for images; we have to request with a regional client so we split them by
    // region
    let mut requests = Vec::with_capacity(clients.len());
    clients.iter().for_each(|(region, ec2_client)| {
        trace!("Requesting images in {}", region);
        let get_future = describe_images_in_region(
            region,
            ec2_client,
            expected_images
                .get(region)
                .map(|i| i.to_owned())
                .unwrap_or_default()
                .into_iter()
                .map(|i| (i.id.clone(), i))
                .collect::<HashMap<String, ImageDef>>(),
        );

        requests.push(join(ready(region), get_future));
    });

    // Send requests in parallel and wait for responses, collecting results into a list.
    requests
        .into_iter()
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await
}

/// Fetches the images whose IDs are keys in `expected_images`
pub(crate) async fn describe_images_in_region(
    region: &Region,
    client: &Ec2Client,
    expected_images: HashMap<String, ImageDef>,
) -> Result<HashMap<String, ImageDef>> {
    info!("Retrieving images in {}", region.to_string());
    let mut images = HashMap::new();

    // Send the request
    let mut get_future = client
        .describe_images()
        .include_deprecated(true)
        .set_image_ids(Some(Vec::from_iter(
            expected_images.keys().map(|k| k.to_owned()),
        )))
        .into_paginator()
        .send();

    // Iterate over the retrieved images
    while let Some(page) = get_future.next().await {
        let retrieved_images = page
            .context(error::DescribeImagesSnafu {
                region: region.to_string(),
            })?
            .images()
            .unwrap_or_default()
            .to_owned();
        for image in retrieved_images {
            // Insert a new key-value pair into the map, with the key containing image ID
            // and the value containing the ImageDef object created from the image
            let image_id = image
                .image_id()
                .ok_or(error::Error::MissingField {
                    missing: "image_id".to_string(),
                })?
                .to_string();
            let expected_public = expected_images
                .get(&image_id)
                .ok_or(error::Error::MissingExpectedPublic {
                    missing: image_id.clone(),
                })?
                .public;
            // If the image is not expected to be public, retrieve the launch permissions
            trace!(
                "Retrieving launch permissions for {} in {}",
                image_id,
                region.as_ref()
            );
            let launch_permissions = if !expected_public {
                Some(
                    get_launch_permissions(client, region.as_ref(), &image_id)
                        .await
                        .context(error::GetLaunchPermissionsSnafu {
                            region: region.as_ref(),
                            image_id: image_id.clone(),
                        })?,
                )
            } else {
                None
            };
            let image_def = ImageDef::from((image.to_owned(), launch_permissions));
            images.insert(image_id, image_def);
        }
    }

    info!("Images in {} have been retrieved", region.to_string());
    Ok(images)
}

pub(crate) mod error {
    use aws_sdk_ec2::operation::describe_images::DescribeImagesError;
    use aws_sdk_ssm::error::SdkError;
    use aws_smithy_types::error::display::DisplayErrorContext;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    #[allow(clippy::large_enum_variant)]
    pub(crate) enum Error {
        #[snafu(display(
            "Failed to describe images in {}: {}",
            region,
            DisplayErrorContext(source)
        ))]
        DescribeImages {
            region: String,
            source: SdkError<DescribeImagesError>,
        },

        #[snafu(display(
            "Failed to retrieve launch permissions for image {} in region {}: {}",
            image_id,
            region,
            source
        ))]
        GetLaunchPermissions {
            region: String,
            image_id: String,
            source: crate::aws::ami::launch_permissions::Error,
        },

        #[snafu(display("Missing field in image: {}", missing))]
        MissingField { missing: String },

        #[snafu(display("Missing image ID in expected image publicity map: {}", missing))]
        MissingExpectedPublic { missing: String },
    }
}

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
