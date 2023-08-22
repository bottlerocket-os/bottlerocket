//! The ssm module owns the 'ssm' subcommand and controls the process of setting SSM parameters
//! based on current build information

#[allow(clippy::module_inception)]
pub(crate) mod ssm;
pub(crate) mod template;

use self::template::RenderedParameter;
use crate::aws::ssm::template::RenderedParametersMap;
use crate::aws::{
    ami::public::ami_is_public, ami::Image, client::build_client_config, parse_arch,
    region_from_string,
};
use crate::Args;
use aws_config::SdkConfig;
use aws_sdk_ec2::{types::ArchitectureValues, Client as Ec2Client};
use aws_sdk_ssm::{config::Region, Client as SsmClient};
use clap::Parser;
use futures::stream::{StreamExt, TryStreamExt};
use governor::{prelude::*, Quota, RateLimiter};
use log::{error, info, trace};
use nonzero_ext::nonzero;
use pubsys_config::InfraConfig;
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
};

/// Sets SSM parameters based on current build information
#[derive(Debug, Parser)]
pub(crate) struct SsmArgs {
    // This is JSON output from `pubsys ami` like `{"us-west-2": "ami-123"}`
    /// Path to the JSON file containing regional AMI IDs to modify
    #[arg(long)]
    ami_input: PathBuf,

    /// The architecture of the machine image
    #[arg(long, value_parser = parse_arch)]
    arch: ArchitectureValues,

    /// The variant name for the current build
    #[arg(long)]
    variant: String,

    /// The version of the current build
    #[arg(long)]
    version: String,

    /// Regions where you want parameters published
    #[arg(long, value_delimiter = ',')]
    regions: Vec<String>,

    /// File holding the parameter templates
    #[arg(long)]
    template_path: PathBuf,

    /// Allows overwrite of existing parameters
    #[arg(long)]
    allow_clobber: bool,

    /// Allows publishing non-public images to the `/aws/` namespace
    #[arg(long)]
    allow_private_images: bool,

    /// If set, writes the generated SSM parameters to this path
    #[arg(long)]
    ssm_parameter_output: Option<PathBuf>,
}

/// Wrapper struct over parameter update and AWS clients needed to execute on it.
#[derive(Debug, Clone)]
struct SsmParamUpdateOp {
    parameter: RenderedParameter,
    ec2_client: Ec2Client,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, ssm_args: &SsmArgs) -> Result<()> {
    // Setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, false)
        .context(error::ConfigSnafu)?;
    trace!("Parsed infra config: {:#?}", infra_config);
    let aws = infra_config.aws.unwrap_or_default();
    let ssm_prefix = aws.ssm_prefix.as_deref().unwrap_or("");

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let regions = if !ssm_args.regions.is_empty() {
        ssm_args.regions.clone()
    } else {
        aws.regions.clone().into()
    };
    ensure!(
        !regions.is_empty(),
        error::MissingConfigSnafu {
            missing: "aws.regions"
        }
    );
    let base_region = region_from_string(&regions[0]);

    let amis = parse_ami_input(&regions, ssm_args)?;

    // Template setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Non-image-specific context for building and rendering templates
    let build_context = BuildContext {
        variant: &ssm_args.variant,
        arch: ssm_args.arch.as_ref(),
        image_version: &ssm_args.version,
    };

    info!(
        "Parsing SSM parameter templates from {}",
        ssm_args.template_path.display()
    );
    let template_parameters = template::get_parameters(&ssm_args.template_path, &build_context)
        .context(error::FindTemplatesSnafu)?;

    if template_parameters.parameters.is_empty() {
        info!(
            "No parameters for this arch/variant in {}",
            ssm_args.template_path.display()
        );
        return Ok(());
    }

    let new_parameters =
        template::render_parameters(template_parameters, &amis, ssm_prefix, &build_context)
            .context(error::RenderTemplatesSnafu)?;
    trace!("Generated templated parameters: {:#?}", new_parameters);

    // If the path to an output file was given, write the rendered parameters to this file
    if let Some(ssm_parameter_output) = &ssm_args.ssm_parameter_output {
        write_rendered_parameters(
            ssm_parameter_output,
            &RenderedParametersMap::from(&new_parameters).rendered_parameters,
        )?;
    }

    // Generate AWS Clients to use for the updates.
    let mut param_update_ops: Vec<SsmParamUpdateOp> = Vec::with_capacity(new_parameters.len());
    let mut aws_sdk_configs: HashMap<Region, SdkConfig> = HashMap::with_capacity(regions.len());
    let mut ssm_clients = HashMap::with_capacity(amis.len());

    for parameter in new_parameters.iter() {
        let region = &parameter.ssm_key.region;
        // Store client configs so that we only have to create them once.
        // The HashMap `entry` API doesn't play well with `async`, so we use a match here instead.
        let client_config = match aws_sdk_configs.get(region) {
            Some(client_config) => client_config.clone(),
            None => {
                let client_config = build_client_config(region, &base_region, &aws).await;
                aws_sdk_configs.insert(region.clone(), client_config.clone());
                client_config
            }
        };

        let ssm_client = SsmClient::new(&client_config);
        if ssm_clients.get(region).is_none() {
            ssm_clients.insert(region.clone(), ssm_client);
        }

        let ec2_client = Ec2Client::new(&client_config);
        param_update_ops.push(SsmParamUpdateOp {
            parameter: parameter.clone(),
            ec2_client,
        });
    }

    // Unless overridden, only allow public images to be published to public parameters.
    if !ssm_args.allow_private_images {
        info!("Ensuring that only public images are published to public parameters.");
        ensure!(
            check_public_namespace_amis_are_public(param_update_ops.iter()).await?,
            error::NoPrivateImagesSnafu
        );
    }

    // SSM get/compare   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Getting current SSM parameters");
    let new_parameter_names: Vec<&SsmKey> =
        new_parameters.iter().map(|param| &param.ssm_key).collect();
    let current_parameters = ssm::get_parameters(&new_parameter_names, &ssm_clients)
        .await
        .context(error::FetchSsmSnafu)?;
    trace!("Current SSM parameters: {:#?}", current_parameters);

    // Show the difference between source and target parameters in SSM.
    let parameters_to_set = key_difference(
        &RenderedParameter::as_ssm_parameters(&new_parameters),
        &current_parameters,
    );
    if parameters_to_set.is_empty() {
        info!("No changes necessary.");
        return Ok(());
    }

    // Unless the user wants to allow it, make sure we're not going to overwrite any existing
    // keys.
    if !ssm_args.allow_clobber {
        let current_keys: HashSet<&SsmKey> = current_parameters.keys().collect();
        let new_keys: HashSet<&SsmKey> = parameters_to_set.keys().collect();
        ensure!(current_keys.is_disjoint(&new_keys), error::NoClobberSnafu);
    }

    // SSM set   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Setting updated SSM parameters.");
    ssm::set_parameters(&parameters_to_set, &ssm_clients)
        .await
        .context(error::SetSsmSnafu)?;

    info!("Validating whether live parameters in SSM reflect changes.");
    ssm::validate_parameters(&parameters_to_set, &ssm_clients)
        .await
        .context(error::ValidateSsmSnafu)?;

    info!("All parameters match requested values.");
    Ok(())
}

/// Write rendered parameters to the file at `ssm_parameters_output`
pub(crate) fn write_rendered_parameters(
    ssm_parameters_output: &PathBuf,
    parameters: &HashMap<String, HashMap<String, String>>,
) -> Result<()> {
    info!(
        "Writing rendered SSM parameters to {:#?}",
        ssm_parameters_output
    );

    serde_json::to_writer_pretty(
        &File::create(ssm_parameters_output).context(error::WriteRenderedSsmParametersSnafu {
            path: ssm_parameters_output,
        })?,
        &parameters,
    )
    .context(error::ParseRenderedSsmParametersSnafu)?;

    info!(
        "Wrote rendered SSM parameters to {:#?}",
        ssm_parameters_output
    );
    Ok(())
}

// Rate limits on the EC2 side use the TokenBucket method, and buckets refill at a rate of 20 tokens per second.
// See https://docs.aws.amazon.com/AWSEC2/latest/APIReference/throttling.html#throttling-rate-based for more details.
const DESCRIBE_IMAGES_RATE_LIMIT: Quota = Quota::per_second(nonzero!(20u32));
const MAX_CONCURRENT_AMI_CHECKS: usize = 8;

/// Given a set of SSM parameter updates, ensures all parameters in the public namespace refer to public AMIs.
async fn check_public_namespace_amis_are_public(
    parameter_updates: impl Iterator<Item = &SsmParamUpdateOp>,
) -> Result<bool> {
    let public_namespace_updates = parameter_updates
        .filter(|update| update.parameter.ssm_key.is_in_public_namespace())
        .cloned();

    // Wrap `crate::aws::ami::public::ami_is_public()` in a future that returns the correct error type.
    let check_ami_public = |update: SsmParamUpdateOp| async move {
        let region = &update.parameter.ssm_key.region;
        let ami_id = &update.parameter.ami.id;
        let is_public = ami_is_public(&update.ec2_client, region.as_ref(), ami_id)
            .await
            .context(error::CheckAmiPublicSnafu {
                ami_id: ami_id.to_string(),
                region: region.to_string(),
            });

        if let Ok(false) = is_public {
            error!(
                "Attempted to set parameter '{}' in {} to '{}', based on AMI {}. That AMI is not marked public!",
                update.parameter.ssm_key.name, region, update.parameter.value, ami_id
            );
        }

        is_public
    };

    // Concurrently check our input parameter updates...
    let rate_limiter = RateLimiter::direct(DESCRIBE_IMAGES_RATE_LIMIT);
    let results: Vec<Result<bool>> = futures::stream::iter(public_namespace_updates)
        .ratelimit_stream(&rate_limiter)
        .then(|update| async move { Ok(check_ami_public(update)) })
        .try_buffer_unordered(usize::min(num_cpus::get(), MAX_CONCURRENT_AMI_CHECKS))
        .collect()
        .await;

    // `collect()` on `TryStreams` doesn't seem to happily invert a `Vec<Result<_>>` to a `Result<Vec<_>>`,
    // so we use the usual `Iterator` methods to do it here.
    Ok(results
        .into_iter()
        .collect::<Result<Vec<bool>>>()?
        .into_iter()
        .all(|is_public| is_public))
}

/// The key to a unique SSM parameter
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) struct SsmKey {
    pub(crate) region: Region,
    pub(crate) name: String,
}

impl SsmKey {
    pub(crate) fn new(region: Region, name: String) -> Self {
        Self { region, name }
    }

    pub(crate) fn is_in_public_namespace(&self) -> bool {
        self.name.starts_with("/aws/")
    }
}

impl AsRef<SsmKey> for SsmKey {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// Non-image-specific context for building and rendering templates
#[derive(Debug, Serialize)]
pub(crate) struct BuildContext<'a> {
    pub(crate) variant: &'a str,
    pub(crate) arch: &'a str,
    pub(crate) image_version: &'a str,
}

/// A map of SsmKey to its value
pub(crate) type SsmParameters = HashMap<SsmKey, String>;

/// Parse the AMI input file
fn parse_ami_input(regions: &[String], ssm_args: &SsmArgs) -> Result<HashMap<Region, Image>> {
    info!("Using AMI data from path: {}", ssm_args.ami_input.display());
    let file = File::open(&ssm_args.ami_input).context(error::FileSnafu {
        op: "open",
        path: &ssm_args.ami_input,
    })?;
    let mut ami_input: HashMap<String, Image> =
        serde_json::from_reader(file).context(error::DeserializeSnafu {
            path: &ssm_args.ami_input,
        })?;
    trace!("Parsed AMI input: {:#?}", ami_input);

    // pubsys will not create a file if it did not create AMIs, so we should only have an empty
    // file if a user created one manually, and they shouldn't be creating an empty file.
    ensure!(
        !ami_input.is_empty(),
        error::InputSnafu {
            path: &ssm_args.ami_input
        }
    );

    // Check that the requested regions are a subset of the regions we *could* publish from the AMI
    // input JSON.
    let requested_regions = HashSet::from_iter(regions.iter());
    let known_regions = HashSet::<&String>::from_iter(ami_input.keys());
    ensure!(
        requested_regions.is_subset(&known_regions),
        error::UnknownRegionsSnafu {
            regions: requested_regions
                .difference(&known_regions)
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        }
    );

    // Parse region names
    let mut amis = HashMap::with_capacity(regions.len());
    for name in regions {
        let image = ami_input
            .remove(name)
            // This could only happen if someone removes the check above...
            .with_context(|| error::UnknownRegionsSnafu {
                regions: vec![name.clone()],
            })?;
        let region = region_from_string(name);
        amis.insert(region.clone(), image);
    }

    Ok(amis)
}

/// Shows the user the difference between two sets of parameters.  We look for parameters in
/// `wanted` that are either missing or changed in `current`.  We print these differences for the
/// user, then return the `wanted` values.
pub(crate) fn key_difference(wanted: &SsmParameters, current: &SsmParameters) -> SsmParameters {
    let mut parameters_to_set = HashMap::new();

    let wanted_keys: HashSet<&SsmKey> = wanted.keys().collect();
    let current_keys: HashSet<&SsmKey> = current.keys().collect();

    for key in wanted_keys.difference(&current_keys) {
        let new_value = &wanted[key];
        println!(
            "{} - {} - new parameter:\n   new value: {}",
            key.name, key.region, new_value,
        );
        parameters_to_set.insert(
            SsmKey::new(key.region.clone(), key.name.clone()),
            new_value.clone(),
        );
    }

    for key in wanted_keys.intersection(&current_keys) {
        let current_value = &current[key];
        let new_value = &wanted[key];

        if current_value == new_value {
            println!("{} - {} - no change", key.name, key.region);
        } else {
            println!(
                "{} - {} - changing value:\n   old value: {}\n   new value: {}",
                key.name, key.region, current_value, new_value
            );
            parameters_to_set.insert(
                SsmKey::new(key.region.clone(), key.name.clone()),
                new_value.clone(),
            );
        }
    }
    // Note: don't care about items that are in current but not wanted; that could happen if you
    // remove a parameter from your templates, for example.

    parameters_to_set
}

mod error {
    use crate::aws::ssm::{ssm, template};
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Error reading config: {}", source))]
        Config {
            source: pubsys_config::Error,
        },

        #[snafu(display(
            "Failed to check whether AMI {} in {} was public: {}",
            ami_id,
            region,
            source
        ))]
        CheckAmiPublic {
            ami_id: String,
            region: String,
            source: crate::aws::ami::public::Error,
        },

        #[snafu(display("Failed to create EC2 client for region {}", region))]
        CreateEc2Client {
            region: String,
        },

        #[snafu(display("Failed to deserialize input from '{}': {}", path.display(), source))]
        Deserialize {
            path: PathBuf,
            source: serde_json::Error,
        },

        #[snafu(display("Failed to fetch parameters from SSM: {}", source))]
        FetchSsm {
            source: ssm::Error,
        },

        #[snafu(display("Failed to {} '{}': {}", op, path.display(), source))]
        File {
            op: String,
            path: PathBuf,
            source: io::Error,
        },

        #[snafu(display("Failed to find templates: {}", source))]
        FindTemplates {
            source: template::Error,
        },

        #[snafu(display("Input '{}' is empty", path.display()))]
        Input {
            path: PathBuf,
        },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig {
            missing: String,
        },

        #[snafu(display("Cowardly refusing to overwrite parameters without ALLOW_CLOBBER"))]
        NoClobber,

        #[snafu(display("Cowardly refusing to publish private image to public namespace without ALLOW_PRIVATE_IMAGES"))]
        NoPrivateImages,

        #[snafu(display("Failed to render templates: {}", source))]
        RenderTemplates {
            source: template::Error,
        },

        #[snafu(display("Failed to set SSM parameters: {}", source))]
        SetSsm {
            source: ssm::Error,
        },

        #[snafu(display(
            "Given region(s) in Infra.toml / regions argument that are not in --ami-input file: {}",
            regions.join(", ")
        ))]
        UnknownRegions {
            regions: Vec<String>,
        },

        ValidateSsm {
            source: ssm::Error,
        },

        #[snafu(display("Failed to parse rendered SSM parameters to JSON: {}", source))]
        ParseRenderedSsmParameters {
            source: serde_json::Error,
        },

        #[snafu(display("Failed to write rendered SSM parameters to {:#?}: {}", path, source))]
        WriteRenderedSsmParameters {
            path: PathBuf,
            source: std::io::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
