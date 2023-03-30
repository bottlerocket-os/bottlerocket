//! The results module owns the reporting of SSM validation results.

use crate::aws::ssm::ssm::Result;
use aws_sdk_ssm::Region;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::str::FromStr;
use tabled::{Table, Tabled};

/// Represent the possible status of an SSM validation
#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum SsmValidationResultStatus {
    /// The expected value was equal to the actual value
    Correct,

    /// The expected value was different from the actual value
    Incorrect,

    /// The parameter was expected but not included in the actual parameters
    Missing,

    /// The parameter was present in the actual parameters but not expected
    Unexpected,
}

impl Display for SsmValidationResultStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Correct => write!(f, "Correct"),
            Self::Incorrect => write!(f, "Incorrect"),
            Self::Missing => write!(f, "Missing"),
            Self::Unexpected => write!(f, "Unexpected"),
        }
    }
}

impl FromStr for SsmValidationResultStatus {
    type Err = super::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Correct" => Ok(Self::Correct),
            "Incorrect" => Ok(Self::Incorrect),
            "Missing" => Ok(Self::Missing),
            "Unexpected" => Ok(Self::Unexpected),
            filter => Err(Self::Err::InvalidStatusFilter {
                filter: filter.to_string(),
            }),
        }
    }
}

/// Represents a single SSM validation result
#[derive(Debug, Eq, Hash, PartialEq, Tabled, Serialize)]
pub struct SsmValidationResult {
    /// The name of the parameter
    pub(crate) name: String,

    /// The expected value of the parameter
    #[tabled(display_with = "display_option")]
    pub(crate) expected_value: Option<String>,

    /// The actual retrieved value of the parameter
    #[tabled(display_with = "display_option")]
    pub(crate) actual_value: Option<String>,

    /// The region the parameter resides in
    #[serde(serialize_with = "serialize_region")]
    pub(crate) region: Region,

    /// The ID of the AMI the parameter is associated with
    #[tabled(display_with = "display_option")]
    pub(crate) ami_id: Option<String>,

    /// The validation status of the parameter
    pub(crate) status: SsmValidationResultStatus,
}

fn display_option(option: &Option<String>) -> &str {
    match option {
        Some(option) => option,
        None => "N/A",
    }
}

fn serialize_region<S>(region: &Region, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(region.to_string().as_str())
}

impl SsmValidationResult {
    pub(crate) fn new(
        name: String,
        expected_value: Option<String>,
        actual_value: Option<String>,
        region: Region,
        ami_id: Option<String>,
    ) -> SsmValidationResult {
        // Determine the validation status based on equality, presence, and absence of expected and
        // actual parameter values
        let status = match (&expected_value, &actual_value) {
            (Some(expected_value), Some(actual_value)) if actual_value.eq(expected_value) => {
                SsmValidationResultStatus::Correct
            }
            (Some(_), Some(_)) => SsmValidationResultStatus::Incorrect,
            (_, None) => SsmValidationResultStatus::Missing,
            (None, _) => SsmValidationResultStatus::Unexpected,
        };
        SsmValidationResult {
            name,
            expected_value,
            actual_value,
            region,
            ami_id,
            status,
        }
    }
}

#[derive(Tabled, Serialize)]
struct SsmValidationRegionSummary {
    correct: i32,
    incorrect: i32,
    missing: i32,
    unexpected: i32,
    accessible: bool,
}

impl From<&HashSet<SsmValidationResult>> for SsmValidationRegionSummary {
    fn from(results: &HashSet<SsmValidationResult>) -> Self {
        let mut region_validation = SsmValidationRegionSummary {
            correct: 0,
            incorrect: 0,
            missing: 0,
            unexpected: 0,
            accessible: true,
        };
        for validation_result in results {
            match validation_result.status {
                SsmValidationResultStatus::Correct => region_validation.correct += 1,
                SsmValidationResultStatus::Incorrect => region_validation.incorrect += 1,
                SsmValidationResultStatus::Missing => region_validation.missing += 1,
                SsmValidationResultStatus::Unexpected => region_validation.unexpected += 1,
            }
        }
        region_validation
    }
}

impl SsmValidationRegionSummary {
    fn no_valid_results() -> Self {
        // When the parameters in a region couldn't be retrieved, use `-1` to indicate this in the
        // output table and set `accessible` to `false`
        SsmValidationRegionSummary {
            correct: -1,
            incorrect: -1,
            missing: -1,
            unexpected: -1,
            accessible: false,
        }
    }
}

/// Represents all SSM validation results
#[derive(Debug)]
pub struct SsmValidationResults {
    pub(crate) results: HashMap<Region, Result<HashSet<SsmValidationResult>>>,
}

impl Default for SsmValidationResults {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl Display for SsmValidationResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create a summary for each region, counting the number of parameters per status
        let region_validations: HashMap<Region, SsmValidationRegionSummary> =
            self.get_results_summary();

        // Represent the HashMap of summaries as a `Table`
        let table = Table::new(
            region_validations
                .iter()
                .map(|(region, results)| (region.to_string(), results))
                .collect::<Vec<(String, &SsmValidationRegionSummary)>>(),
        )
        .to_string();
        write!(f, "{}", table)
    }
}

impl SsmValidationResults {
    pub fn new(results: HashMap<Region, Result<HashSet<SsmValidationResult>>>) -> Self {
        SsmValidationResults { results }
    }

    /// Returns a HashSet containing all validation results whose status is present in
    /// `requested_status`
    pub fn get_results_for_status(
        &self,
        requested_status: &[SsmValidationResultStatus],
    ) -> HashSet<&SsmValidationResult> {
        let mut results = HashSet::new();
        for region_results in self.results.values().flatten() {
            results.extend(
                region_results
                    .iter()
                    .filter(|result| requested_status.contains(&result.status))
                    .collect::<HashSet<&SsmValidationResult>>(),
            )
        }
        results
    }

    fn get_results_summary(&self) -> HashMap<Region, SsmValidationRegionSummary> {
        self.results
            .iter()
            .map(|(region, region_result)| {
                region_result
                    .as_ref()
                    .map(|region_validation| {
                        (
                            region.clone(),
                            SsmValidationRegionSummary::from(region_validation),
                        )
                    })
                    .unwrap_or((
                        region.clone(),
                        SsmValidationRegionSummary::no_valid_results(),
                    ))
            })
            .collect()
    }

    pub(crate) fn get_json_summary(&self) -> serde_json::Value {
        serde_json::json!(self
            .get_results_summary()
            .into_iter()
            .map(|(region, results)| (region.to_string(), results))
            .collect::<HashMap<String, SsmValidationRegionSummary>>())
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use crate::aws::validate_ssm::results::{
        SsmValidationResult, SsmValidationResultStatus, SsmValidationResults,
    };
    use aws_sdk_ssm::Region;

    // These tests assert that the `get_results_for_status` function returns the correct values.

    // Tests empty SsmValidationResults
    #[test]
    fn get_results_for_status_empty() {
        let results = SsmValidationResults::new(HashMap::from([
            (Region::new("us-west-2"), Ok(HashSet::from([]))),
            (Region::new("us-east-1"), Ok(HashSet::from([]))),
        ]));
        let results_filtered = results.get_results_for_status(&vec![
            SsmValidationResultStatus::Correct,
            SsmValidationResultStatus::Incorrect,
            SsmValidationResultStatus::Missing,
            SsmValidationResultStatus::Unexpected,
        ]);

        assert_eq!(results_filtered, HashSet::new());
    }

    // Tests the `Correct` status
    #[test]
    fn get_results_for_status_correct() {
        let results = SsmValidationResults::new(HashMap::from([
            (
                Region::new("us-west-2"),
                Ok(HashSet::from([
                    SsmValidationResult::new(
                        "test3-parameter-name".to_string(),
                        Some("test3-parameter-value".to_string()),
                        None,
                        Region::new("us-west-2"),
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
                        Region::new("us-west-2"),
                        None,
                    ),
                ])),
            ),
            (
                Region::new("us-east-1"),
                Ok(HashSet::from([
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
                        Region::new("us-east-1"),
                        Some("test1-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test2-parameter-name".to_string(),
                        Some("test2-parameter-value".to_string()),
                        Some("test2-parameter-value-wrong".to_string()),
                        Region::new("us-east-1"),
                        Some("test2-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test4-parameter-name".to_string(),
                        None,
                        Some("test4-parameter-value".to_string()),
                        Region::new("us-east-1"),
                        None,
                    ),
                ])),
            ),
        ]));
        let results_filtered =
            results.get_results_for_status(&vec![SsmValidationResultStatus::Correct]);

        assert_eq!(
            results_filtered,
            HashSet::from([
                &SsmValidationResult::new(
                    "test1-parameter-name".to_string(),
                    Some("test1-parameter-value".to_string()),
                    Some("test1-parameter-value".to_string()),
                    Region::new("us-west-2"),
                    Some("test1-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test1-parameter-name".to_string(),
                    Some("test1-parameter-value".to_string()),
                    Some("test1-parameter-value".to_string()),
                    Region::new("us-east-1"),
                    Some("test1-image-id".to_string()),
                )
            ])
        );
    }

    // Tests a filter containing the `Correct` and `Incorrect` statuses
    #[test]
    fn get_results_for_status_correct_incorrect() {
        let results = SsmValidationResults::new(HashMap::from([
            (
                Region::new("us-west-2"),
                Ok(HashSet::from([
                    SsmValidationResult::new(
                        "test3-parameter-name".to_string(),
                        Some("test3-parameter-value".to_string()),
                        None,
                        Region::new("us-west-2"),
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
                        Region::new("us-west-2"),
                        None,
                    ),
                ])),
            ),
            (
                Region::new("us-east-1"),
                Ok(HashSet::from([
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
                        Region::new("us-east-1"),
                        Some("test1-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test2-parameter-name".to_string(),
                        Some("test2-parameter-value".to_string()),
                        Some("test2-parameter-value-wrong".to_string()),
                        Region::new("us-east-1"),
                        Some("test2-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test4-parameter-name".to_string(),
                        None,
                        Some("test4-parameter-value".to_string()),
                        Region::new("us-east-1"),
                        None,
                    ),
                ])),
            ),
        ]));
        let results_filtered = results.get_results_for_status(&vec![
            SsmValidationResultStatus::Correct,
            SsmValidationResultStatus::Incorrect,
        ]);

        assert_eq!(
            results_filtered,
            HashSet::from([
                &SsmValidationResult::new(
                    "test1-parameter-name".to_string(),
                    Some("test1-parameter-value".to_string()),
                    Some("test1-parameter-value".to_string()),
                    Region::new("us-west-2"),
                    Some("test1-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test1-parameter-name".to_string(),
                    Some("test1-parameter-value".to_string()),
                    Some("test1-parameter-value".to_string()),
                    Region::new("us-east-1"),
                    Some("test1-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test2-parameter-name".to_string(),
                    Some("test2-parameter-value".to_string()),
                    Some("test2-parameter-value-wrong".to_string()),
                    Region::new("us-west-2"),
                    Some("test2-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test2-parameter-name".to_string(),
                    Some("test2-parameter-value".to_string()),
                    Some("test2-parameter-value-wrong".to_string()),
                    Region::new("us-east-1"),
                    Some("test2-image-id".to_string()),
                )
            ])
        );
    }

    // Tests a filter containing all statuses
    #[test]
    fn get_results_for_status_all() {
        let results = SsmValidationResults::new(HashMap::from([
            (
                Region::new("us-west-2"),
                Ok(HashSet::from([
                    SsmValidationResult::new(
                        "test3-parameter-name".to_string(),
                        Some("test3-parameter-value".to_string()),
                        None,
                        Region::new("us-west-2"),
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
                        Region::new("us-west-2"),
                        None,
                    ),
                ])),
            ),
            (
                Region::new("us-east-1"),
                Ok(HashSet::from([
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
                        Region::new("us-east-1"),
                        Some("test1-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test2-parameter-name".to_string(),
                        Some("test2-parameter-value".to_string()),
                        Some("test2-parameter-value-wrong".to_string()),
                        Region::new("us-east-1"),
                        Some("test2-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test4-parameter-name".to_string(),
                        None,
                        Some("test4-parameter-value".to_string()),
                        Region::new("us-east-1"),
                        None,
                    ),
                ])),
            ),
        ]));
        let results_filtered = results.get_results_for_status(&vec![
            SsmValidationResultStatus::Correct,
            SsmValidationResultStatus::Incorrect,
            SsmValidationResultStatus::Missing,
            SsmValidationResultStatus::Unexpected,
        ]);

        assert_eq!(
            results_filtered,
            HashSet::from([
                &SsmValidationResult::new(
                    "test1-parameter-name".to_string(),
                    Some("test1-parameter-value".to_string()),
                    Some("test1-parameter-value".to_string()),
                    Region::new("us-west-2"),
                    Some("test1-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test1-parameter-name".to_string(),
                    Some("test1-parameter-value".to_string()),
                    Some("test1-parameter-value".to_string()),
                    Region::new("us-east-1"),
                    Some("test1-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test2-parameter-name".to_string(),
                    Some("test2-parameter-value".to_string()),
                    Some("test2-parameter-value-wrong".to_string()),
                    Region::new("us-west-2"),
                    Some("test2-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test2-parameter-name".to_string(),
                    Some("test2-parameter-value".to_string()),
                    Some("test2-parameter-value-wrong".to_string()),
                    Region::new("us-east-1"),
                    Some("test2-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test3-parameter-name".to_string(),
                    Some("test3-parameter-value".to_string()),
                    None,
                    Region::new("us-west-2"),
                    Some("test3-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test4-parameter-name".to_string(),
                    None,
                    Some("test4-parameter-value".to_string()),
                    Region::new("us-west-2"),
                    None,
                ),
                &SsmValidationResult::new(
                    "test3-parameter-name".to_string(),
                    Some("test3-parameter-value".to_string()),
                    None,
                    Region::new("us-east-1"),
                    Some("test3-image-id".to_string()),
                ),
                &SsmValidationResult::new(
                    "test4-parameter-name".to_string(),
                    None,
                    Some("test4-parameter-value".to_string()),
                    Region::new("us-east-1"),
                    None,
                )
            ])
        );
    }

    // Tests the `Missing` filter when none of the SsmValidationResults have this status
    #[test]
    fn get_results_for_status_missing_none() {
        let results = SsmValidationResults::new(HashMap::from([
            (
                Region::new("us-west-2"),
                Ok(HashSet::from([
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
                        Region::new("us-west-2"),
                        None,
                    ),
                ])),
            ),
            (
                Region::new("us-east-1"),
                Ok(HashSet::from([
                    SsmValidationResult::new(
                        "test1-parameter-name".to_string(),
                        Some("test1-parameter-value".to_string()),
                        Some("test1-parameter-value".to_string()),
                        Region::new("us-east-1"),
                        Some("test1-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test2-parameter-name".to_string(),
                        Some("test2-parameter-value".to_string()),
                        Some("test2-parameter-value-wrong".to_string()),
                        Region::new("us-east-1"),
                        Some("test2-image-id".to_string()),
                    ),
                    SsmValidationResult::new(
                        "test4-parameter-name".to_string(),
                        None,
                        Some("test4-parameter-value".to_string()),
                        Region::new("us-east-1"),
                        None,
                    ),
                ])),
            ),
        ]));
        let results_filtered =
            results.get_results_for_status(&vec![SsmValidationResultStatus::Missing]);

        assert_eq!(results_filtered, HashSet::new());
    }
}
