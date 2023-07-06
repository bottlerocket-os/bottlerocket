//! The validate_ssm module owns the 'validate-ssm' subcommand and controls the process of
//! validating SSM parameters and AMIs

pub mod results;

use self::results::{SsmValidationResult, SsmValidationResultStatus, SsmValidationResults};
use super::ssm::ssm::get_parameters_by_prefix;
use super::ssm::{SsmKey, SsmParameters};
use crate::aws::client::build_client_config;
use crate::Args;
use aws_sdk_ssm::{Client as SsmClient, Region};
use clap::Parser;
use log::{error, info, trace};
use pubsys_config::InfraConfig;
use snafu::ResultExt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

/// Validates SSM parameters and AMIs
#[derive(Debug, Parser)]
pub struct ValidateSsmArgs {
    /// File holding the expected parameters
    #[arg(long)]
    expected_parameters_path: PathBuf,

    /// If this flag is set, check for unexpected parameters in the validation regions. If not,
    /// only the parameters present in the expected parameters file will be validated.
    #[arg(long)]
    check_unexpected: bool,

    /// Optional path where the validation results should be written
    #[arg(long)]
    write_results_path: Option<PathBuf>,

    /// Optional filter to only write validation results with these statuses to the above path
    /// Available statuses are: `Correct`, `Incorrect`, `Missing`, `Unexpected`
    #[arg(long, requires = "write_results_path")]
    write_results_filter: Option<Vec<SsmValidationResultStatus>>,

    /// If this flag is added, print the results summary table as JSON instead of a
    /// plaintext table
    #[arg(long)]
    json: bool,
}

/// Performs SSM parameter validation and returns the `SsmValidationResults` object
pub async fn validate(
    args: &Args,
    validate_ssm_args: &ValidateSsmArgs,
) -> Result<SsmValidationResults> {
    info!("Parsing Infra.toml file");

    // If a lock file exists, use that, otherwise use Infra.toml
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, false)
        .context(error::ConfigSnafu)?;

    let aws = infra_config.aws.clone().unwrap_or_default();

    trace!("Parsed infra config: {:#?}", infra_config);

    let ssm_prefix = aws.ssm_prefix.as_deref().unwrap_or("");

    // Parse the file holding expected parameters
    info!("Parsing expected parameters file");
    let expected_parameters = parse_parameters(&validate_ssm_args.expected_parameters_path).await?;

    info!("Parsed expected parameters file");

    // Create a HashMap of SsmClients, one for each region where validation should happen
    let base_region = Region::new(aws.regions[0].clone());
    let mut ssm_clients = HashMap::with_capacity(expected_parameters.len());

    for region in expected_parameters.keys() {
        let client_config = build_client_config(region, &base_region, &aws).await;
        let ssm_client = SsmClient::new(&client_config);
        ssm_clients.insert(region.clone(), ssm_client);
    }

    // Retrieve the SSM parameters using the SsmClients
    info!("Retrieving SSM parameters");
    let parameters = get_parameters_by_prefix(&ssm_clients, ssm_prefix)
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

    // Validate the retrieved SSM parameters per region
    info!("Validating SSM parameters");
    let results: HashMap<Region, HashSet<SsmValidationResult>> = parameters
        .into_iter()
        .map(|(region, region_result)| {
            (
                region.clone(),
                validate_parameters_in_region(
                    expected_parameters.get(region).unwrap_or(&HashMap::new()),
                    &region_result,
                    validate_ssm_args.check_unexpected,
                ),
            )
        })
        .collect::<HashMap<Region, HashSet<SsmValidationResult>>>();

    let validation_results = SsmValidationResults::new(results);

    // If a path was given to write the results to, write the results
    if let Some(write_results_path) = &validate_ssm_args.write_results_path {
        // Filter the results by given status, and if no statuses were given, get all results
        info!("Writing results to file");
        let results = if let Some(filter) = &validate_ssm_args.write_results_filter {
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

/// Validates SSM parameters in a single region, based on a HashMap (SsmKey, String) of expected
/// parameters and a HashMap (SsmKey, String) of actual retrieved parameters. Returns a HashSet of
/// SsmValidationResult objects.
pub(crate) fn validate_parameters_in_region(
    expected_parameters: &HashMap<SsmKey, String>,
    actual_parameters: &Result<SsmParameters>,
    check_unexpected: bool,
) -> HashSet<SsmValidationResult> {
    match actual_parameters {
        Ok(actual_parameters) => {
            // Clone the HashMap of actual parameters so items can be removed
            let mut actual_parameters = actual_parameters.clone();
            let mut results = HashSet::new();

            // Validate all expected parameters, creating an SsmValidationResult object and
            // removing the corresponding parameter from `actual_parameters` if found
            for (ssm_key, ssm_value) in expected_parameters {
                results.insert(SsmValidationResult::new(
                    ssm_key.name.to_owned(),
                    Some(ssm_value.clone()),
                    Ok(actual_parameters.get(ssm_key).map(|v| v.to_owned())),
                    ssm_key.region.clone(),
                ));
                actual_parameters.remove(ssm_key);
            }

            if check_unexpected {
                // Any remaining parameters in `actual_parameters` were not present in `expected_parameters`
                // and therefore get the `Unexpected` status
                for (ssm_key, ssm_value) in actual_parameters {
                    results.insert(SsmValidationResult::new(
                        ssm_key.name.to_owned(),
                        None,
                        Ok(Some(ssm_value)),
                        ssm_key.region.clone(),
                    ));
                }
            }
            results
        }
        Err(_) => expected_parameters
            .iter()
            .map(|(ssm_key, ssm_value)| {
                SsmValidationResult::new(
                    ssm_key.name.to_owned(),
                    Some(ssm_value.to_owned()),
                    Err(error::Error::UnreachableRegion {
                        region: ssm_key.region.to_string(),
                    }),
                    ssm_key.region.clone(),
                )
            })
            .collect(),
    }
}

type RegionName = String;
type ParameterName = String;
type ParameterValue = String;

/// Parse the file holding expected parameters. Return a HashMap of Region mapped to a HashMap
/// of the parameters in that region, with each parameter being a mapping of `SsmKey` to its
/// value as `String`.
pub(crate) async fn parse_parameters(
    expected_parameters_file: &PathBuf,
) -> Result<HashMap<Region, HashMap<SsmKey, String>>> {
    // Parse the JSON file as a HashMap of region_name, mapped to a HashMap of parameter_name and
    // parameter_value
    let expected_parameters: HashMap<RegionName, HashMap<ParameterName, ParameterValue>> =
        serde_json::from_reader(&File::open(expected_parameters_file.clone()).context(
            error::ReadExpectedParameterFileSnafu {
                path: expected_parameters_file,
            },
        )?)
        .context(error::ParseExpectedParameterFileSnafu)?;

    // Iterate over the parsed HashMap, converting the nested HashMap into a HashMap of Region
    // mapped to a HashMap of SsmKey, String
    let parameter_map = expected_parameters
        .into_iter()
        .map(|(region, parameters)| {
            (
                Region::new(region.clone()),
                parameters
                    .into_iter()
                    .map(|(parameter_name, parameter_value)| {
                        (
                            SsmKey::new(Region::new(region.clone()), parameter_name),
                            parameter_value,
                        )
                    })
                    .collect::<HashMap<SsmKey, String>>(),
            )
        })
        .collect();

    Ok(parameter_map)
}

/// Common entrypoint from main()
pub(crate) async fn run(args: &Args, validate_ssm_args: &ValidateSsmArgs) -> Result<()> {
    let results = validate(args, validate_ssm_args).await?;

    if validate_ssm_args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&results.get_json_summary())
                .context(error::SerializeResultsSummarySnafu)?
        )
    } else {
        println!("{}", results)
    }
    Ok(())
}

pub(crate) mod error {
    use crate::aws::ssm::ssm;
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Error reading config: {}", source))]
        Config { source: pubsys_config::Error },

        #[snafu(display("Failed to fetch parameters from SSM: {}", source))]
        FetchSsm { source: ssm::error::Error },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig { missing: String },

        #[snafu(display("Failed to validate SSM parameters: {}", missing))]
        ValidateSsm { missing: String },

        #[snafu(display("Failed to parse expected parameters file: {}", source))]
        ParseExpectedParameterFile { source: serde_json::Error },

        #[snafu(display("Failed to read expected parameters file: {}", path.display()))]
        ReadExpectedParameterFile {
            source: std::io::Error,
            path: PathBuf,
        },

        #[snafu(display("Invalid validation status filter: {}", filter))]
        InvalidStatusFilter { filter: String },

        #[snafu(display("Failed to serialize validation results to json: {}", source))]
        SerializeValidationResults { source: serde_json::Error },

        #[snafu(display("Failed to retrieve SSM parameters from region {}", region))]
        UnreachableRegion { region: String },

        #[snafu(display("Failed to write validation results to {}: {}", path.display(), source))]
        WriteValidationResults {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to serialize results summary into JSON: {}", source))]
        SerializeResultsSummary { source: serde_json::Error },
    }
}

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use crate::aws::{
        ssm::{SsmKey, SsmParameters},
        validate_ssm::{results::SsmValidationResult, validate_parameters_in_region},
    };
    use aws_sdk_ssm::Region;
    use std::collections::{HashMap, HashSet};

    // These tests assert that the parameters can be validated correctly.

    // Tests validation of parameters where the expected value is equal to the actual value
    #[test]
    fn validate_parameters_all_correct() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let actual_parameters: SsmParameters = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                Ok(Some("test3-parameter-value".to_string())),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Ok(Some("test1-parameter-value".to_string())),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Ok(Some("test2-parameter-value".to_string())),
                Region::new("us-west-2"),
            ),
        ]);
        let results =
            validate_parameters_in_region(&expected_parameters, &Ok(actual_parameters), true);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the expected value is different from the actual value
    #[test]
    fn validate_parameters_all_incorrect() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let actual_parameters: SsmParameters = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value-wrong".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value-wrong".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value-wrong".to_string(),
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                Ok(Some("test3-parameter-value-wrong".to_string())),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Ok(Some("test1-parameter-value-wrong".to_string())),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Ok(Some("test2-parameter-value-wrong".to_string())),
                Region::new("us-west-2"),
            ),
        ]);
        let results =
            validate_parameters_in_region(&expected_parameters, &Ok(actual_parameters), true);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the actual value is missing
    #[test]
    fn validate_parameters_all_missing() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let actual_parameters: SsmParameters = HashMap::new();
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                Ok(None),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Ok(None),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Ok(None),
                Region::new("us-west-2"),
            ),
        ]);
        let results =
            validate_parameters_in_region(&expected_parameters, &Ok(actual_parameters), true);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the expected value is missing
    #[test]
    fn validate_parameters_all_unexpected() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::new();
        let actual_parameters: SsmParameters = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                None,
                Ok(Some("test3-parameter-value".to_string())),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                None,
                Ok(Some("test1-parameter-value".to_string())),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                None,
                Ok(Some("test2-parameter-value".to_string())),
                Region::new("us-west-2"),
            ),
        ]);
        let results =
            validate_parameters_in_region(&expected_parameters, &Ok(actual_parameters), true);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where each status (Correct, Incorrect, Missing, Unexpected)
    // happens once
    #[test]
    fn validate_parameters_mixed() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let actual_parameters: SsmParameters = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value-wrong".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test4-parameter-name".to_string(),
                },
                "test4-parameter-value".to_string(),
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                Ok(None),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Ok(Some("test1-parameter-value".to_string())),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Ok(Some("test2-parameter-value-wrong".to_string())),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test4-parameter-name".to_string(),
                None,
                Ok(Some("test4-parameter-value".to_string())),
                Region::new("us-east-1"),
            ),
        ]);
        let results =
            validate_parameters_in_region(&expected_parameters, &Ok(actual_parameters), true);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where each reachable status (Correct, Incorrect, Missing, Unexpected)
    // happens once and `--check-unexpected` is false
    #[test]
    fn validate_parameters_mixed_unexpected_false() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let actual_parameters: SsmParameters = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value-wrong".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test4-parameter-name".to_string(),
                },
                "test4-parameter-value".to_string(),
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                Ok(None),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Ok(Some("test1-parameter-value".to_string())),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Ok(Some("test2-parameter-value-wrong".to_string())),
                Region::new("us-west-2"),
            ),
        ]);
        let results =
            validate_parameters_in_region(&expected_parameters, &Ok(actual_parameters), false);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the status is Unreachable
    #[test]
    fn validate_parameters_unreachable() {
        let expected_parameters: HashMap<SsmKey, String> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                "test1-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                "test2-parameter-value".to_string(),
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                "test3-parameter-value".to_string(),
            ),
        ]);
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                Err(crate::aws::validate_ssm::Error::UnreachableRegion {
                    region: "us-west-2".to_string(),
                }),
                Region::new("us-east-1"),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Err(crate::aws::validate_ssm::Error::UnreachableRegion {
                    region: "us-west-2".to_string(),
                }),
                Region::new("us-west-2"),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Err(crate::aws::validate_ssm::Error::UnreachableRegion {
                    region: "us-west-2".to_string(),
                }),
                Region::new("us-west-2"),
            ),
        ]);
        let results = validate_parameters_in_region(
            &expected_parameters,
            &Err(crate::aws::validate_ssm::Error::UnreachableRegion {
                region: "us-west-2".to_string(),
            }),
            false,
        );

        assert_eq!(results, expected_results);
    }
}
