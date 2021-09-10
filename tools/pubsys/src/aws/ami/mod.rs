//! The ami module owns the 'ami' subcommand and controls the process of registering and copying
//! EC2 AMIs.

mod register;
mod snapshot;
pub(crate) mod wait;

use crate::aws::publish_ami::{get_snapshots, modify_image, modify_snapshots};
use crate::aws::{client::build_client, parse_arch, region_from_string};
use crate::Args;
use futures::future::{join, lazy, ready, FutureExt};
use futures::stream::{self, StreamExt};
use log::{error, info, trace, warn};
use pubsys_config::{AwsConfig, InfraConfig};
use register::{get_ami_id, register_image, RegisteredIds};
use rusoto_core::{Region, RusotoError};
use rusoto_ebs::EbsClient;
use rusoto_ec2::{CopyImageError, CopyImageRequest, CopyImageResult, Ec2, Ec2Client};
use rusoto_sts::{
    GetCallerIdentityError, GetCallerIdentityRequest, GetCallerIdentityResponse, Sts, StsClient,
};
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use wait::wait_for_ami;

/// Builds Bottlerocket AMIs using latest build artifacts
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub(crate) struct AmiArgs {
    /// Path to the image containing the root volume
    #[structopt(short = "r", long, parse(from_os_str))]
    root_image: PathBuf,

    /// Path to the image containing the data volume
    #[structopt(short = "d", long, parse(from_os_str))]
    data_image: PathBuf,

    /// Desired root volume size in gibibytes
    #[structopt(long)]
    root_volume_size: Option<i64>,

    /// Desired data volume size in gibibytes
    #[structopt(long)]
    data_volume_size: i64,

    /// The architecture of the machine image
    #[structopt(short = "a", long, parse(try_from_str = parse_arch))]
    arch: String,

    /// The desired AMI name
    #[structopt(short = "n", long)]
    name: String,

    /// The desired AMI description
    #[structopt(long)]
    description: Option<String>,

    /// Don't display progress bars
    #[structopt(long)]
    no_progress: bool,

    /// Regions where you want the AMI, the first will be used as the base for copying
    #[structopt(long, use_delimiter = true)]
    regions: Vec<String>,

    /// If specified, save created regional AMI IDs in JSON at this path.
    #[structopt(long)]
    ami_output: Option<PathBuf>,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, ami_args: &AmiArgs) -> Result<()> {
    match _run(args, ami_args).await {
        Ok(amis) => {
            // Write the AMI IDs to file if requested
            if let Some(ref path) = ami_args.ami_output {
                let file = File::create(path).context(error::FileCreate { path })?;
                serde_json::to_writer_pretty(file, &amis).context(error::Serialize { path })?;
                info!("Wrote AMI data to {}", path.display());
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

async fn _run(args: &Args, ami_args: &AmiArgs) -> Result<HashMap<String, Image>> {
    let mut amis = HashMap::new();

    // If a lock file exists, use that, otherwise use Infra.toml or default
    let infra_config =
        InfraConfig::from_path_or_lock(&args.infra_config_path, true).context(error::Config)?;
    trace!("Using infra config: {:?}", infra_config);

    let aws = infra_config.aws.unwrap_or_else(|| Default::default());

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let mut regions = if !ami_args.regions.is_empty() {
        ami_args.regions.clone()
    } else {
        aws.regions.clone().into()
    }
    .into_iter()
    .map(|name| region_from_string(&name, &aws).context(error::ParseRegion))
    .collect::<Result<Vec<Region>>>()?;

    ensure!(
        !regions.is_empty(),
        error::MissingConfig {
            missing: "aws.regions"
        }
    );

    // We register in this base region first, then copy from there to any other regions.
    let base_region = regions.remove(0);

    // Build EBS client for snapshot management, and EC2 client for registration
    let base_ebs_client =
        build_client::<EbsClient>(&base_region, &base_region, &aws).context(error::Client {
            client_type: "EBS",
            region: base_region.name(),
        })?;
    let base_ec2_client =
        build_client::<Ec2Client>(&base_region, &base_region, &aws).context(error::Client {
            client_type: "EC2",
            region: base_region.name(),
        })?;

    // Check if the AMI already exists, in which case we can use the existing ID, otherwise we
    // register a new one.
    let maybe_id = get_ami_id(
        &ami_args.name,
        &ami_args.arch,
        base_region.name(),
        &base_ec2_client,
    )
    .await
    .context(error::GetAmiId {
        name: &ami_args.name,
        arch: &ami_args.arch,
        region: base_region.name(),
    })?;

    let (ids_of_image, already_registered) = if let Some(found_id) = maybe_id {
        warn!(
            "Found '{}' already registered in {}: {}",
            ami_args.name,
            base_region.name(),
            found_id
        );
        let snapshot_ids = get_snapshots(&found_id, &base_region, &base_ec2_client)
            .await
            .context(error::GetSnapshots {
                image_id: &found_id,
                region: base_region.name(),
            })?;
        let found_ids = RegisteredIds {
            image_id: found_id,
            snapshot_ids,
        };
        (found_ids, true)
    } else {
        let new_ids = register_image(
            ami_args,
            base_region.name(),
            base_ebs_client,
            &base_ec2_client,
        )
        .await
        .context(error::RegisterImage {
            name: &ami_args.name,
            arch: &ami_args.arch,
            region: base_region.name(),
        })?;
        info!(
            "Registered AMI '{}' in {}: {}",
            ami_args.name,
            base_region.name(),
            new_ids.image_id
        );
        (new_ids, false)
    };

    amis.insert(
        base_region.name().to_string(),
        Image::new(&ids_of_image.image_id, &ami_args.name),
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
    .context(error::WaitAmi {
        id: &ids_of_image.image_id,
        region: base_region.name(),
    })?;

    // For every other region, initiate copy-image calls.

    // First we need to find the account IDs for any given roles, so we can grant access to those
    // accounts to copy the AMI and snapshots.
    info!("Getting account IDs for target regions so we can grant access to copy source AMI");
    let mut account_ids = get_account_ids(&regions, &base_region, &aws).await?;

    // Get the account ID used in the base region; we don't need to grant to it so we can remove it
    // from the list.
    let base_sts_client =
        build_client::<StsClient>(&base_region, &base_region, &aws).context(error::Client {
            client_type: "STS",
            region: base_region.name(),
        })?;
    let response = base_sts_client
        .get_caller_identity(GetCallerIdentityRequest {})
        .await
        .context(error::GetCallerIdentity {
            region: base_region.name(),
        })?;
    let base_account_id = response.account.context(error::MissingInResponse {
        request_type: "GetCallerIdentity",
        missing: "account",
    })?;
    account_ids.remove(&base_account_id);

    // If we have any accounts other than the base account, grant them access.
    if !account_ids.is_empty() {
        info!("Granting access to target accounts so we can copy the AMI");
        let account_id_vec: Vec<_> = account_ids.into_iter().collect();

        modify_snapshots(
            Some(account_id_vec.clone()),
            None,
            "add",
            &ids_of_image.snapshot_ids,
            &base_ec2_client,
            &base_region,
        )
        .await
        .context(error::GrantAccess {
            thing: "snapshots",
            region: base_region.name(),
        })?;

        modify_image(
            Some(account_id_vec.clone()),
            None,
            "add",
            &ids_of_image.image_id,
            &base_ec2_client,
            &base_region,
        )
        .await
        .context(error::GrantAccess {
            thing: "image",
            region: base_region.name(),
        })?;
    }

    // Next, make EC2 clients so we can fetch and copy AMIs.  We make a map storing our regional
    // clients because they're used in a future and need to live until the future is resolved.
    let mut ec2_clients = HashMap::with_capacity(regions.len());
    for region in regions.iter() {
        let ec2_client =
            build_client::<Ec2Client>(&region, &base_region, &aws).context(error::Client {
                client_type: "EC2",
                region: region.name(),
            })?;
        ec2_clients.insert(region.clone(), ec2_client);
    }

    // First, we check if the AMI already exists in each region.
    info!("Checking whether AMIs already exist in target regions");
    let mut get_requests = Vec::with_capacity(regions.len());
    for region in regions.iter() {
        let ec2_client = &ec2_clients[region];
        let get_request = get_ami_id(&ami_args.name, &ami_args.arch, region.name(), ec2_client);
        let info_future = ready(region.clone());
        get_requests.push(join(info_future, get_request));
    }
    let request_stream = stream::iter(get_requests).buffer_unordered(4);
    let get_responses: Vec<(Region, std::result::Result<Option<String>, register::Error>)> =
        request_stream.collect().await;

    // If an AMI already existed, just add it to our list, otherwise prepare a copy request.
    let mut copy_requests = Vec::with_capacity(regions.len());
    for (region, get_response) in get_responses {
        let get_response = get_response.context(error::GetAmiId {
            name: &ami_args.name,
            arch: &ami_args.arch,
            region: region.name(),
        })?;
        if let Some(id) = get_response {
            info!(
                "Found '{}' already registered in {}: {}",
                ami_args.name,
                region.name(),
                id
            );
            amis.insert(region.name().to_string(), Image::new(&id, &ami_args.name));
            continue;
        }

        let ec2_client = &ec2_clients[&region];
        let copy_request = CopyImageRequest {
            description: ami_args.description.clone(),
            name: ami_args.name.clone(),
            source_image_id: ids_of_image.image_id.clone(),
            source_region: base_region.name().to_string(),
            ..Default::default()
        };
        let copy_future = ec2_client.copy_image(copy_request);

        let base_region_name = base_region.name();
        // Store the region so we can output it to the user
        let region_future = ready(region.clone());
        // Let the user know the copy is starting, when this future goes to run
        let message_future = lazy(move |_| {
            info!(
                "Starting copy from {} to {}",
                base_region_name,
                region.name()
            )
        });
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
        std::result::Result<CopyImageResult, RusotoError<CopyImageError>>,
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
                        ami_args.name,
                        region.name(),
                        image_id,
                    );
                    amis.insert(
                        region.name().to_string(),
                        Image::new(&image_id, &ami_args.name),
                    );
                } else {
                    saw_error = true;
                    error!(
                        "Registered AMI '{}' in {} but didn't receive an AMI ID!",
                        ami_args.name,
                        region.name(),
                    );
                }
            }
            Err(e) => {
                saw_error = true;
                error!("Copy to {} failed: {}", region.name(), e);
            }
        }
    }

    ensure!(!saw_error, error::AmiCopy);

    Ok(amis)
}

/// If JSON output was requested, we serialize out a mapping of region to AMI information; this
/// struct holds the information we save about each AMI.  The `ssm` subcommand uses this
/// information to populate templates representing SSM parameter names and values.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Image {
    pub(crate) id: String,
    pub(crate) name: String,
}

impl Image {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

/// Returns the set of account IDs associated with the roles configured for the given regions.
async fn get_account_ids(
    regions: &[Region],
    base_region: &Region,
    aws: &AwsConfig,
) -> Result<HashSet<String>> {
    let mut grant_accounts = HashSet::new();

    // We make a map storing our regional clients because they're used in a future and need to
    // live until the future is resolved.
    let mut sts_clients = HashMap::with_capacity(regions.len());
    for region in regions.iter() {
        let sts_client =
            build_client::<StsClient>(&region, &base_region, &aws).context(error::Client {
                client_type: "STS",
                region: region.name(),
            })?;
        sts_clients.insert(region.clone(), sts_client);
    }

    let mut requests = Vec::with_capacity(regions.len());
    for region in regions.iter() {
        let sts_client = &sts_clients[region];
        let response_future = sts_client.get_caller_identity(GetCallerIdentityRequest {});

        // Store the region so we can include it in any errors
        let region_future = ready(region.clone());
        requests.push(join(region_future, response_future));
    }

    let request_stream = stream::iter(requests).buffer_unordered(4);
    // Run through the stream and collect results into a list.
    let responses: Vec<(
        Region,
        std::result::Result<GetCallerIdentityResponse, RusotoError<GetCallerIdentityError>>,
    )> = request_stream.collect().await;

    for (region, response) in responses {
        let response = response.context(error::GetCallerIdentity {
            region: region.name(),
        })?;
        let account_id = response.account.context(error::MissingInResponse {
            request_type: "GetCallerIdentity",
            missing: "account",
        })?;
        grant_accounts.insert(account_id);
    }
    trace!("Found account IDs {:?}", grant_accounts);

    Ok(grant_accounts)
}

mod error {
    use crate::aws::{self, ami, publish_ami};
    use rusoto_core::RusotoError;
    use rusoto_sts::GetCallerIdentityError;
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Some AMIs failed to copy, see above"))]
        AmiCopy,

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
            source: RusotoError<GetCallerIdentityError>,
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

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig {
            missing: String,
        },

        #[snafu(display("Response to {} was missing {}", request_type, missing))]
        MissingInResponse {
            request_type: String,
            missing: String,
        },

        ParseRegion {
            source: crate::aws::Error,
        },

        #[snafu(display("Error registering {} {} in {}: {}", arch, name, region, source))]
        RegisterImage {
            name: String,
            arch: String,
            region: String,
            source: ami::register::Error,
        },

        #[snafu(display("Failed to serialize output to '{}': {}", path.display(), source))]
        Serialize {
            path: PathBuf,
            source: serde_json::Error,
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
