//! The ssm module owns the 'ssm' subcommand and controls the process of setting SSM parameters
//! based on current build information

pub(crate) mod ssm;
pub(crate) mod template;

use crate::aws::{ami::Image, client::build_client, parse_arch, region_from_string};
use crate::Args;
use log::{info, trace};
use pubsys_config::{AwsConfig, InfraConfig};
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::StructOpt;

/// Sets SSM parameters based on current build information
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub(crate) struct SsmArgs {
    // This is JSON output from `pubsys ami` like `{"us-west-2": "ami-123"}`
    /// Path to the JSON file containing regional AMI IDs to modify
    #[structopt(long, parse(from_os_str))]
    ami_input: PathBuf,

    /// The architecture of the machine image
    #[structopt(long, parse(try_from_str = parse_arch))]
    arch: String,

    /// The variant name for the current build
    #[structopt(long)]
    variant: String,

    /// The version of the current build
    #[structopt(long)]
    version: String,

    /// Regions where you want parameters published
    #[structopt(long, use_delimiter = true)]
    regions: Vec<String>,

    /// File holding the parameter templates
    #[structopt(long)]
    template_path: PathBuf,

    /// Allows overwrite of existing parameters
    #[structopt(long)]
    allow_clobber: bool,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, ssm_args: &SsmArgs) -> Result<()> {
    // Setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config =
        InfraConfig::from_path_or_lock(&args.infra_config_path, false).context(error::Config)?;
    trace!("Parsed infra config: {:#?}", infra_config);
    let aws = infra_config.aws.unwrap_or_else(Default::default);
    let ssm_prefix = aws.ssm_prefix.as_deref().unwrap_or_else(|| "");

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let regions = if !ssm_args.regions.is_empty() {
        ssm_args.regions.clone()
    } else {
        aws.regions.clone().into()
    };
    ensure!(
        !regions.is_empty(),
        error::MissingConfig {
            missing: "aws.regions"
        }
    );
    let base_region = region_from_string(&regions[0], &aws).context(error::ParseRegion)?;

    let amis = parse_ami_input(&regions, &ssm_args, &aws)?;

    let mut ssm_clients = HashMap::with_capacity(amis.len());
    for region in amis.keys() {
        let ssm_client =
            build_client::<SsmClient>(&region, &base_region, &aws).context(error::Client {
                client_type: "SSM",
                region: region.name(),
            })?;
        ssm_clients.insert(region.clone(), ssm_client);
    }

    // Template setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Non-image-specific context for building and rendering templates
    let build_context = BuildContext {
        variant: &ssm_args.variant,
        arch: &ssm_args.arch,
        image_version: &ssm_args.version,
    };

    info!(
        "Parsing SSM parameter templates from {}",
        ssm_args.template_path.display()
    );
    let template_parameters = template::get_parameters(&ssm_args.template_path, &build_context)
        .context(error::FindTemplates)?;

    if template_parameters.parameters.is_empty() {
        info!(
            "No parameters for this arch/variant in {}",
            ssm_args.template_path.display()
        );
        return Ok(());
    }

    let new_parameters =
        template::render_parameters(template_parameters, amis, ssm_prefix, &build_context)
            .context(error::RenderTemplates)?;
    trace!("Generated templated parameters: {:#?}", new_parameters);

    // SSM get/compare   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Getting current SSM parameters");
    let new_parameter_names: Vec<&SsmKey> = new_parameters.keys().collect();
    let current_parameters = ssm::get_parameters(&new_parameter_names, &ssm_clients)
        .await
        .context(error::FetchSsm)?;
    trace!("Current SSM parameters: {:#?}", current_parameters);

    // Show the difference between source and target parameters in SSM.
    let parameters_to_set = key_difference(&new_parameters, &current_parameters);
    if parameters_to_set.is_empty() {
        info!("No changes necessary.");
        return Ok(());
    }

    // Unless the user wants to allow it, make sure we're not going to overwrite any existing
    // keys.
    if !ssm_args.allow_clobber {
        let current_keys: HashSet<&SsmKey> = current_parameters.keys().collect();
        let new_keys: HashSet<&SsmKey> = parameters_to_set.keys().collect();
        ensure!(current_keys.is_disjoint(&new_keys), error::NoClobber);
    }

    // SSM set   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Setting updated SSM parameters.");
    ssm::set_parameters(&parameters_to_set, &ssm_clients)
        .await
        .context(error::SetSsm)?;

    info!("Validating whether live parameters in SSM reflect changes.");
    ssm::validate_parameters(&parameters_to_set, &ssm_clients)
        .await
        .context(error::ValidateSsm)?;

    info!("All parameters match requested values.");
    Ok(())
}

/// The key to a unique SSM parameter
#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) struct SsmKey {
    pub(crate) region: Region,
    pub(crate) name: String,
}

impl SsmKey {
    pub(crate) fn new(region: Region, name: String) -> Self {
        Self { region, name }
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
type SsmParameters = HashMap<SsmKey, String>;

/// Parse the AMI input file
fn parse_ami_input(
    regions: &[String],
    ssm_args: &SsmArgs,
    aws: &AwsConfig,
) -> Result<HashMap<Region, Image>> {
    info!("Using AMI data from path: {}", ssm_args.ami_input.display());
    let file = File::open(&ssm_args.ami_input).context(error::File {
        op: "open",
        path: &ssm_args.ami_input,
    })?;
    let mut ami_input: HashMap<String, Image> =
        serde_json::from_reader(file).context(error::Deserialize {
            path: &ssm_args.ami_input,
        })?;
    trace!("Parsed AMI input: {:#?}", ami_input);

    // pubsys will not create a file if it did not create AMIs, so we should only have an empty
    // file if a user created one manually, and they shouldn't be creating an empty file.
    ensure!(
        !ami_input.is_empty(),
        error::Input {
            path: &ssm_args.ami_input
        }
    );

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
        let image = ami_input
            .remove(name)
            // This could only happen if someone removes the check above...
            .with_context(|| error::UnknownRegions {
                regions: vec![name.clone()],
            })?;
        let region = region_from_string(&name, &aws).context(error::ParseRegion)?;
        amis.insert(region, image);
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
            key.name,
            key.region.name(),
            new_value,
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
            println!("{} - {} - no change", key.name, key.region.name());
        } else {
            println!(
                "{} - {} - changing value:\n   old value: {}\n   new value: {}",
                key.name,
                key.region.name(),
                current_value,
                new_value
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
    use crate::aws;
    use crate::aws::ssm::{ssm, template};
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
            source: pubsys_config::Error,
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

        ParseRegion {
            source: crate::aws::Error,
        },

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
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
