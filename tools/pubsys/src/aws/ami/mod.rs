//! The ami module owns the 'ami' subcommand and controls the process of registering and copying
//! EC2 AMIs.

mod register;
mod snapshot;
mod wait;

use crate::aws::{client::build_client, region_from_string};
use crate::config::InfraConfig;
use crate::Args;
use futures::future::{join, lazy, ready, FutureExt};
use futures::stream::{self, StreamExt};
use log::{error, info, trace};
use register::{get_ami_id, register_image};
use rusoto_core::{Region, RusotoError};
use rusoto_ebs::EbsClient;
use rusoto_ec2::{CopyImageError, CopyImageRequest, CopyImageResult, Ec2, Ec2Client};
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, VecDeque};
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
    #[structopt(short = "a", long)]
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

    info!(
        "Using infra config from path: {}",
        args.infra_config_path.display()
    );
    let infra_config = InfraConfig::from_path(&args.infra_config_path).context(error::Config)?;
    trace!("Parsed infra config: {:?}", infra_config);

    let aws = infra_config.aws.unwrap_or_else(|| Default::default());

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let mut regions = if !ami_args.regions.is_empty() {
        VecDeque::from(ami_args.regions.clone())
    } else {
        aws.regions.clone()
    }
    .into_iter()
    .map(|name| region_from_string(&name, &aws).context(error::ParseRegion))
    .collect::<Result<VecDeque<Region>>>()?;

    // We register in this base region first, then copy from there to any other regions.
    let base_region = regions.pop_front().context(error::MissingConfig {
        missing: "aws.regions",
    })?;

    // Build EBS client for snapshot management, and EC2 client for registration
    let ebs_client = build_client::<EbsClient>(&base_region, &base_region, &aws).context(error::Client {
        client_type: "EBS",
        region: base_region.name(),
    })?;
    let ec2_client = build_client::<Ec2Client>(&base_region, &base_region, &aws).context(error::Client {
        client_type: "EC2",
        region: base_region.name(),
    })?;

    // Check if the AMI already exists, in which case we can use the existing ID, otherwise we
    // register a new one.
    let maybe_id = get_ami_id(
        &ami_args.name,
        &ami_args.arch,
        base_region.name(),
        &ec2_client,
    )
    .await
    .context(error::GetAmiId {
        name: &ami_args.name,
        arch: &ami_args.arch,
        region: base_region.name(),
    })?;

    let (image_id, already_registered) = if let Some(found_id) = maybe_id {
        info!(
            "Found '{}' already registered in {}: {}",
            ami_args.name,
            base_region.name(),
            found_id
        );
        (found_id, true)
    } else {
        let new_id = register_image(ami_args, base_region.name(), ebs_client, &ec2_client)
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
            new_id
        );
        (new_id, false)
    };

    amis.insert(
        base_region.name().to_string(),
        Image::new(&image_id, &ami_args.name),
    );

    // If we don't need to copy AMIs, we're done.
    if regions.is_empty() {
        return Ok(amis);
    }

    // Wait for AMI to be available so it can be copied
    let successes_required = if already_registered { 1 } else { 3 };
    wait_for_ami(
        &image_id,
        &base_region,
        &base_region,
        "available",
        successes_required,
        &aws,
    )
    .await
    .context(error::WaitAmi {
        id: &image_id,
        region: base_region.name(),
    })?;

    // For every other region, initiate copy-image calls.
    // We make a map storing our regional clients because they're used in a future and need to
    // live until the future is resolved.
    let mut ec2_clients = HashMap::with_capacity(regions.len());
    for region in regions.iter() {
        let ec2_client = build_client::<Ec2Client>(&region, &base_region, &aws).context(error::Client {
            client_type: "EC2",
            region: base_region.name(),
        })?;
        ec2_clients.insert(region.clone(), ec2_client);
    }

    let mut copy_requests = Vec::with_capacity(regions.len());
    for region in regions.iter() {
        let ec2_client = &ec2_clients[region];
        if let Some(id) = get_ami_id(&ami_args.name, &ami_args.arch, region.name(), ec2_client)
            .await
            .context(error::GetAmiId {
                name: &ami_args.name,
                arch: &ami_args.arch,
                region: base_region.name(),
            })?
        {
            info!(
                "Found '{}' already registered in {}: {}",
                ami_args.name,
                region.name(),
                id
            );
            amis.insert(region.name().to_string(), Image::new(&id, &ami_args.name));
            continue;
        }
        let request = CopyImageRequest {
            description: ami_args.description.clone(),
            name: ami_args.name.clone(),
            source_image_id: image_id.clone(),
            source_region: base_region.name().to_string(),
            ..Default::default()
        };
        let response_future = ec2_client.copy_image(request);

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
        copy_requests.push(message_future.then(|_| join(region_future, response_future)));
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

mod error {
    use crate::aws::{self, ami};
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
            source: crate::config::Error,
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

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig {
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
