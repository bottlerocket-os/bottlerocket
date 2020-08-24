//! Provides the list of log requests that `logdog` will run and provides the handler functions to
//! run them.
//!
//! # Static Log Requests
//!
//! At build time, the `build.rs` file checks which variant is being built and creates a symlink
//! file which points to the log requests for the current variant. This file is named `logdog.conf`.
//! We load `logdog.conf` and `logdog.common.conf` files into static strings at compile time, and
//! these provide the list of log requests that `logdog` will run.

use crate::error::{self, Result};
use reqwest::blocking::{Client, Response};
use snafu::{ensure, OptionExt, ResultExt};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use url::Url;

/// The `logdog` log requests that all variants have in common.
const COMMON_REQUESTS: &str = include_str!("../conf/logdog.common.conf");
/// The `logdog` log requests that are specific to the current variant.
const VARIANT_REQUESTS: &str = include_str!("../conf/current/logdog.conf");

/// Returns the list of log requests to run by combining `VARIANT_REQUESTS` and `COMMON_REQUESTS`.
/// These are read at compile time from files named `logdog.conf` and `logdog.common.conf`
/// respectively.
pub(crate) fn log_requests() -> Vec<&'static str> {
    COMMON_REQUESTS
        .lines()
        .chain(VARIANT_REQUESTS.lines())
        .filter(|&command| !command.is_empty() && !command.trim_start().starts_with('#'))
        .collect()
}

/// A logdog `LogRequest` is in the format `mode filename instructions`. `mode` specifies what type
/// of command it is, e.g. `exec ` for a command or `http` for an HTTP get request. `filename` is
/// the name of the output file. `instructions` is any additional information needed.  For example,
/// an `exec` request'ss instructions will include the program and program arguments. An `http`
/// request's instructions will be the URL.
///
/// # Examples
///
/// This request will run the echo program with the arguments `hello` `world` and write the output
/// to a file named `hello.txt`. In this example `exec` is the mode, `hello.txt` is the output
/// filename, and `echo hello world` is the instructions.
///
/// ```text
/// exec hello.txt echo hello world
/// ```
///
/// This request will run an HTTP get request to the url `http://example.com` and write the response
/// body to `example.txt`:
///
/// ```text
/// http exampe.txt http://example.com
/// ```
///
/// This request will copy a file from `/etc/some/conf` to a file name `some-conf`:
///
/// ```text
/// file some-conf /etc/some/conf
/// ```
#[derive(Debug, Clone)]
struct LogRequest<'a> {
    /// The log request mode. For example `exec`, `http`, or `file`.
    mode: &'a str,
    /// The filename that the logs will be written to.
    filename: &'a str,
    /// Any additional instructions or commands needed to fulfill the log request. For example, with
    /// `exec` this will be a program invocation like `echo hello world`. For an `http` request this
    /// will be a URL. For a `file` request this will be the source file path.
    instructions: &'a str,
}

/// This is used in error construction.
impl ToString for LogRequest<'_> {
    fn to_string(&self) -> String {
        if self.instructions.is_empty() {
            format!("{} {}", self.mode, self.filename)
        } else {
            format!("{} {} {}", self.mode, self.filename, self.instructions)
        }
    }
}

/// Runs a `LogRequest` and writes its output to a file in `tempdir`.
pub(crate) fn handle_log_request<S, P>(request: S, tempdir: P) -> Result<()>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let request = request.as_ref();
    // get the first and second token: i.e. mode and output filename, put the remainder of the
    // log request into the instructions field (or default to an empty string).
    let mut iter = request.splitn(3, ' ');
    let req = LogRequest {
        mode: iter.next().context(error::ModeMissing)?,
        filename: iter.next().context(error::FilenameMissing { request })?,
        instructions: iter.next().unwrap_or(""),
    };
    // execute the log request with the correct handler based on the mode field.
    match req.mode {
        "exec" => handle_exec_request(&req, tempdir)?,
        "http" | "https" => handle_http_request(&req, tempdir)?,
        "file" => handle_file_request(&req, tempdir)?,
        unmatched => {
            return Err(error::Error::UnhandledRequest {
                mode: unmatched.into(),
                request: request.into(),
            })
        }
    }
    Ok(())
}

/// Runs an `exec` `LogRequest`'s `instructions` and writes its output to to `tempdir`.
fn handle_exec_request<P>(request: &LogRequest<'_>, tempdir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let split = shell_words::split(request.instructions).with_context(|| error::CommandParse {
        command: request.to_string(),
    })?;
    let (command, args) = split.split_first().with_context(|| error::CommandMissing {
        request: request.to_string(),
    })?;
    let outpath = tempdir.as_ref().join(request.filename);
    let ofile = File::create(&outpath).context(error::CommandOutputFile { path: &outpath })?;
    let stderr_file = ofile
        .try_clone()
        .context(error::CommandErrFile { path: &outpath })?;
    Command::new(command)
        .args(args)
        .stdout(Stdio::from(ofile))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .with_context(|| error::CommandSpawn {
            command: request.to_string(),
        })?
        .wait_with_output()
        .with_context(|| error::CommandFinish {
            command: request.to_string(),
        })?;
    Ok(())
}

/// Executes an `http` `LogRequest` and writes the response body to a file in `tempdir`.
fn handle_http_request<P>(request: &LogRequest<'_>, tempdir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    ensure!(
        !request.instructions.is_empty(),
        error::HttpMissingUrl {
            request: request.to_string(),
        }
    );
    let outpath = tempdir.as_ref().join(request.filename);
    let response = send_get_request(request.instructions)?;
    let data = response.bytes().with_context(|| error::HttpResponseBytes {
        request: request.to_string(),
    })?;
    fs::write(&outpath, &data).with_context(|| error::HttpWriteBytes {
        request: request.to_string(),
        path: &outpath,
    })?;
    Ok(())
}

/// Uses the reqwest library to send a GET request to `URL` and returns the response.
fn send_get_request(url: &str) -> Result<Response> {
    let url = Url::parse(&url).context(error::HttpUrlParse { url })?;
    let client = Client::builder()
        .build()
        .with_context(|| error::HttpClient { url: url.clone() })?;
    let response = client
        .get(url.clone())
        .send()
        .with_context(|| error::HttpSend { url: url.clone() })?;
    let response = response
        .error_for_status()
        .context(error::HttpResponse { url })?;
    Ok(response)
}

/// Copies a file from the path given by `request.instructions` to the tempdir with filename given
/// by `request.filename`.
fn handle_file_request<P>(request: &LogRequest<'_>, tempdir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    ensure!(
        !request.instructions.is_empty(),
        error::FileFromEmpty {
            request: request.to_string()
        }
    );
    let dest = tempdir.as_ref().join(request.filename);
    let _ = fs::copy(&request.instructions, &dest).with_context(|| error::FileCopy {
        request: request.to_string(),
        from: request.instructions,
        to: &dest,
    })?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::log_request::handle_log_request;
    use std::fs::write;
    use tempfile::TempDir;

    #[test]
    fn file_request() {
        let source_dir = TempDir::new().unwrap();
        let source_filepath = source_dir.path().join("foo-bar.source");
        let want = "123";
        write(&source_filepath, want).unwrap();
        let request = format!("file foo-bar {}", source_filepath.display());
        let outdir = TempDir::new().unwrap();
        handle_log_request(&request, outdir.path()).unwrap();
        let outfile = outdir.path().join("foo-bar");
        let got = std::fs::read_to_string(&outfile).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn exec_request() {
        let want = "hello world! \"quoted\"\n";
        let request = r#"exec output-file.txt echo 'hello' "world!" "\"quoted\"""#;
        let outdir = TempDir::new().unwrap();
        handle_log_request(&request, outdir.path()).unwrap();
        let outfile = outdir.path().join("output-file.txt");
        let got = std::fs::read_to_string(&outfile).unwrap();
        assert_eq!(got, want);
    }
}
