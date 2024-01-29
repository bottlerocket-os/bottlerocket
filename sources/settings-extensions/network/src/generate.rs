/// Generators for network settings
use error::{ExecutionSnafu, GenerateError, InvalidHostnameSnafu};
use modeled_types::ValidLinuxHostname;
use snafu::ResultExt;
use std::process::Command;

pub fn generate_hostname() -> std::result::Result<ValidLinuxHostname, GenerateError> {
    let ret = Command::new("netdog")
        .arg("generate-hostname")
        .output()
        .context(ExecutionSnafu)?;

    if !ret.status.success() {
        Err(GenerateError::Failure {
            message: String::from_utf8_lossy(&ret.stderr).to_string(),
        })
    } else {
        let hostname = parse_stdout(String::from_utf8_lossy(&ret.stdout).to_string());
        Ok(ValidLinuxHostname::try_from(hostname).context(InvalidHostnameSnafu)?)
    }
}

fn parse_stdout(stdout: String) -> String {
    // netdog gives us a response in the form of `"{hostname}"\n`, we should strip the whitespace
    // and any quotations.
    stdout.trim().trim_matches('"').to_string()
}

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum GenerateError {
        #[snafu(display("Failed to run netdog command: {}", source))]
        Execution { source: std::io::Error },
        #[snafu(display("Generation failed: {}", message))]
        Failure { message: String },
        #[snafu(display("Generated invalid hostname: {}", source))]
        InvalidHostname { source: modeled_types::error::Error },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_hostname() {
        let stdout = String::from("\"foo.bar.com\"\n");
        let hostname = parse_stdout(stdout);
        assert_eq!(hostname, "foo.bar.com")
    }
}
