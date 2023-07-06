//! The promote_ssm module owns the 'promote-ssm' subcommand and controls the process of copying
//! SSM parameters from one version to another

use crate::aws::client::build_client_config;
use crate::aws::ssm::template::RenderedParametersMap;
use crate::aws::ssm::{key_difference, ssm, template, BuildContext, SsmKey};
use crate::aws::validate_ssm::parse_parameters;
use crate::aws::{parse_arch, region_from_string};
use crate::Args;
use aws_sdk_ec2::model::ArchitectureValues;
use aws_sdk_ssm::{Client as SsmClient, Region};
use clap::Parser;
use log::{info, trace};
use pubsys_config::InfraConfig;
use snafu::{ensure, ResultExt};
use std::collections::HashMap;
use std::path::PathBuf;

/// Copies sets of SSM parameters
#[derive(Debug, Parser)]
pub(crate) struct PromoteArgs {
    /// The architecture of the machine image
    #[arg(long, value_parser = parse_arch)]
    arch: ArchitectureValues,

    /// The variant name for the current build
    #[arg(long)]
    variant: String,

    /// Version number (or string) to copy from
    #[arg(long)]
    source: String,

    /// Version number (or string) to copy to
    #[arg(long)]
    target: String,

    /// Comma-separated list of regions to promote in, overriding Infra.toml
    #[arg(long, value_delimiter = ',')]
    regions: Vec<String>,

    /// File holding the parameter templates
    #[arg(long)]
    template_path: PathBuf,

    /// If set, contains the path to the file holding the original SSM parameters
    /// and where the newly promoted parameters will be written
    #[arg(long)]
    ssm_parameter_output: Option<PathBuf>,
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, promote_args: &PromoteArgs) -> Result<()> {
    info!(
        "Promoting SSM parameters from {} to {}",
        promote_args.source, promote_args.target
    );

    // Setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, false)
        .context(error::ConfigSnafu)?;

    trace!("Parsed infra config: {:#?}", infra_config);
    let aws = infra_config.aws.unwrap_or_default();
    let ssm_prefix = aws.ssm_prefix.as_deref().unwrap_or("");

    // If the user gave an override list of regions, use that, otherwise use what's in the config.
    let regions = if !promote_args.regions.is_empty() {
        promote_args.regions.clone()
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
    let base_region = &regions[0];

    let mut ssm_clients = HashMap::with_capacity(regions.len());
    for region in &regions {
        let client_config = build_client_config(region, base_region, &aws).await;
        let ssm_client = SsmClient::new(&client_config);
        ssm_clients.insert(region.clone(), ssm_client);
    }

    // Template setup   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    // Non-image-specific context for building and rendering templates
    let source_build_context = BuildContext {
        variant: &promote_args.variant,
        arch: promote_args.arch.as_str(),
        image_version: &promote_args.source,
    };

    let target_build_context = BuildContext {
        variant: &promote_args.variant,
        arch: promote_args.arch.as_str(),
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
            .context(error::FindTemplatesSnafu)?;

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
            .context(error::RenderTemplatesSnafu)?;
    let target_parameter_map =
        template::render_parameter_names(&template_parameters, ssm_prefix, &target_build_context)
            .context(error::RenderTemplatesSnafu)?;

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
        .context(error::FetchSsmSnafu)?;
    trace!(
        "Current source SSM parameters: {:#?}",
        current_source_parameters
    );
    ensure!(
        !current_source_parameters.is_empty(),
        error::EmptySourceSnafu {
            version: &promote_args.source
        }
    );

    let current_target_parameters = ssm::get_parameters(&target_keys, &ssm_clients)
        .await
        .context(error::FetchSsmSnafu)?;
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

    // If an output file path was given, read the existing parameters in `ssm_parameter_output` and
    // write the newly promoted parameters to `ssm_parameter_output` along with the original
    // parameters
    if let Some(ssm_parameter_output) = &promote_args.ssm_parameter_output {
        append_rendered_parameters(ssm_parameter_output, &set_parameters).await?;
    }

    // SSM set   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    info!("Setting updated SSM parameters.");
    ssm::set_parameters(&set_parameters, &ssm_clients)
        .await
        .context(error::SetSsmSnafu)?;

    info!("Validating whether live parameters in SSM reflect changes.");
    ssm::validate_parameters(&set_parameters, &ssm_clients)
        .await
        .context(error::ValidateSsmSnafu)?;

    info!("All parameters match requested values.");
    Ok(())
}

/// Read parameters in given file, add newly promoted parameters, and write combined parameters to
/// the given file
async fn append_rendered_parameters(
    ssm_parameters_output: &PathBuf,
    set_parameters: &HashMap<SsmKey, String>,
) -> Result<()> {
    // If the file doesn't exist, assume that there are no existing parameters
    let parsed_parameters = parse_parameters(&ssm_parameters_output.to_owned())
        .await
        .or_else({
            |e| match e {
                crate::aws::validate_ssm::Error::ReadExpectedParameterFile { .. } => {
                    Ok(HashMap::new())
                }
                _ => Err(e),
            }
        })
        .context(error::ParseExistingSsmParametersSnafu {
            path: ssm_parameters_output,
        })?
        // SsmKey contains region information, so we can lose the top-level region.
        .into_values()
        .fold(HashMap::new(), |mut acc, params| {
            acc.extend(params);
            acc
        });

    let combined_parameters = merge_parameters(parsed_parameters, set_parameters);

    write_rendered_parameters(
        ssm_parameters_output,
        &RenderedParametersMap::from(combined_parameters).rendered_parameters,
    )
    .context(error::WriteRenderedSsmParametersSnafu {
        path: ssm_parameters_output,
    })?;

    Ok(())
}

/// Return a HashMap of Region mapped to a HashMap of SsmKey, String pairs, representing the newly
/// promoted parameters as well as the original parameters. In case of a parameter collision,
/// the parameter takes the promoted value.
fn merge_parameters(
    source_parameters: HashMap<SsmKey, String>,
    set_parameters: &HashMap<SsmKey, String>,
) -> HashMap<Region, HashMap<SsmKey, String>> {
    let mut combined_parameters = HashMap::new();

    source_parameters
        .into_iter()
        // Process the `set_parameters` second so that they overwrite existing values.
        .chain(set_parameters.clone().into_iter())
        .for_each(|(ssm_key, ssm_value)| {
            combined_parameters
                // The `entry()` API demands that we clone
                .entry(ssm_key.region.clone())
                .or_insert(HashMap::new())
                .insert(ssm_key, ssm_value);
        });

    combined_parameters
}

mod error {
    use std::path::PathBuf;

    use crate::aws::{
        ssm::{ssm, template},
        validate_ssm,
    };
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
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

        #[snafu(display(
            "Failed to parse existing SSM parameters at path {:?}: {}",
            path,
            source,
        ))]
        ParseExistingSsmParameters {
            source: validate_ssm::error::Error,
            path: PathBuf,
        },

        #[snafu(display("Failed to parse rendered SSM parameters to JSON: {}", source))]
        ParseRenderedSsmParameters {
            source: serde_json::Error,
        },

        #[snafu(display("Failed to write rendered SSM parameters to {}: {}", path.display(), source))]
        WriteRenderedSsmParameters {
            path: PathBuf,
            source: crate::aws::ssm::Error,
        },
    }
}
pub(crate) use error::Error;

use super::ssm::write_rendered_parameters;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::aws::{promote_ssm::merge_parameters, ssm::SsmKey};
    use aws_sdk_ssm::Region;

    #[test]
    fn combined_parameters() {
        let existing_parameters = HashMap::from([
            (
                SsmKey::new(Region::new("us-west-2"), "test1-parameter-name".to_string()),
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey::new(Region::new("us-west-2"), "test2-parameter-name".to_string()),
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey::new(Region::new("us-east-1"), "test3-parameter-name".to_string()),
                "test3-parameter-value".to_string(),
            ),
            (
                SsmKey::new(
                    Region::new("us-east-1"),
                    "test4-unpromoted-parameter-name".to_string(),
                ),
                "test4-unpromoted-parameter-value".to_string(),
            ),
        ]);
        let set_parameters = HashMap::from([
            (
                SsmKey::new(
                    Region::new("us-west-2"),
                    "test1-parameter-name-promoted".to_string(),
                ),
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey::new(
                    Region::new("us-west-2"),
                    "test2-parameter-name-promoted".to_string(),
                ),
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey::new(
                    Region::new("us-east-1"),
                    "test3-parameter-name-promoted".to_string(),
                ),
                "test3-parameter-value".to_string(),
            ),
        ]);
        let map = merge_parameters(existing_parameters, &set_parameters);
        let expected_map = HashMap::from([
            (
                Region::new("us-west-2"),
                HashMap::from([
                    (
                        SsmKey::new(Region::new("us-west-2"), "test1-parameter-name".to_string()),
                        "test1-parameter-value".to_string(),
                    ),
                    (
                        SsmKey::new(Region::new("us-west-2"), "test2-parameter-name".to_string()),
                        "test2-parameter-value".to_string(),
                    ),
                    (
                        SsmKey::new(
                            Region::new("us-west-2"),
                            "test1-parameter-name-promoted".to_string(),
                        ),
                        "test1-parameter-value".to_string(),
                    ),
                    (
                        SsmKey::new(
                            Region::new("us-west-2"),
                            "test2-parameter-name-promoted".to_string(),
                        ),
                        "test2-parameter-value".to_string(),
                    ),
                ]),
            ),
            (
                Region::new("us-east-1"),
                HashMap::from([
                    (
                        SsmKey::new(Region::new("us-east-1"), "test3-parameter-name".to_string()),
                        "test3-parameter-value".to_string(),
                    ),
                    (
                        SsmKey::new(
                            Region::new("us-east-1"),
                            "test3-parameter-name-promoted".to_string(),
                        ),
                        "test3-parameter-value".to_string(),
                    ),
                    (
                        SsmKey::new(
                            Region::new("us-east-1"),
                            "test4-unpromoted-parameter-name".to_string(),
                        ),
                        "test4-unpromoted-parameter-value".to_string(),
                    ),
                ]),
            ),
        ]);
        assert_eq!(map, expected_map);
    }

    #[test]
    fn combined_parameters_overwrite() {
        let existing_parameters = HashMap::from([
            (
                SsmKey::new(Region::new("us-west-2"), "test1-parameter-name".to_string()),
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey::new(Region::new("us-west-2"), "test2-parameter-name".to_string()),
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey::new(Region::new("us-east-1"), "test3-parameter-name".to_string()),
                "test3-parameter-value".to_string(),
            ),
        ]);
        let set_parameters = HashMap::from([
            (
                SsmKey::new(Region::new("us-west-2"), "test1-parameter-name".to_string()),
                "test1-parameter-value-new".to_string(),
            ),
            (
                SsmKey::new(Region::new("us-west-2"), "test2-parameter-name".to_string()),
                "test2-parameter-value-new".to_string(),
            ),
            (
                SsmKey::new(
                    Region::new("us-east-1"),
                    "test3-parameter-name-promoted".to_string(),
                ),
                "test3-parameter-value".to_string(),
            ),
        ]);
        let map = merge_parameters(existing_parameters, &set_parameters);
        let expected_map = HashMap::from([
            (
                Region::new("us-west-2"),
                HashMap::from([
                    (
                        SsmKey::new(Region::new("us-west-2"), "test1-parameter-name".to_string()),
                        "test1-parameter-value-new".to_string(),
                    ),
                    (
                        SsmKey::new(Region::new("us-west-2"), "test2-parameter-name".to_string()),
                        "test2-parameter-value-new".to_string(),
                    ),
                ]),
            ),
            (
                Region::new("us-east-1"),
                HashMap::from([
                    (
                        SsmKey::new(Region::new("us-east-1"), "test3-parameter-name".to_string()),
                        "test3-parameter-value".to_string(),
                    ),
                    (
                        SsmKey::new(
                            Region::new("us-east-1"),
                            "test3-parameter-name-promoted".to_string(),
                        ),
                        "test3-parameter-value".to_string(),
                    ),
                ]),
            ),
        ]);
        assert_eq!(map, expected_map);
    }
}
