use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, usize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum CheckStatus {
    /// Successfully verified to be in the expected state.
    PASS,
    /// Found to not be in the expected state.
    FAIL,
    /// Unable to verify state, manual verification required.
    #[default]
    SKIP,
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mode {
    Automatic,
    Manual,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// CheckerMetadata contains the metadata about individual checkers. This data
/// is used by bloodhound to discover details about the available checks and
/// make decisions about including the checks based on input like the compliance
/// level to evaluate.
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckerMetadata {
    pub name: String,
    pub id: String,
    pub level: u8,
    pub title: String,
    pub mode: Mode,
}

impl fmt::Display for CheckerMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = serde_json::to_string(&self).unwrap_or_default();
        write!(f, "{}", output)
    }
}

/// CheckerResult contains the results of a performed check.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CheckerResult {
    pub status: CheckStatus,
    pub error: String,
}

impl fmt::Display for CheckerResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = serde_json::to_string(&self).unwrap_or_default();
        write!(f, "{}", output)
    }
}

/// The Checker trait defines the interface for a compliance check. Checkers are
/// expected to be able to provide metadata about the check it performs, and be
/// able to execute that check and provide the results of its findings.
///
/// The expectation for bloodhound checkers is:
/// - Check is performed normally, return JSON output from execute and exit 0
/// - Check has failed validation, return JSON output from execute and exit 0
/// - Check could not be performed, return error text to stderr and exit 1
pub trait Checker {
    fn metadata(&self) -> CheckerMetadata;
    fn execute(&self) -> CheckerResult;
}

/// Common checker type for reporting manual check results.
pub struct ManualChecker {
    pub name: String,
    pub id: String,
    pub title: String,
    pub level: u8,
}

impl Checker for ManualChecker {
    fn execute(&self) -> CheckerResult {
        CheckerResult {
            error: "Manual check, see benchmark for audit details.".to_string(),
            status: CheckStatus::SKIP,
        }
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: self.title.to_string(),
            id: self.id.to_string(),
            level: self.level,
            name: self.name.to_string(),
            mode: Mode::Manual,
        }
    }
}

/// Used to help serialize output into simpler JSON structure.
#[derive(Debug, Serialize)]
pub struct IndividualResult {
    #[serde(flatten)]
    pub metadata: CheckerMetadata,
    #[serde(flatten)]
    pub result: CheckerResult,
}

/// ReportResults are the overall compliance checking containing the results of
/// all individual checks run.
#[derive(Debug, Serialize)]
pub struct ReportResults {
    pub level: u8,
    pub total: usize,
    pub passed: usize,
    pub skipped: usize,
    pub failed: usize,
    pub status: CheckStatus,
    pub timestamp: String,
    #[serde(flatten)]
    pub metadata: ReportMetadata,
    pub results: BTreeMap<String, IndividualResult>,
}

impl ReportResults {
    /// Initialize a new `ReportResults` with the default values.
    pub fn new(level: u8, metadata: ReportMetadata) -> Self {
        let current_time: DateTime<Utc> = Utc::now();
        ReportResults {
            level,
            total: 0,
            passed: 0,
            skipped: 0,
            failed: 0,
            status: CheckStatus::SKIP,
            timestamp: format!("{:?}", current_time),
            metadata,
            results: BTreeMap::new(),
        }
    }

    /// Add the results of a checker run to the overall results.
    pub fn add_result(&mut self, metadata: CheckerMetadata, result: CheckerResult) {
        self.total += 1;
        match result.status {
            CheckStatus::FAIL => {
                self.failed += 1;
                self.status = CheckStatus::FAIL;
            }
            CheckStatus::PASS => {
                self.passed += 1;
                if self.status == CheckStatus::SKIP {
                    // We only want to mark as passing if at least one of the
                    // checks ran and passed
                    self.status = CheckStatus::PASS;
                }
            }
            CheckStatus::SKIP => {
                self.skipped += 1;
            }
        }
        self.results
            .insert(metadata.name.clone(), IndividualResult { metadata, result });
    }
}
