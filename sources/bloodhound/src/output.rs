use std::io::{Error, Write};

use crate::results::ReportResults;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ReportWriter {
    fn write(&self, report: &ReportResults, output: &mut dyn Write) -> Result<(), Error>;
}

pub struct TextReportWriter {}

impl ReportWriter for TextReportWriter {
    /// Writes a text formatted report to the provided output destination.
    fn write(&self, report: &ReportResults, output: &mut dyn Write) -> Result<(), Error> {
        if let Some(name) = &report.metadata.name {
            writeln!(output, "{:17}{}", "Benchmark name:", name)?;
        }
        if let Some(version) = &report.metadata.version {
            writeln!(output, "{:17}{}", "Version:", version)?;
        }
        if let Some(url) = &report.metadata.url {
            writeln!(output, "{:17}{}", "Reference:", url)?;
        }
        writeln!(output, "{:17}{}", "Benchmark level:", report.level)?;
        writeln!(output, "{:17}{}", "Start time:", report.timestamp)?;
        writeln!(output)?;

        for test_result in report.results.values() {
            writeln!(
                output,
                "[{}] {:9} {} ({})",
                test_result.result.status,
                test_result.metadata.id,
                test_result.metadata.title,
                test_result.metadata.mode
            )?;
        }

        writeln!(output)?;
        writeln!(output, "{:17}{}", "Passed:", report.passed)?;
        writeln!(output, "{:17}{}", "Failed:", report.failed)?;
        writeln!(output, "{:17}{}", "Skipped:", report.skipped)?;
        writeln!(output, "{:17}{}", "Total checks:", report.total)?;
        writeln!(output)?;
        writeln!(output, "Compliance check result: {}", report.status)
    }
}

pub struct JsonReportWriter {}

impl ReportWriter for JsonReportWriter {
    /// Writes a json formatted report to the provided output destination.
    fn write(&self, report: &ReportResults, output: &mut dyn Write) -> Result<(), Error> {
        let json = serde_json::to_string(&report)?;
        writeln!(output, "{}", json)
    }
}
