//! The validate_ssm module owns the 'validate-ssm' subcommand and controls the process of
//! validating SSM parameters and AMIs

pub mod results;

use self::results::{SsmValidationResult, SsmValidationResultStatus, SsmValidationResults};
use super::ssm::ssm::get_parameters_by_prefix;
use super::ssm::{SsmKey, SsmParameters};
use crate::aws::client::build_client_config;
use crate::Args;
use aws_sdk_ssm::{Client as SsmClient, Region};
use log::{info, trace};
use pubsys_config::InfraConfig;
use serde::Deserialize;
use snafu::ResultExt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
use structopt::{clap, StructOpt};

/// Validates SSM parameters and AMIs
#[derive(Debug, StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct ValidateSsmArgs {
    /// File holding the validation configuration
    #[structopt(long, parse(from_os_str))]
    validation_config_path: PathBuf,

    /// Optional path where the validation results should be written
    #[structopt(long, parse(from_os_str))]
    write_results_path: Option<PathBuf>,

    #[structopt(long, requires = "write-results-path")]
    /// Optional filter to only write validation results with these statuses to the above path
    /// Available statuses are: `Correct`, `Incorrect`, `Missing`, `Unexpected`
    write_results_filter: Option<Vec<SsmValidationResultStatus>>,

    /// If this flag is added, print the results summary table as JSON instead of a
    /// plaintext table
    #[structopt(long)]
    json: bool,
}

/// Structure of the validation configuration file
#[derive(Debug, Deserialize)]
pub(crate) struct ValidationConfig {
    /// Vec of paths to JSON files containing expected metadata (image ids and SSM parameters)
    expected_metadata_lists: Vec<PathBuf>,

    /// Vec of regions where the parameters should be validated
    validation_regions: Vec<String>,
}

/// A structure that allows us to store a parameter value along with the AMI ID it refers to. In
/// some cases, then AMI ID *is* the parameter value and both fields will hold the AMI ID. In other
/// cases the parameter value is not the AMI ID, but we need to remember which AMI ID it refers to.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SsmValue {
    /// The value of the SSM parameter
    pub(crate) value: String,

    /// The ID of the AMI the parameter is associated with, used for validation result reporting
    pub(crate) ami_id: String,
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

    // Read the validation config file and parse it into the `ValidationConfig` struct
    let validation_config_file = File::open(&validate_ssm_args.validation_config_path).context(
        error::ReadValidationConfigSnafu {
            path: validate_ssm_args.validation_config_path.clone(),
        },
    )?;
    let validation_config: ValidationConfig = serde_json::from_reader(validation_config_file)
        .context(error::ParseValidationConfigSnafu)?;

    let ssm_prefix = aws.ssm_prefix.as_deref().unwrap_or("");

    // Parse the parameter lists found in the validation config
    info!("Parsing expected parameter lists");
    let expected_parameters = parse_parameter_lists(
        validation_config.expected_metadata_lists,
        &validation_config.validation_regions,
    )
    .await?;

    info!("Parsed expected parameter lists");

    // Create a Vec of Regions based on the region names in the validation config
    let validation_regions: Vec<Region> = validation_config
        .validation_regions
        .iter()
        .map(|s| Region::new(s.clone()))
        .collect();

    // Create a HashMap of SsmClients, one for each region where validation should happen
    let base_region = &validation_regions[0];
    let mut ssm_clients = HashMap::with_capacity(validation_regions.len());

    for region in &validation_regions {
        let client_config = build_client_config(region, base_region, &aws).await;
        let ssm_client = SsmClient::new(&client_config);
        ssm_clients.insert(region.clone(), ssm_client);
    }

    // Retrieve the SSM parameters using the SsmClients
    info!("Retrieving SSM parameters");
    let parameters = get_parameters_by_prefix(&ssm_clients, ssm_prefix).await;

    // Validate the retrieved SSM parameters per region
    info!("Validating SSM parameters");
    let results: HashMap<Region, crate::aws::ssm::ssm::Result<HashSet<SsmValidationResult>>> =
        parameters
            .into_iter()
            .map(|(region, region_result)| {
                (
                    region.clone(),
                    region_result.map(|result| {
                        validate_parameters_in_region(
                            expected_parameters.get(region).unwrap_or(&HashMap::new()),
                            &result,
                        )
                    }),
                )
            })
            .collect::<HashMap<Region, crate::aws::ssm::ssm::Result<HashSet<SsmValidationResult>>>>(
            );

    let validation_results = SsmValidationResults::new(results);

    // If a path was given to write the results to, write the results
    if let Some(write_results_path) = &validate_ssm_args.write_results_path {
        // Filter the results by given status, and if no statuses were given, get all results
        info!("Writing results to file");
        let filtered_results = validation_results.get_results_for_status(
            validate_ssm_args
                .write_results_filter
                .as_ref()
                .unwrap_or(&vec![
                    SsmValidationResultStatus::Correct,
                    SsmValidationResultStatus::Incorrect,
                    SsmValidationResultStatus::Missing,
                    SsmValidationResultStatus::Unexpected,
                ]),
        );

        // Write the results as JSON
        serde_json::to_writer_pretty(
            &File::create(write_results_path).context(error::WriteValidationResultsSnafu {
                path: write_results_path,
            })?,
            &filtered_results,
        )
        .context(error::SerializeValidationResultsSnafu)?;
    }

    Ok(validation_results)
}

/// Validates SSM parameters in a single region, based on a HashMap (SsmKey, SsmValue) of expected
/// parameters and a HashMap (SsmKey, String) of actual retrieved parameters. Returns a HashSet of
/// SsmValidationResult objects.
pub(crate) fn validate_parameters_in_region(
    expected_parameters: &HashMap<SsmKey, SsmValue>,
    actual_parameters: &SsmParameters,
) -> HashSet<SsmValidationResult> {
    // Clone the HashMap of actual parameters so items can be removed
    let mut actual_parameters = actual_parameters.clone();
    let mut results = HashSet::new();

    // Validate all expected parameters, creating an SsmValidationResult object and
    // removing the corresponding parameter from `actual_parameters` if found
    for (ssm_key, ssm_value) in expected_parameters {
        results.insert(SsmValidationResult::new(
            ssm_key.name.to_owned(),
            Some(ssm_value.value.clone()),
            actual_parameters.get(ssm_key).map(|v| v.to_owned()),
            ssm_key.region.clone(),
            Some(ssm_value.ami_id.clone()),
        ));
        actual_parameters.remove(ssm_key);
    }

    // Any remaining parameters in `actual_parameters` were not present in `expected_parameters`
    // and therefore get the `Unexpected` status
    for (ssm_key, ssm_value) in actual_parameters {
        results.insert(SsmValidationResult::new(
            ssm_key.name.to_owned(),
            None,
            Some(ssm_value),
            ssm_key.region.clone(),
            None,
        ));
    }
    results
}

type RegionName = String;
type AmiId = String;
type ParameterName = String;
type ParameterValue = String;

/// Parse the lists of parameters whose paths are in `parameter_lists`. Only parse the parameters
/// in the regions present in `validation_regions`. Return a HashMap of Region mapped to a HashMap
/// of the parameters in that region, with each parameter being a mapping of `SsmKey` to `SsmValue`.
pub(crate) async fn parse_parameter_lists(
    parameter_lists: Vec<PathBuf>,
    validation_regions: &[String],
) -> Result<HashMap<Region, HashMap<SsmKey, SsmValue>>> {
    let mut parameter_map: HashMap<Region, HashMap<SsmKey, SsmValue>> = HashMap::new();
    for parameter_list_path in parameter_lists {
        // Parse the JSON list as a HashMap of region_name, mapped to a HashMap of ami_id, mapped to
        // a HashMap of parameter_name and parameter_value
        let parameter_list: HashMap<
            RegionName,
            HashMap<AmiId, HashMap<ParameterName, ParameterValue>>,
        > = serde_json::from_reader(&File::open(parameter_list_path.clone()).context(
            error::ReadExpectedParameterListSnafu {
                path: parameter_list_path,
            },
        )?)
        .context(error::ParseExpectedParameterListSnafu)?;

        // Iterate over the parsed HashMap, converting the nested HashMap into a HashMap of Region
        // mapped to a HashMap of SsmKey, SsmValue
        parameter_list
            .iter()
            .filter(|(region, _)| validation_regions.contains(region))
            .flat_map(|(region, ami_ids)| {
                ami_ids
                    .iter()
                    .map(move |(ami_id, param_names)| (region, ami_id, param_names))
            })
            .flat_map(|(region, ami_id, params)| {
                params.iter().map(move |(parameter_name, parameter_value)| {
                    (
                        region.clone(),
                        ami_id.clone(),
                        parameter_name.clone(),
                        parameter_value.clone(),
                    )
                })
            })
            .for_each(|(region, ami_id, parameter_name, parameter_value)| {
                parameter_map
                    .entry(Region::new(region.clone()))
                    .or_insert(HashMap::new())
                    .insert(
                        SsmKey::new(Region::new(region), parameter_name),
                        SsmValue {
                            value: parameter_value,
                            ami_id,
                        },
                    );
            });
    }
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

mod error {
    use crate::aws::ssm::ssm;
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Error reading config: {}", source))]
        Config { source: pubsys_config::Error },

        #[snafu(display("Error reading validation config at path {}: {}", path.display(), source))]
        ReadValidationConfig {
            source: std::io::Error,
            path: PathBuf,
        },

        #[snafu(display("Error parsing validation config: {}", source))]
        ParseValidationConfig { source: serde_json::Error },

        #[snafu(display("Missing field in validation config: {}", missing))]
        MissingField { missing: String },

        #[snafu(display("Missing region in expected parameters: {}", missing))]
        MissingExpectedRegion { missing: String },

        #[snafu(display("Missing region in actual parameters: {}", missing))]
        MissingActualRegion { missing: String },

        #[snafu(display("Found no parameters in source version {}", version))]
        EmptySource { version: String },

        #[snafu(display("Failed to fetch parameters from SSM: {}", source))]
        FetchSsm { source: ssm::error::Error },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig { missing: String },

        #[snafu(display("Failed to validate SSM parameters: {}", missing))]
        ValidateSsm { missing: String },

        #[snafu(display("Failed to validate SSM parameters in region: {}", region))]
        ValidateSsmRegion { region: String },

        #[snafu(display("Failed to parse AMI list: {}", source))]
        ParseExpectedParameterList { source: serde_json::Error },

        #[snafu(display("Failed to read AMI list: {}", path.display()))]
        ReadExpectedParameterList {
            source: std::io::Error,
            path: PathBuf,
        },

        #[snafu(display("Invalid validation status filter: {}", filter))]
        InvalidStatusFilter { filter: String },

        #[snafu(display("Failed to serialize validation results to json: {}", source))]
        SerializeValidationResults { source: serde_json::Error },

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
        validate_ssm::{results::SsmValidationResult, validate_parameters_in_region, SsmValue},
    };
    use aws_sdk_ssm::Region;
    use std::collections::{HashMap, HashSet};

    // These tests assert that the parameters can be validated correctly.

    // Tests validation of parameters where the expected value is equal to the actual value
    #[test]
    fn validate_parameters_all_correct() {
        let expected_parameters: HashMap<SsmKey, SsmValue> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test1-parameter-value".to_string(),
                    ami_id: "test1-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test2-parameter-value".to_string(),
                    ami_id: "test2-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test3-parameter-value".to_string(),
                    ami_id: "test3-image-id".to_string(),
                },
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
                Some("test3-parameter-value".to_string()),
                Region::new("us-east-1"),
                Some("test3-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Some("test1-parameter-value".to_string()),
                Region::new("us-west-2"),
                Some("test1-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Some("test2-parameter-value".to_string()),
                Region::new("us-west-2"),
                Some("test2-image-id".to_string()),
            ),
        ]);
        let results = validate_parameters_in_region(&expected_parameters, &actual_parameters);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the expected value is different from the actual value
    #[test]
    fn validate_parameters_all_incorrect() {
        let expected_parameters: HashMap<SsmKey, SsmValue> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test1-parameter-value".to_string(),
                    ami_id: "test1-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test2-parameter-value".to_string(),
                    ami_id: "test2-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test3-parameter-value".to_string(),
                    ami_id: "test3-image-id".to_string(),
                },
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
                Some("test3-parameter-value-wrong".to_string()),
                Region::new("us-east-1"),
                Some("test3-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Some("test1-parameter-value-wrong".to_string()),
                Region::new("us-west-2"),
                Some("test1-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Some("test2-parameter-value-wrong".to_string()),
                Region::new("us-west-2"),
                Some("test2-image-id".to_string()),
            ),
        ]);
        let results = validate_parameters_in_region(&expected_parameters, &actual_parameters);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the actual value is missing
    #[test]
    fn validate_parameters_all_missing() {
        let expected_parameters: HashMap<SsmKey, SsmValue> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test1-parameter-value".to_string(),
                    ami_id: "test1-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test2-parameter-value".to_string(),
                    ami_id: "test2-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test3-parameter-value".to_string(),
                    ami_id: "test3-image-id".to_string(),
                },
            ),
        ]);
        let actual_parameters: SsmParameters = HashMap::new();
        let expected_results = HashSet::from_iter(vec![
            SsmValidationResult::new(
                "test3-parameter-name".to_string(),
                Some("test3-parameter-value".to_string()),
                None,
                Region::new("us-east-1"),
                Some("test3-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                None,
                Region::new("us-west-2"),
                Some("test1-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                None,
                Region::new("us-west-2"),
                Some("test2-image-id".to_string()),
            ),
        ]);
        let results = validate_parameters_in_region(&expected_parameters, &actual_parameters);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where the expected value is missing
    #[test]
    fn validate_parameters_all_unexpected() {
        let expected_parameters: HashMap<SsmKey, SsmValue> = HashMap::new();
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
                Some("test3-parameter-value".to_string()),
                Region::new("us-east-1"),
                None,
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                None,
                Some("test1-parameter-value".to_string()),
                Region::new("us-west-2"),
                None,
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                None,
                Some("test2-parameter-value".to_string()),
                Region::new("us-west-2"),
                None,
            ),
        ]);
        let results = validate_parameters_in_region(&expected_parameters, &actual_parameters);

        assert_eq!(results, expected_results);
    }

    // Tests validation of parameters where each status (Correct, Incorrect, Missing, Unexpected)
    // happens once
    #[test]
    fn validate_parameters_mixed() {
        let expected_parameters: HashMap<SsmKey, SsmValue> = HashMap::from([
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test1-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test1-parameter-value".to_string(),
                    ami_id: "test1-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-west-2"),
                    name: "test2-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test2-parameter-value".to_string(),
                    ami_id: "test2-image-id".to_string(),
                },
            ),
            (
                SsmKey {
                    region: Region::new("us-east-1"),
                    name: "test3-parameter-name".to_string(),
                },
                SsmValue {
                    value: "test3-parameter-value".to_string(),
                    ami_id: "test3-image-id".to_string(),
                },
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
                None,
                Region::new("us-east-1"),
                Some("test3-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test1-parameter-name".to_string(),
                Some("test1-parameter-value".to_string()),
                Some("test1-parameter-value".to_string()),
                Region::new("us-west-2"),
                Some("test1-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test2-parameter-name".to_string(),
                Some("test2-parameter-value".to_string()),
                Some("test2-parameter-value-wrong".to_string()),
                Region::new("us-west-2"),
                Some("test2-image-id".to_string()),
            ),
            SsmValidationResult::new(
                "test4-parameter-name".to_string(),
                None,
                Some("test4-parameter-value".to_string()),
                Region::new("us-east-1"),
                None,
            ),
        ]);
        let results = validate_parameters_in_region(&expected_parameters, &actual_parameters);

        assert_eq!(results, expected_results);
    }
}
