//! The promote_ssm module owns the 'promote-ssm' subcommand and controls the process of copying
//! SSM parameters from one version to another

use crate::aws::client::build_client;
use crate::aws::ssm::{key_difference, ssm, template, BuildContext, SsmKey};
use crate::aws::{parse_arch, region_from_string};
use crate::Args;
use log::{info, trace};
use pubsys_config::InfraConfig;
use rusoto_core::Region;
use rusoto_ssm::SsmClient;
use snafu::{ensure, ResultExt};
use std::collections::HashMap;
use std::path::PathBuf;
use structopt::StructOpt;

/// Copies sets of SSM parameters
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub(crate) struct PromoteArgs {
    /// The architecture of the machine image
    #[structopt(long, parse(try_from_str = parse_arch))]
    arch: String,

    /// The variant name for the current build
    #[structopt(long)]
    variant: String,

    /// Version number (or string) to copy from
    #[structopt(long)]
    source: String,

    /// Version number (or string) to copy to
    #[structopt(long)]
    target: String,

    /// Comma-separated list of regions to promote in, overriding Infra.toml
    #[structopt(long, use_delimiter = true)]
    regions: Vec<String>,

    /// File holding the parameter templates
    #[structopt(long)]
    template_path: PathBuf,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, promote_args: &PromoteArgs) -> Result<()> {
    info!(
        "Promoting SSM parameters from {} to {}",
        promote_args.source, promote_args.target
    );

    // Setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config =
        InfraConfig::from_path_or_lock(&args.infra_config_path, false).context(error::Config)?;

    trace!("Parsed infra config: {:#?}", infra_config);
    let aws = infra_config.aws.unwrap_or_else(Default::default);
    let ssm_prefix = aws.ssm_prefix.as_deref().unwrap_or_else(|| "");

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let regions = if !promote_args.regions.is_empty() {
        promote_args.regions.clone()
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
    let base_region = &regions[0];

    let mut ssm_clients = HashMap::with_capacity(regions.len());
    for region in &regions {
        let ssm_client =
            build_client::<SsmClient>(region, &base_region, &aws).context(error::Client {
                client_type: "SSM",
                region: region.name(),
            })?;
        ssm_clients.insert(region.clone(), ssm_client);
    }

    // Template setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Non-image-specific context for building and rendering templates
    let source_build_context = BuildContext {
        variant: &promote_args.variant,
        arch: &promote_args.arch,
        image_version: &promote_args.source,
    };

    let target_build_context = BuildContext {
        variant: &promote_args.variant,
        arch: &promote_args.arch,
        image_version: &promote_args.target,
    };

    info!(
        "Parsing SSM parameter templates from {}",
        promote_args.template_path.display()
    );
    // Doesn't matter which build context we use to find template files because version isn't used
    // in their naming
    let template_parameters =
        template::get_parameters(&promote_args.template_path, &source_build_context)
            .context(error::FindTemplates)?;

    if template_parameters.parameters.is_empty() {
        info!(
            "No parameters for this arch/variant in {}",
            promote_args.template_path.display()
        );
        return Ok(());
    }

    // Render parameter names into maps of {template string => rendered value}.  We need the
    // template strings so we can associate source parameters with target parameters that came
    // from the same template, so we know what to copy.
    let source_parameter_map =
        template::render_parameter_names(&template_parameters, ssm_prefix, &source_build_context)
            .context(error::RenderTemplates)?;
    let target_parameter_map =
        template::render_parameter_names(&template_parameters, ssm_prefix, &target_build_context)
            .context(error::RenderTemplates)?;

    // Parameters are the same in each region, so we need to associate each region with each of
    // the parameter names so we can fetch them.
    let source_keys: Vec<SsmKey> = regions
        .iter()
        .flat_map(|region| {
            source_parameter_map
                .values()
                .map(move |name| SsmKey::new(region.clone(), name.clone()))
        })
        .collect();
    let target_keys: Vec<SsmKey> = regions
        .iter()
        .flat_map(|region| {
            target_parameter_map
                .values()
                .map(move |name| SsmKey::new(region.clone(), name.clone()))
        })
        .collect();

    // SSM get/compare   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Getting current SSM parameters for source and target names");
    let current_source_parameters = ssm::get_parameters(&source_keys, &ssm_clients)
        .await
        .context(error::FetchSsm)?;
    trace!(
        "Current source SSM parameters: {:#?}",
        current_source_parameters
    );
    ensure!(
        !current_source_parameters.is_empty(),
        error::EmptySource {
            version: &promote_args.source
        }
    );

    let current_target_parameters = ssm::get_parameters(&target_keys, &ssm_clients)
        .await
        .context(error::FetchSsm)?;
    trace!(
        "Current target SSM parameters: {:#?}",
        current_target_parameters
    );

    // Build a map of rendered source parameter names to rendered target parameter names.  This
    // will let us find which target parameters to set based on the source parameter names we get
    // back from SSM.
    let source_target_map: HashMap<&String, &String> = source_parameter_map
        .iter()
        .map(|(k, v)| (v, &target_parameter_map[k]))
        .collect();

    // Show the difference between source and target parameters in SSM.  We use the
    // source_target_map we built above to map source keys to target keys (generated from the same
    // template) so that the diff code has common keys to compare.
    let set_parameters = key_difference(
        &current_source_parameters
            .into_iter()
            .map(|(key, value)| {
                (
                    SsmKey::new(key.region, source_target_map[&key.name].to_string()),
                    value,
                )
            })
            .collect(),
        &current_target_parameters,
    );
    if set_parameters.is_empty() {
        info!("No changes necessary.");
        return Ok(());
    }

    // SSM set   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Setting updated SSM parameters.");
    ssm::set_parameters(&set_parameters, &ssm_clients)
        .await
        .context(error::SetSsm)?;

    info!("Validating whether live parameters in SSM reflect changes.");
    ssm::validate_parameters(&set_parameters, &ssm_clients)
        .await
        .context(error::ValidateSsm)?;

    info!("All parameters match requested values.");
    Ok(())
}

mod error {
    use crate::aws;
    use crate::aws::ssm::{ssm, template};
    use snafu::Snafu;

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

        #[snafu(display("Found no parameters in source version {}", version))]
        EmptySource {
            version: String,
        },

        #[snafu(display("Failed to fetch parameters from SSM: {}", source))]
        FetchSsm {
            source: ssm::Error,
        },

        #[snafu(display("Failed to find templates: {}", source))]
        FindTemplates {
            source: template::Error,
        },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig {
            missing: String,
        },

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

        ValidateSsm {
            source: ssm::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
