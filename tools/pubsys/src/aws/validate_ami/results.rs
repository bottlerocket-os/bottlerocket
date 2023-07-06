//! The results module owns the reporting of EC2 image validation results.

use super::ami::ImageDef;
use super::Result;
use aws_sdk_ec2::Region;
use serde::{Deserialize, Serialize};
use serde_plain::{derive_display_from_serialize, derive_fromstr_from_deserialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use tabled::{Table, Tabled};

/// Represent the possible status of an EC2 image validation
#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum AmiValidationResultStatus {
    /// The image was found and its monitored fields have the expected values
    Correct,

    /// The image was found but some of the monitored fields do not have the expected values
    Incorrect,

    /// The image was expected but not included in the actual images
    Missing,

    /// The region containing the image was not reachable
    Unreachable,
}

derive_display_from_serialize!(AmiValidationResultStatus);
derive_fromstr_from_deserialize!(AmiValidationResultStatus);

/// Represents a single EC2 image validation result
#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct AmiValidationResult {
    /// The ID of the image
    pub(crate) id: String,

    /// `ImageDef` containing expected values for the image
    pub(crate) expected_image_def: ImageDef,

    /// `ImageDef` containing actual values for the image
    pub(crate) actual_image_def: Option<ImageDef>,

    /// The region the image resides in
    #[serde(serialize_with = "serialize_region")]
    pub(crate) region: Region,

    /// The validation status of the image
    pub(crate) status: AmiValidationResultStatus,
}

fn serialize_region<S>(region: &Region, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(region.to_string().as_str())
}

impl AmiValidationResult {
    pub(crate) fn new(
        id: String,
        expected_image_def: ImageDef,
        actual_image_def: Result<Option<ImageDef>>,
        region: Region,
    ) -> Self {
        // Determine the validation status based on equality, presence, and absence of expected and
        // actual image values
        let status = match (&expected_image_def, &actual_image_def) {
            (expected_image_def, Ok(Some(actual_image_def)))
                if actual_image_def == expected_image_def =>
            {
                AmiValidationResultStatus::Correct
            }
            (_, Ok(Some(_))) => AmiValidationResultStatus::Incorrect,
            (_, Ok(None)) => AmiValidationResultStatus::Missing,
            (_, Err(_)) => AmiValidationResultStatus::Unreachable,
        };
        AmiValidationResult {
            id,
            expected_image_def,
            actual_image_def: actual_image_def.unwrap_or_default(),
            region,
            status,
        }
    }
}

#[derive(Tabled, Serialize)]
struct AmiValidationRegionSummary {
    correct: u64,
    incorrect: u64,
    missing: u64,
    unreachable: u64,
}

impl From<&HashSet<AmiValidationResult>> for AmiValidationRegionSummary {
    fn from(results: &HashSet<AmiValidationResult>) -> Self {
        let mut region_validation = AmiValidationRegionSummary {
            correct: 0,
            incorrect: 0,
            missing: 0,
            unreachable: 0,
        };
        for validation_result in results {
            match validation_result.status {
                AmiValidationResultStatus::Correct => region_validation.correct += 1,
                AmiValidationResultStatus::Incorrect => region_validation.incorrect += 1,
                AmiValidationResultStatus::Missing => region_validation.missing += 1,
                AmiValidationResultStatus::Unreachable => region_validation.missing += 1,
            }
        }
        region_validation
    }
}

/// Represents all EC2 image validation results
#[derive(Debug)]
pub(crate) struct AmiValidationResults {
    pub(crate) results: HashMap<Region, HashSet<AmiValidationResult>>,
}

impl Default for AmiValidationResults {
    fn default() -> Self {
        Self::from_result_map(HashMap::new())
    }
}

impl Display for AmiValidationResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create a summary for each region, counting the number of parameters per status
        let region_validations: HashMap<Region, AmiValidationRegionSummary> =
            self.get_results_summary();

        // Represent the `HashMap` of summaries as a `Table`
        let table = Table::new(
            region_validations
                .iter()
                .map(|(region, results)| (region.to_string(), results))
                .collect::<Vec<(String, &AmiValidationRegionSummary)>>(),
        )
        .to_string();
        write!(f, "{}", table)
    }
}

impl AmiValidationResults {
    pub(crate) fn from_result_map(results: HashMap<Region, HashSet<AmiValidationResult>>) -> Self {
        AmiValidationResults { results }
    }

    /// Returns a `HashSet` containing all validation results whose status is present in `requested_status`
    pub(crate) fn get_results_for_status(
        &self,
        requested_status: &[AmiValidationResultStatus],
    ) -> HashSet<&AmiValidationResult> {
        let mut results = HashSet::new();
        for region_results in self.results.values() {
            results.extend(
                region_results
                    .iter()
                    .filter(|result| requested_status.contains(&result.status))
                    .collect::<HashSet<&AmiValidationResult>>(),
            )
        }
        results
    }

    /// Returns a `HashSet` containing all validation results
    pub(crate) fn get_all_results(&self) -> HashSet<&AmiValidationResult> {
        let mut results = HashSet::new();
        for region_results in self.results.values() {
            results.extend(region_results)
        }
        results
    }

    fn get_results_summary(&self) -> HashMap<Region, AmiValidationRegionSummary> {
        self.results
            .iter()
            .map(|(region, region_result)| {
                (
                    region.clone(),
                    AmiValidationRegionSummary::from(region_result),
                )
            })
            .collect()
    }

    pub(crate) fn get_json_summary(&self) -> serde_json::Value {
        serde_json::json!(self
            .get_results_summary()
            .into_iter()
            .map(|(region, results)| (region.to_string(), results))
            .collect::<HashMap<String, AmiValidationRegionSummary>>())
    }
}

#[cfg(test)]
mod test {
    use super::{AmiValidationResult, AmiValidationResultStatus, AmiValidationResults};
    use crate::aws::validate_ami::ami::ImageDef;
    use aws_sdk_ssm::Region;
    use std::collections::{HashMap, HashSet};

    // These tests assert that the `get_results_for_status` function returns the correct values.

    // Tests empty `AmiValidationResults`
    #[test]
    fn get_results_for_status_empty() {
        let results = AmiValidationResults::from_result_map(HashMap::from([
            (Region::new("us-west-2"), HashSet::from([])),
            (Region::new("us-east-1"), HashSet::from([])),
        ]));
        let results_filtered = results.get_results_for_status(&vec![
            AmiValidationResultStatus::Correct,
            AmiValidationResultStatus::Incorrect,
            AmiValidationResultStatus::Missing,
        ]);

        assert_eq!(results_filtered, HashSet::new());
    }

    // Tests the `Correct` status
    #[test]
    fn get_results_for_status_correct() {
        let results = AmiValidationResults::from_result_map(HashMap::from([
            (
                Region::new("us-west-2"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "not simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                ]),
            ),
            (
                Region::new("us-east-1"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "not simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                ]),
            ),
        ]));
        let results_filtered =
            results.get_results_for_status(&vec![AmiValidationResultStatus::Correct]);

        assert_eq!(
            results_filtered,
            HashSet::from([
                &AmiValidationResult::new(
                    "test1-image-id".to_string(),
                    ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-west-2"),
                ),
                &AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-east-1"),
                )
            ])
        );
    }

    // Tests a filter containing the `Correct` and `Incorrect` statuses
    #[test]
    fn get_results_for_status_correct_incorrect() {
        let results = AmiValidationResults::from_result_map(HashMap::from([
            (
                Region::new("us-west-2"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(None),
                        Region::new("us-west-2"),
                    ),
                ]),
            ),
            (
                Region::new("us-east-1"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(None),
                        Region::new("us-east-1"),
                    ),
                ]),
            ),
        ]));
        let results_filtered = results.get_results_for_status(&vec![
            AmiValidationResultStatus::Correct,
            AmiValidationResultStatus::Incorrect,
        ]);

        assert_eq!(
            results_filtered,
            HashSet::from([
                &AmiValidationResult::new(
                    "test1-image-id".to_string(),
                    ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-west-2"),
                ),
                &AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-east-1"),
                ),
                &AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: false,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-west-2"),
                ),
                &AmiValidationResult::new(
                    "test1-image-id".to_string(),
                    ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: false,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-east-1"),
                )
            ])
        );
    }

    // Tests a filter containing all statuses
    #[test]
    fn get_results_for_status_all() {
        let results = AmiValidationResults::from_result_map(HashMap::from([
            (
                Region::new("us-west-2"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(None),
                        Region::new("us-west-2"),
                    ),
                ]),
            ),
            (
                Region::new("us-east-1"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(None),
                        Region::new("us-east-1"),
                    ),
                ]),
            ),
            (
                Region::new("us-east-2"),
                HashSet::from([AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Err(crate::aws::validate_ami::error::Error::UnreachableRegion {
                        region: "us-east-2".to_string(),
                    }),
                    Region::new("us-east-2"),
                )]),
            ),
        ]));
        let results_filtered = results.get_results_for_status(&vec![
            AmiValidationResultStatus::Correct,
            AmiValidationResultStatus::Incorrect,
            AmiValidationResultStatus::Missing,
            AmiValidationResultStatus::Unreachable,
        ]);

        assert_eq!(
            results_filtered,
            HashSet::from([
                &AmiValidationResult::new(
                    "test1-image-id".to_string(),
                    ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-west-2"),
                ),
                &AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-east-1"),
                ),
                &AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: false,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-west-2"),
                ),
                &AmiValidationResult::new(
                    "test1-image-id".to_string(),
                    ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(Some(ImageDef {
                        id: "test1-image-id".to_string(),
                        name: "test1-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: false,
                        sriov_net_support: "simple".to_string(),
                    })),
                    Region::new("us-east-1"),
                ),
                &AmiValidationResult::new(
                    "test2-image-id".to_string(),
                    ImageDef {
                        id: "test2-image-id".to_string(),
                        name: "test2-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(None),
                    Region::new("us-west-2"),
                ),
                &AmiValidationResult::new(
                    "test2-image-id".to_string(),
                    ImageDef {
                        id: "test2-image-id".to_string(),
                        name: "test2-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Ok(None),
                    Region::new("us-east-1"),
                ),
                &AmiValidationResult::new(
                    "test3-image-id".to_string(),
                    ImageDef {
                        id: "test3-image-id".to_string(),
                        name: "test3-image".to_string(),
                        public: true,
                        launch_permissions: None,
                        ena_support: true,
                        sriov_net_support: "simple".to_string(),
                    },
                    Err(crate::aws::validate_ami::error::Error::UnreachableRegion {
                        region: "us-east-2".to_string(),
                    }),
                    Region::new("us-east-2"),
                ),
            ])
        );
    }

    // Tests the `Missing` filter when none of the AmiValidationResults have this status
    #[test]
    fn get_results_for_status_missing_none() {
        let results = AmiValidationResults::from_result_map(HashMap::from([
            (
                Region::new("us-west-2"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "not simple".to_string(),
                        })),
                        Region::new("us-west-2"),
                    ),
                ]),
            ),
            (
                Region::new("us-east-1"),
                HashSet::from([
                    AmiValidationResult::new(
                        "test3-image-id".to_string(),
                        ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test3-image-id".to_string(),
                            name: "test3-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test1-image-id".to_string(),
                        ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test1-image-id".to_string(),
                            name: "test1-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: false,
                            sriov_net_support: "simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                    AmiValidationResult::new(
                        "test2-image-id".to_string(),
                        ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "simple".to_string(),
                        },
                        Ok(Some(ImageDef {
                            id: "test2-image-id".to_string(),
                            name: "test2-image".to_string(),
                            public: true,
                            launch_permissions: None,
                            ena_support: true,
                            sriov_net_support: "not simple".to_string(),
                        })),
                        Region::new("us-east-1"),
                    ),
                ]),
            ),
        ]));
        let results_filtered =
            results.get_results_for_status(&vec![AmiValidationResultStatus::Missing]);

        assert_eq!(results_filtered, HashSet::new());
    }
}
