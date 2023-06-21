/*!
# Introduction

Bloodhound is a command line orchestrator for running a set of compliance
checks. This can be used to run CIS benchmark compliance, though it can be extended
to perform any kind of check that adheres to the expected checker interface.

Checks are performed and their results are provided in an overall report.
The checker report can be written to a file, or viewed from stdout.
By default the report is provided in a human readable text format, but can also
be generated as JSON to make it easy to consume programmatically for integrating
into further compliance automation.

# Usage

Bloodhound is ultimately intended to be used through the Bottlerocket `apiclient`
interface.
If executing directly, run `bloodhound --help` for usage information.
*/

use bloodhound::args::*;
use bloodhound::output::{JsonReportWriter, ReportWriter, TextReportWriter};
use bloodhound::results::{
    CheckStatus, CheckerMetadata, CheckerResult, ReportMetadata, ReportResults,
};
use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{stdout, BufReader, Error, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};
use std::{fs, path::PathBuf};

// Define some exit codes for error conditions
const CHECKER_DISCOVERY_ERROR: i32 = 2;
const REPORT_OUTPUT_ERROR: i32 = 3;
const NO_CHECKS_RUN_ERROR: i32 = 4;

/// Discovers the metadata information for the checkers being run or provides a default.
fn read_metadata(check_dir: &Path) -> ReportMetadata {
    let meta_path = check_dir.join("metadata.json");

    if let Ok(file) = File::open(meta_path) {
        let reader = BufReader::new(file);
        if let Ok(report_metadata) = serde_json::from_reader(reader) {
            return report_metadata;
        }
    }

    ReportMetadata {
        name: None,
        version: None,
        url: None,
    }
}

/// Discover all executable checker files for the given directory.
/// By looking at the provided path, this does some basic checks and filtering
/// to find the executable files that appear to be bloodhound checkers.
/// It will parse the checker metadata to provide a mapping between a filesystem
/// path and a specific check, including only those checkers that are within the
/// request compliance level.
fn find_checkers(check_dir: &PathBuf, level: u8) -> HashMap<String, CheckerMetadata> {
    let entries: Vec<DirEntry> = fs::read_dir(check_dir)
        .unwrap()
        .filter_map(|file| file.ok())
        .collect();
    let mut result = HashMap::new();

    for entry in entries {
        if let Ok(file_metadata) = fs::metadata(entry.path()) {
            // Filter out any subdirectories
            if file_metadata.is_dir() {
                continue;
            }

            // Skip any files that are not executable
            if file_metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }

            // It's an executable file, make sure it implements our expected checker interface
            let metadata;
            if let Ok(output) = Command::new(entry.path()).arg("metadata").output() {
                metadata = String::from_utf8_lossy(&output.stdout).to_string();
            } else {
                eprintln!(
                    "{:?} does not appear to be a checker executable",
                    entry.path()
                );
                continue;
            }

            if let Ok(checker_data) = serde_json::from_str::<CheckerMetadata>(&metadata) {
                if checker_data.level <= level {
                    result.insert(
                        // Assuming here that we will never have non-ASCII characters in our usage
                        entry.path().as_os_str().to_string_lossy().to_string(),
                        checker_data,
                    );
                }
            } else {
                eprintln!("Unable to parse checker metadata from {:?}", entry.path());
            }
        }
    }

    result
}

/// Processes the output results from calling an individual checker. Results of
/// parsing the checker output are added to the overall execution report.
fn process_checker_results(output: Output, data: CheckerMetadata, report: &mut ReportResults) {
    if output.status.success() {
        let check_output = String::from_utf8_lossy(&output.stdout).to_string();
        if let Ok(checker_result) = serde_json::from_str::<CheckerResult>(&check_output) {
            report.add_result(data, checker_result);
            return;
        }
    }

    let check_output = String::from_utf8_lossy(&output.stderr).to_string();
    report.add_result(
        data,
        CheckerResult {
            status: CheckStatus::SKIP,
            error: check_output,
        },
    );
}

fn get_output(output: &Option<String>) -> Result<Box<dyn Write>, Error> {
    match output {
        Some(path) => File::create(path).map(|f| Box::new(f) as Box<dyn Write>),
        None => Ok(Box::new(stdout())),
    }
}

fn main() {
    let args: Arguments = argh::from_env();

    // Discover all checkers in checks directory
    let check_dir = PathBuf::from(args.check_dir);
    if !check_dir.is_dir() {
        eprintln!("Checker path {:?} is not a directory!", check_dir);
        std::process::exit(CHECKER_DISCOVERY_ERROR);
    }

    // Look through the directory and find any checkers, then filter out checks based on input
    let checkers = find_checkers(&check_dir, args.level);
    if checkers.is_empty() {
        eprintln!("No checkers found in {:?}!", check_dir);
        std::process::exit(CHECKER_DISCOVERY_ERROR);
    }

    let report_metadata = read_metadata(&check_dir);
    let mut report = ReportResults::new(args.level, report_metadata);

    // Execute each checker and capture results
    for (checker, data) in checkers {
        if let Ok(output) = Command::new(checker).output() {
            process_checker_results(output, data, &mut report);
        } else {
            // Something failed in execution, mark as skipped with message
            let msg = format!("Error executing {} checker.", data.name);
            report.add_result(
                data,
                CheckerResult {
                    status: CheckStatus::SKIP,
                    error: msg,
                },
            );
        }
    }

    // Write appropriate output results report
    let mut output_dest = get_output(&args.output).unwrap_or_else(|err| {
        eprintln!("Error writing to output destination {}!", err);
        std::process::exit(REPORT_OUTPUT_ERROR);
    });

    let reporter: &dyn ReportWriter = match args.format {
        Format::Json => &JsonReportWriter {},
        Format::Text => &TextReportWriter {},
    };

    if let Err(err) = reporter.write(&report, &mut *output_dest) {
        eprintln!("Error writing report output: {}", err);
        std::process::exit(REPORT_OUTPUT_ERROR);
    }

    if report.status == CheckStatus::SKIP {
        // Something is wrong, no automated checks were able to run. Better
        // alert on something like this as it may be a sign of a larger problem.
        eprintln!("Warning: No checks were able to run");
        std::process::exit(NO_CHECKS_RUN_ERROR);
    }
}
