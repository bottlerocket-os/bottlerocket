//! The ami module owns the describing of images in EC2.

use aws_sdk_ec2::model::Image;
use aws_sdk_ec2::{Client as Ec2Client, Region};
use futures::future::{join, ready};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;

/// Structure of the EC2 image fields that should be validated
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ImageDef {
    /// The id of the EC2 image
    pub(crate) image_id: String,

    /// Whether or not the EC2 image is public
    pub(crate) public: bool,

    /// Whether or not the EC2 image supports Elastic Network Adapter
    pub(crate) ena_support: bool,

    /// The level of the EC2 image's Single Root I/O Virtualization support
    pub(crate) sriov_net_support: String,
}

impl From<Image> for ImageDef {
    fn from(image: Image) -> Self {
        Self {
            image_id: image.image_id().unwrap_or_default().to_string(),
            public: image.public().unwrap_or_default(),
            ena_support: image.ena_support().unwrap_or_default(),
            sriov_net_support: image.sriov_net_support().unwrap_or_default().to_string(),
        }
    }
}

impl ImageDef {
    // Creates a new ImageDef with a given image_id and expected values for public, ena_support,
    // and sriov_net_support
    pub(crate) fn expected(image_id: String) -> Self {
        Self {
            image_id,
            public: true,
            ena_support: true,
            sriov_net_support: "simple".to_string(),
        }
    }
}

pub(crate) async fn describe_images<'a>(
    clients: &'a HashMap<Region, Ec2Client>,
    image_ids: &HashMap<Region, Vec<String>>,
) -> HashMap<&'a Region, Result<HashMap<String, ImageDef>>> {
    // Build requests for images; we have to request with a regional client so we split them by
    // region
    let mut requests = Vec::with_capacity(clients.len());
    for region in clients.keys() {
        trace!("Requesting images in {}", region);
        let ec2_client: &Ec2Client = &clients[region];
        let get_future = describe_images_in_region(
            region,
            ec2_client,
            image_ids
                .get(region)
                .map(|i| i.to_owned())
                .unwrap_or(vec![]),
        );

        requests.push(join(ready(region), get_future));
    }

    // Send requests in parallel and wait for responses, collecting results into a list.
    requests
        .into_iter()
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await
}

/// Fetches all images in a single region
pub(crate) async fn describe_images_in_region(
    region: &Region,
    client: &Ec2Client,
    image_ids: Vec<String>,
) -> Result<HashMap<String, ImageDef>> {
    info!("Retrieving images in {}", region.to_string());
    let mut images = HashMap::new();

    // Send the request
    let mut get_future = client
        .describe_images()
        .include_deprecated(true)
        .set_image_ids(Some(image_ids))
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
            // Insert a new key-value pair into the map, with the key containing image id
            // and the value containing the ImageDef object created from the image
            images.insert(
                image
                    .image_id()
                    .ok_or(error::Error::MissingField {
                        missing: "image_id".to_string(),
                    })?
                    .to_string(),
                ImageDef::from(image.to_owned()),
            );
        }
    }

    info!("Images in {} have been retrieved", region.to_string());
    Ok(images)
}

pub(crate) mod error {
    use aws_sdk_ec2::error::DescribeImagesError;
    use aws_sdk_ssm::types::SdkError;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    #[allow(clippy::large_enum_variant)]
    pub enum Error {
        #[snafu(display("Failed to describe images in {}: {}", region, source))]
        DescribeImages {
            region: String,
            source: SdkError<DescribeImagesError>,
        },

        #[snafu(display("Missing field in image: {}", missing))]
        MissingField { missing: String },
    }
}

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
