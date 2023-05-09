//! The ami module owns the 'ami' subcommand and controls the process of registering and copying
//! EC2 AMIs.

pub(crate) mod launch_permissions;
pub(crate) mod public;
mod register;
mod snapshot;
pub(crate) mod wait;

use crate::aws::ami::launch_permissions::get_launch_permissions;
use crate::aws::ami::public::ami_is_public;
use crate::aws::publish_ami::{get_snapshots, modify_image, modify_snapshots, ModifyOptions};
use crate::aws::{client::build_client_config, parse_arch, region_from_string};
use crate::Args;
use aws_sdk_ebs::Client as EbsClient;
use aws_sdk_ec2::error::CopyImageError;
use aws_sdk_ec2::model::{ArchitectureValues, OperationType};
use aws_sdk_ec2::output::CopyImageOutput;
use aws_sdk_ec2::types::SdkError;
use aws_sdk_ec2::{Client as Ec2Client, Region};
use aws_sdk_sts::error::GetCallerIdentityError;
use aws_sdk_sts::output::GetCallerIdentityOutput;
use aws_sdk_sts::Client as StsClient;
use clap::Parser;
use futures::future::{join, lazy, ready, FutureExt};
use futures::stream::{self, StreamExt};
use log::{error, info, trace, warn};
use pubsys_config::{AwsConfig as PubsysAwsConfig, InfraConfig};
use register::{get_ami_id, register_image, RegisteredIds};
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use wait::wait_for_ami;

const WARN_SEPARATOR: &str = "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!";

/// Builds Bottlerocket AMIs using latest build artifacts
#[derive(Debug, Parser)]
pub(crate) struct AmiArgs {
    /// Path to the image containing the os volume
    #[arg(short = 'o', long)]
    os_image: PathBuf,

    /// Path to the image containing the data volume
    #[arg(short = 'd', long)]
    data_image: Option<PathBuf>,

    /// Path to the variant manifest
    #[arg(short = 'v', long)]
    variant_manifest: PathBuf,

    /// Path to the UEFI data
    #[arg(short = 'e', long)]
    uefi_data: PathBuf,

    /// The architecture of the machine image
    #[arg(short = 'a', long, value_parser = parse_arch)]
    arch: ArchitectureValues,

    /// The desired AMI name
    #[arg(short = 'n', long)]
    name: String,

    /// The desired AMI description
    #[arg(long)]
    description: Option<String>,

    /// Don't display progress bars
    #[arg(long)]
    no_progress: bool,

    /// Regions where you want the AMI, the first will be used as the base for copying
    #[arg(long, value_delimiter = ',')]
    regions: Vec<String>,

    /// If specified, save created regional AMI IDs in JSON at this path.
    #[arg(long)]
    ami_output: Option<PathBuf>,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, ami_args: &AmiArgs) -> Result<()> {
    match _run(args, ami_args).await {
        Ok(amis) => {
            // Write the AMI IDs to file if requested
            if let Some(ref path) = ami_args.ami_output {
                write_amis(path, &amis).context(error::WriteAmisSnafu { path })?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

async fn _run(args: &Args, ami_args: &AmiArgs) -> Result<HashMap<String, Image>> {
    let mut amis = HashMap::new();

    // If a lock file exists, use that, otherwise use Infra.toml or default
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, true)
        .context(error::ConfigSnafu)?;
    trace!("Using infra config: {:?}", infra_config);

    let aws = infra_config.aws.unwrap_or_default();

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let mut regions = if !ami_args.regions.is_empty() {
        ami_args.regions.clone()
    } else {
        aws.regions.clone().into()
    }
    .into_iter()
    .map(|name| region_from_string(&name))
    .collect::<Vec<Region>>();

    ensure!(
        !regions.is_empty(),
        error::MissingConfigSnafu {
            missing: "aws.regions"
        }
    );

    // We register in this base region first, then copy from there to any other regions.
    let base_region = regions.remove(0);

    // Build EBS client for snapshot management, and EC2 client for registration
    let client_config = build_client_config(&base_region, &base_region, &aws).await;

    let base_ebs_client = EbsClient::new(&client_config);

    let base_ec2_client = Ec2Client::new(&client_config);

    // Check if the AMI already exists, in which case we can use the existing ID, otherwise we
    // register a new one.
    let maybe_id = get_ami_id(
        &ami_args.name,
        &ami_args.arch,
        &base_region,
        &base_ec2_client,
    )
    .await
    .context(error::GetAmiIdSnafu {
        name: &ami_args.name,
        arch: ami_args.arch.as_ref(),
        region: base_region.as_ref(),
    })?;

    // If the AMI does not exist yet, `public` should be false and `launch_permissions` empty
    let mut public = false;
    let mut launch_permissions = vec![];

    let (ids_of_image, already_registered) = if let Some(found_id) = maybe_id {
        warn!(
            "\n{}\n\nFound '{}' already registered in {}: {}\n\n{0}",
            WARN_SEPARATOR, ami_args.name, base_region, found_id
        );
        let snapshot_ids = get_snapshots(&found_id, &base_region, &base_ec2_client)
            .await
            .context(error::GetSnapshotsSnafu {
                image_id: &found_id,
                region: base_region.as_ref(),
            })?;
        let found_ids = RegisteredIds {
            image_id: found_id.clone(),
            snapshot_ids,
        };

        public = ami_is_public(&base_ec2_client, base_region.as_ref(), &found_id)
            .await
            .context(error::IsAmiPublicSnafu {
                image_id: found_id.clone(),
                region: base_region.to_string(),
            })?;

        launch_permissions =
            get_launch_permissions(&base_ec2_client, base_region.as_ref(), &found_id)
                .await
                .context(error::DescribeImageAttributeSnafu {
                    image_id: found_id,
                    region: base_region.to_string(),
                })?;

        (found_ids, true)
    } else {
        let new_ids = register_image(ami_args, &base_region, base_ebs_client, &base_ec2_client)
            .await
            .context(error::RegisterImageSnafu {
                name: &ami_args.name,
                arch: ami_args.arch.as_ref(),
                region: base_region.as_ref(),
            })?;
        info!(
            "Registered AMI '{}' in {}: {}",
            ami_args.name, base_region, new_ids.image_id
        );
        (new_ids, false)
    };

    amis.insert(
        base_region.as_ref().to_string(),
        Image::new(
            &ids_of_image.image_id,
            &ami_args.name,
            Some(public),
            Some(launch_permissions),
        ),
    );

    // If we don't need to copy AMIs, we're done.
    if regions.is_empty() {
        return Ok(amis);
    }

    // Wait for AMI to be available so it can be copied
    let successes_required = if already_registered { 1 } else { 3 };
    wait_for_ami(
        &ids_of_image.image_id,
        &base_region,
        &base_region,
        "available",
        successes_required,
        &aws,
    )
    .await
    .context(error::WaitAmiSnafu {
        id: &ids_of_image.image_id,
        region: base_region.as_ref(),
    })?;

    // For every other region, initiate copy-image calls.

    // First we need to find the account IDs for any given roles, so we can grant access to those
    // accounts to copy the AMI and snapshots.
    info!("Getting account IDs for target regions so we can grant access to copy source AMI");
    let mut account_ids = get_account_ids(&regions, &base_region, &aws).await?;

    // Get the account ID used in the base region; we don't need to grant to it so we can remove it
    // from the list.
    let client_config = build_client_config(&base_region, &base_region, &aws).await;
    let base_sts_client = StsClient::new(&client_config);

    let response = base_sts_client.get_caller_identity().send().await.context(
        error::GetCallerIdentitySnafu {
            region: base_region.as_ref(),
        },
    )?;
    let base_account_id = response.account.context(error::MissingInResponseSnafu {
        request_type: "GetCallerIdentity",
        missing: "account",
    })?;
    account_ids.remove(&base_account_id);

    // If we have any accounts other than the base account, grant them access.
    if !account_ids.is_empty() {
        info!("Granting access to target accounts so we can copy the AMI");
        let account_id_vec: Vec<_> = account_ids.into_iter().collect();

        let modify_options = ModifyOptions {
            user_ids: account_id_vec,
            group_names: Vec::new(),
            organization_arns: Vec::new(),
            organizational_unit_arns: Vec::new(),
        };

        modify_snapshots(
            &modify_options,
            &OperationType::Add,
            &ids_of_image.snapshot_ids,
            &base_ec2_client,
            &base_region,
        )
        .await
        .context(error::GrantAccessSnafu {
            thing: "snapshots",
            region: base_region.as_ref(),
        })?;

        modify_image(
            &modify_options,
            &OperationType::Add,
            &ids_of_image.image_id,
            &base_ec2_client,
        )
        .await
        .context(error::GrantImageAccessSnafu {
            thing: "image",
            region: base_region.as_ref(),
        })?;
    }

    // Next, make EC2 clients so we can fetch and copy AMIs.  We make a map storing our regional
    // clients because they're used in a future and need to live until the future is resolved.
    let mut ec2_clients = HashMap::with_capacity(regions.len());
    for region in regions.iter() {
        let client_config = build_client_config(region, &base_region, &aws).await;
        let ec2_client = Ec2Client::new(&client_config);
        ec2_clients.insert(region.clone(), ec2_client);
    }

    // First, we check if the AMI already exists in each region.
    info!("Checking whether AMIs already exist in target regions");
    let mut get_requests = Vec::with_capacity(regions.len());
    for region in regions.iter() {
        let ec2_client = &ec2_clients[region];
        let get_request = get_ami_id(&ami_args.name, &ami_args.arch, region, ec2_client);
        let info_future = ready(region.clone());
        get_requests.push(join(info_future, get_request));
    }
    let request_stream = stream::iter(get_requests).buffer_unordered(4);
    let get_responses: Vec<(Region, std::result::Result<Option<String>, register::Error>)> =
        request_stream.collect().await;

    // If an AMI already existed, just add it to our list, otherwise prepare a copy request.
    let mut copy_requests = Vec::with_capacity(regions.len());
    for (region, get_response) in get_responses {
        let get_response = get_response.context(error::GetAmiIdSnafu {
            name: &ami_args.name,
            arch: ami_args.arch.as_ref(),
            region: region.as_ref(),
        })?;
        if let Some(id) = get_response {
            info!(
                "Found '{}' already registered in {}: {}",
                ami_args.name, region, id
            );
            let public = ami_is_public(&ec2_clients[&region], region.as_ref(), &id)
                .await
                .context(error::IsAmiPublicSnafu {
                    image_id: id.clone(),
                    region: base_region.to_string(),
                })?;

            let launch_permissions =
                get_launch_permissions(&ec2_clients[&region], region.as_ref(), &id)
                    .await
                    .context(error::DescribeImageAttributeSnafu {
                        region: region.as_ref(),
                        image_id: id.clone(),
                    })?;

            amis.insert(
                region.as_ref().to_string(),
                Image::new(&id, &ami_args.name, Some(public), Some(launch_permissions)),
            );
            continue;
        }

        let ec2_client = &ec2_clients[&region];
        let base_region = base_region.to_owned();
        let copy_future = ec2_client
            .copy_image()
            .set_description(ami_args.description.clone())
            .set_name(Some(ami_args.name.clone()))
            .set_source_image_id(Some(ids_of_image.image_id.clone()))
            .set_source_region(Some(base_region.as_ref().to_string()))
            .send();

        // Store the region so we can output it to the user
        let region_future = ready(region.clone());
        // Let the user know the copy is starting, when this future goes to run
        let message_future =
            lazy(move |_| info!("Starting copy from {} to {}", base_region, region));
        copy_requests.push(message_future.then(|_| join(region_future, copy_future)));
    }

    // If all target regions already have the AMI, we're done.
    if copy_requests.is_empty() {
        return Ok(amis);
    }

    // Start requests; they return almost immediately and the copying work is done by the service
    // afterward.  You should wait for the AMI status to be "available" before launching it.
    // (We still use buffer_unordered, rather than something like join_all, to retain some control
    // over the number of requests going out in case we need it later, but this will effectively
    // spin through all regions quickly because the requests return before any copying is done.)
    let request_stream = stream::iter(copy_requests).buffer_unordered(4);
    // Run through the stream and collect results into a list.
    let copy_responses: Vec<(
        Region,
        std::result::Result<CopyImageOutput, SdkError<CopyImageError>>,
    )> = request_stream.collect().await;

    // Report on successes and errors; don't fail immediately if we see an error so we can report
    // all successful IDs.
    let mut saw_error = false;
    for (region, copy_response) in copy_responses {
        match copy_response {
            Ok(success) => {
                if let Some(image_id) = success.image_id {
                    info!(
                        "Registered AMI '{}' in {}: {}",
                        ami_args.name, region, image_id,
                    );
                    amis.insert(
                        region.as_ref().to_string(),
                        Image::new(&image_id, &ami_args.name, Some(false), Some(vec![])),
                    );
                } else {
                    saw_error = true;
                    error!(
                        "Registered AMI '{}' in {} but didn't receive an AMI ID!",
                        ami_args.name, region,
                    );
                }
            }
            Err(e) => {
                saw_error = true;
                error!(
                    "Copy to {} failed: {}",
                    region,
                    e.into_service_error().code().unwrap_or("unknown")
                );
            }
        }
    }

    ensure!(!saw_error, error::AmiCopySnafu);

    Ok(amis)
}

/// If JSON output was requested, we serialize out a mapping of region to AMI information; this
/// struct holds the information we save about each AMI.  The `ssm` subcommand uses this
/// information to populate templates representing SSM parameter names and values.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub(crate) struct Image {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) public: Option<bool>,
    pub(crate) launch_permissions: Option<Vec<LaunchPermissionDef>>,
}

impl Image {
    fn new(
        id: &str,
        name: &str,
        public: Option<bool>,
        launch_permissions: Option<Vec<LaunchPermissionDef>>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            public,
            launch_permissions,
        }
    }
}

/// Returns the set of account IDs associated with the roles configured for the given regions.
async fn get_account_ids(
    regions: &[Region],
    base_region: &Region,
    pubsys_aws_config: &PubsysAwsConfig,
) -> Result<HashSet<String>> {
    let mut grant_accounts = HashSet::new();

    // We make a map storing our regional clients because they're used in a future and need to
    // live until the future is resolved.
    let mut sts_clients = HashMap::with_capacity(regions.len());
    for region in regions.iter() {
        let client_config = build_client_config(region, base_region, pubsys_aws_config).await;
        let sts_client = StsClient::new(&client_config);
        sts_clients.insert(region.clone(), sts_client);
    }

    let mut requests = Vec::with_capacity(regions.len());
    for region in regions.iter() {
        let sts_client = &sts_clients[region];
        let response_future = sts_client.get_caller_identity().send();

        // Store the region so we can include it in any errors
        let region_future = ready(region.clone());
        requests.push(join(region_future, response_future));
    }

    let request_stream = stream::iter(requests).buffer_unordered(4);
    // Run through the stream and collect results into a list.
    let responses: Vec<(
        Region,
        std::result::Result<GetCallerIdentityOutput, SdkError<GetCallerIdentityError>>,
    )> = request_stream.collect().await;

    for (region, response) in responses {
        let response = response.context(error::GetCallerIdentitySnafu {
            region: region.as_ref(),
        })?;
        let account_id = response.account.context(error::MissingInResponseSnafu {
            request_type: "GetCallerIdentity",
            missing: "account",
        })?;
        grant_accounts.insert(account_id);
    }
    trace!("Found account IDs {:?}", grant_accounts);

    Ok(grant_accounts)
}

mod error {
    use crate::aws::{ami, publish_ami};
    use aws_sdk_ec2::error::ModifyImageAttributeError;
    use aws_sdk_ec2::model::LaunchPermission;
    use aws_sdk_ec2::types::SdkError;
    use aws_sdk_sts::error::GetCallerIdentityError;
    use snafu::Snafu;
    use std::path::PathBuf;

    use super::public;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Some AMIs failed to copy, see above"))]
        AmiCopy,

        #[snafu(display("Error reading config: {}", source))]
        Config { source: pubsys_config::Error },

        #[snafu(display(
            "Failed to describe image attributes for image {} in region {}: {}",
            image_id,
            region,
            source
        ))]
        DescribeImageAttribute {
            image_id: String,
            region: String,
            source: super::launch_permissions::Error,
        },

        #[snafu(display("Failed to create file '{}': {}", path.display(), source))]
        FileCreate {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Error getting AMI ID for {} {} in {}: {}", arch, name, region, source))]
        GetAmiId {
            name: String,
            arch: String,
            region: String,
            source: ami::register::Error,
        },

        #[snafu(display("Error getting account ID in {}: {}", region, source))]
        GetCallerIdentity {
            region: String,
            source: SdkError<GetCallerIdentityError>,
        },

        #[snafu(display(
            "Failed to get snapshot IDs associated with {} in {}: {}",
            image_id,
            region,
            source
        ))]
        GetSnapshots {
            image_id: String,
            region: String,
            source: publish_ami::Error,
        },

        #[snafu(display("Failed to grant access to {} in {}: {}", thing, region, source))]
        GrantAccess {
            thing: String,
            region: String,
            source: publish_ami::Error,
        },

        #[snafu(display("Failed to grant access to {} in {}: {}", thing, region, source))]
        GrantImageAccess {
            thing: String,
            region: String,
            source: SdkError<ModifyImageAttributeError>,
        },

        #[snafu(display(
            "Failed to check if AMI with id {} is public in {}: {}",
            image_id,
            region,
            source
        ))]
        IsAmiPublic {
            image_id: String,
            region: String,
            source: public::Error,
        },

        #[snafu(display("Invalid launch permission: {:?}", launch_permission))]
        InvalidLaunchPermission { launch_permission: LaunchPermission },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig { missing: String },

        #[snafu(display("Response to {} was missing {}", request_type, missing))]
        MissingInResponse {
            request_type: String,
            missing: String,
        },

        #[snafu(display("Error registering {} {} in {}: {}", arch, name, region, source))]
        RegisterImage {
            name: String,
            arch: String,
            region: String,
            source: ami::register::Error,
        },

        #[snafu(display("AMI '{}' in {} did not become available: {}", id, region, source))]
        WaitAmi {
            id: String,
            region: String,
            source: ami::wait::Error,
        },

        #[snafu(display("Failed to write AMIs to '{}': {}", path.display(), source))]
        WriteAmis {
            path: PathBuf,
            source: publish_ami::Error,
        },
    }
}
pub(crate) use error::Error;

use self::launch_permissions::LaunchPermissionDef;

use super::publish_ami::write_amis;
type Result<T> = std::result::Result<T, error::Error>;
