//! The validate_ami module owns the 'validate-ami' subcommand and controls the process of validating
//! EC2 images

pub(crate) mod ami;
pub(crate) mod results;

use self::ami::{ImageData, ImageDef};
use self::results::{AmiValidationResult, AmiValidationResultStatus, AmiValidationResults};
use crate::aws::client::build_client_config;
use crate::aws::validate_ami::ami::describe_images;
use crate::Args;
use aws_sdk_ec2::{Client as AmiClient, Region};
use clap::Parser;
use log::{error, info, trace};
use pubsys_config::InfraConfig;
use snafu::ResultExt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

/// Validates EC2 images by calling `describe-images` on all images in the file given by
/// `expected-amis-path` and ensuring that the returned `public`, `ena-support`,
/// `sriov-net-support`, and `launch-permissions` fields have the expected values.
#[derive(Debug, Parser)]
pub(crate) struct ValidateAmiArgs {
    /// File holding the expected amis
    #[arg(long)]
    expected_amis_path: PathBuf,

    /// Optional path where the validation results should be written
    #[arg(long)]
    write_results_path: Option<PathBuf>,

    #[arg(long, requires = "write_results_path")]
    /// Optional filter to only write validation results with these statuses to the above path
    /// The available statuses are: `Correct`, `Incorrect`, `Missing`.
    write_results_filter: Option<Vec<AmiValidationResultStatus>>,

    #[arg(long)]
    /// If this argument is given, print the validation results summary as a JSON object instead
    /// of a plaintext table
    json: bool,
}

/// Performs EC2 image validation and returns the `AmiValidationResults` object
pub(crate) async fn validate(
    args: &Args,
    validate_ami_args: &ValidateAmiArgs,
) -> Result<AmiValidationResults> {
    info!("Parsing Infra.toml file");

    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, false)
        .context(error::ConfigSnafu)?;

    trace!("Parsed infra config: {:#?}", infra_config);

    let aws = infra_config.aws.unwrap_or_default();

    // Parse the expected ami file
    info!("Parsing expected ami file");
    let expected_images = parse_expected_amis(&validate_ami_args.expected_amis_path).await?;

    info!("Parsed expected ami file");

    // Create a `HashMap` of `AmiClient`s, one for each region where validation should happen
    let base_region = &Region::new(
        aws.regions
            .get(0)
            .ok_or(error::Error::EmptyInfraRegions {
                path: args.infra_config_path.clone(),
            })?
            .clone(),
    );
    let mut ami_clients = HashMap::with_capacity(expected_images.len());

    for region in expected_images.keys() {
        let client_config = build_client_config(region, base_region, &aws).await;
        let ami_client = AmiClient::new(&client_config);
        ami_clients.insert(region.clone(), ami_client);
    }

    // Retrieve the EC2 images using the `AmiClient`s
    info!("Retrieving EC2 images");
    let images = describe_images(&ami_clients, &expected_images)
        .await
        .into_iter()
        .map(|(region, result)| {
            (
                region,
                result.map_err(|e| {
                    error!(
                        "Failed to retrieve images in region {}: {}",
                        region.to_string(),
                        e
                    );
                    error::Error::UnreachableRegion {
                        region: region.to_string(),
                    }
                }),
            )
        })
        .collect::<HashMap<&Region, Result<_>>>();

    // Validate the retrieved EC2 images per region
    info!("Validating EC2 images");
    let results: HashMap<Region, HashSet<AmiValidationResult>> = images
        .into_iter()
        .map(|(region, region_result)| {
            (
                region.clone(),
                validate_images_in_region(
                    &expected_images
                        .get(region)
                        .map(|e| e.to_owned())
                        .unwrap_or_default(),
                    &region_result,
                    region,
                ),
            )
        })
        .collect();

    let validation_results = AmiValidationResults::from_result_map(results);

    // If a path was given, write the results
    if let Some(write_results_path) = &validate_ami_args.write_results_path {
        // Filter the results by given status, and if no statuses were given, get all results
        info!("Writing results to file");
        let results = if let Some(filter) = &validate_ami_args.write_results_filter {
            validation_results.get_results_for_status(filter)
        } else {
            validation_results.get_all_results()
        };

        // Write the results as JSON
        serde_json::to_writer_pretty(
            &File::create(write_results_path).context(error::WriteValidationResultsSnafu {
                path: write_results_path,
            })?,
            &results,
        )
        .context(error::SerializeValidationResultsSnafu)?;
    }

    Ok(validation_results)
}

/// Validates EC2 images in a single region, based on a `Vec<ImageDef>` of expected images
/// and a `HashMap<AmiId, ImageDef>` of actual retrieved images. Returns a
/// `HashSet<AmiValidationResult>` containing the result objects.
pub(crate) fn validate_images_in_region(
    expected_images: &[ImageDef],
    actual_images: &Result<HashMap<AmiId, ImageDef>>,
    region: &Region,
) -> HashSet<AmiValidationResult> {
    match actual_images {
        Ok(actual_images) => expected_images
            .iter()
            .map(|image| {
                let new_image = if image.public {
                    ImageDef {
                        launch_permissions: None,
                        ..image.clone()
                    }
                } else {
                    image.clone()
                };
                AmiValidationResult::new(
                    image.id.clone(),
                    new_image,
                    Ok(actual_images.get(&image.id).map(|v| v.to_owned())),
                    region.clone(),
                )
            })
            .collect(),
        Err(_) => expected_images
            .iter()
            .map(|image| {
                AmiValidationResult::new(
                    image.id.clone(),
                    image.clone(),
                    Err(error::Error::UnreachableRegion {
                        region: region.to_string(),
                    }),
                    region.clone(),
                )
            })
            .collect(),
    }
}

type RegionName = String;
type AmiId = String;

/// Parse the file holding image values. Return a `HashMap` of `Region` mapped to a vec of `ImageDef`s
/// for that region.
pub(crate) async fn parse_expected_amis(
    expected_amis_path: &PathBuf,
) -> Result<HashMap<Region, Vec<ImageDef>>> {
    // Parse the JSON file as a `HashMap` of region_name, mapped to an `ImageData` struct
    let expected_amis: HashMap<RegionName, ImageData> = serde_json::from_reader(
        &File::open(expected_amis_path.clone()).context(error::ReadExpectedImagesFileSnafu {
            path: expected_amis_path,
        })?,
    )
    .context(error::ParseExpectedImagesFileSnafu)?;

    // Extract the `Vec<ImageDef>` from the `ImageData` structs
    let vectored_images = expected_amis
        .into_iter()
        .map(|(region, value)| (Region::new(region), value.images()))
        .collect::<HashMap<Region, Vec<ImageDef>>>();

    Ok(vectored_images)
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, validate_ami_args: &ValidateAmiArgs) -> Result<()> {
    let results = validate(args, validate_ami_args).await?;

    if validate_ami_args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&results.get_json_summary())
                .context(error::SerializeResultsSummarySnafu)?
        )
    } else {
        println!("{}", results);
    }
    Ok(())
}

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Error reading config: {}", source))]
        Config { source: pubsys_config::Error },

        #[snafu(display("Empty regions array in Infra.toml at path {}", path.display()))]
        EmptyInfraRegions { path: PathBuf },

        #[snafu(display("Failed to parse image file: {}", source))]
        ParseExpectedImagesFile { source: serde_json::Error },

        #[snafu(display("Failed to read image file: {:?}", path))]
        ReadExpectedImagesFile {
            source: std::io::Error,
            path: PathBuf,
        },

        #[snafu(display("Failed to serialize validation results to json: {}", source))]
        SerializeValidationResults { source: serde_json::Error },

        #[snafu(display("Failed to retrieve images from region {}", region))]
        UnreachableRegion { region: String },

        #[snafu(display("Failed to write validation results to {:?}: {}", path, source))]
        WriteValidationResults {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to serialize results summary to JSON: {}", source))]
        SerializeResultsSummary { source: serde_json::Error },
    }
}

pub(crate) use error::Error;

type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use super::ami::ImageDef;
    use super::validate_images_in_region;
    use crate::aws::{
        ami::launch_permissions::LaunchPermissionDef,
        validate_ami::results::{AmiValidationResult, AmiValidationResultStatus},
    };
    use aws_sdk_ec2::Region;
    use std::collections::{HashMap, HashSet};

    // These tests assert that the images can be validated correctly.

    // Tests validation of images where the expected value is equal to the actual value
    #[test]
    fn validate_images_all_correct() {
        let expected_parameters: Vec<ImageDef> = vec![
            ImageDef {
                id: "test1-image-id".to_string(),
                name: "test1-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test2-image-id".to_string(),
                name: "test2-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test3-image-id".to_string(),
                name: "test3-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
        ];
        let actual_parameters: HashMap<String, ImageDef> = HashMap::from([
            (
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
            ),
            (
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
            ),
            (
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            AmiValidationResult::new(
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
        ]);
        let results = validate_images_in_region(
            &expected_parameters,
            &Ok(actual_parameters),
            &Region::new("us-west-2"),
        );

        for result in &results {
            assert_eq!(result.status, AmiValidationResultStatus::Correct);
        }
        assert_eq!(results, expected_results);
    }

    // Tests validation of images where the expected value is different from the actual value
    #[test]
    fn validate_images_all_incorrect() {
        let expected_parameters: Vec<ImageDef> = vec![
            ImageDef {
                id: "test1-image-id".to_string(),
                name: "test1-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test2-image-id".to_string(),
                name: "test2-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test3-image-id".to_string(),
                name: "test3-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
        ];
        let actual_parameters: HashMap<String, ImageDef> = HashMap::from([
            (
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: false,
                    sriov_net_support: "simple".to_string(),
                },
            ),
            (
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: false,
                    launch_permissions: Some(vec![LaunchPermissionDef::Group("all".to_string())]),
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
            ),
            (
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "not simple".to_string(),
                },
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            AmiValidationResult::new(
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "not simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: false,
                    launch_permissions: Some(vec![LaunchPermissionDef::Group("all".to_string())]),
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: false,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
        ]);
        let results = validate_images_in_region(
            &expected_parameters,
            &Ok(actual_parameters),
            &Region::new("us-west-2"),
        );
        for result in &results {
            assert_eq!(result.status, AmiValidationResultStatus::Incorrect);
        }
        assert_eq!(results, expected_results);
    }

    // Tests validation of images where the actual value is missing
    #[test]
    fn validate_images_all_missing() {
        let expected_parameters: Vec<ImageDef> = vec![
            ImageDef {
                id: "test1-image-id".to_string(),
                name: "test1-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test2-image-id".to_string(),
                name: "test2-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test3-image-id".to_string(),
                name: "test3-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
        ];
        let actual_parameters = HashMap::new();
        let expected_results = HashSet::from_iter(vec![
            AmiValidationResult::new(
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(None),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(None),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(None),
                Region::new("us-west-2"),
            ),
        ]);
        let results = validate_images_in_region(
            &expected_parameters,
            &Ok(actual_parameters),
            &Region::new("us-west-2"),
        );
        for result in &results {
            assert_eq!(result.status, AmiValidationResultStatus::Missing);
        }
        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where each reachable status (Correct, Incorrect, Missing) happens once
    #[test]
    fn validate_images_mixed() {
        let expected_parameters: Vec<ImageDef> = vec![
            ImageDef {
                id: "test1-image-id".to_string(),
                name: "test1-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test2-image-id".to_string(),
                name: "test2-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test3-image-id".to_string(),
                name: "test3-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
        ];
        let actual_parameters: HashMap<String, ImageDef> = HashMap::from([
            (
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
            ),
            (
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: false,
                    launch_permissions: Some(vec![LaunchPermissionDef::Group("all".to_string())]),
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            AmiValidationResult::new(
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(Some(ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: false,
                    launch_permissions: Some(vec![LaunchPermissionDef::Group("all".to_string())]),
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                })),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Ok(None),
                Region::new("us-west-2"),
            ),
        ]);
        let results = validate_images_in_region(
            &expected_parameters,
            &Ok(actual_parameters),
            &Region::new("us-west-2"),
        );

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the region is unreachable
    #[test]
    fn validate_images_unreachable() {
        let expected_parameters: Vec<ImageDef> = vec![
            ImageDef {
                id: "test1-image-id".to_string(),
                name: "test1-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test2-image-id".to_string(),
                name: "test2-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
            ImageDef {
                id: "test3-image-id".to_string(),
                name: "test3-image".to_string(),
                public: true,
                launch_permissions: None,
                ena_support: true,
                sriov_net_support: "simple".to_string(),
            },
        ];
        let expected_results = HashSet::from_iter(vec![
            AmiValidationResult::new(
                "test1-image-id".to_string(),
                ImageDef {
                    id: "test1-image-id".to_string(),
                    name: "test1-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Err(crate::aws::validate_ami::Error::UnreachableRegion {
                    region: "us-west-2".to_string(),
                }),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test2-image-id".to_string(),
                ImageDef {
                    id: "test2-image-id".to_string(),
                    name: "test2-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Err(crate::aws::validate_ami::Error::UnreachableRegion {
                    region: "us-west-2".to_string(),
                }),
                Region::new("us-west-2"),
            ),
            AmiValidationResult::new(
                "test3-image-id".to_string(),
                ImageDef {
                    id: "test3-image-id".to_string(),
                    name: "test3-image".to_string(),
                    public: true,
                    launch_permissions: None,
                    ena_support: true,
                    sriov_net_support: "simple".to_string(),
                },
                Err(crate::aws::validate_ami::Error::UnreachableRegion {
                    region: "us-west-2".to_string(),
                }),
                Region::new("us-west-2"),
            ),
        ]);
        let results = validate_images_in_region(
            &expected_parameters,
            &Err(crate::aws::validate_ami::Error::UnreachableRegion {
                region: "us-west-2".to_string(),
            }),
            &Region::new("us-west-2"),
        );

        assert_eq!(results, expected_results);
    }
}
