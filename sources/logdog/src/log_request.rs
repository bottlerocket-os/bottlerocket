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
use datastore::deserialization::from_map;
use datastore::serialization::to_pairs;
use glob::{glob, Pattern};
use reqwest::blocking::{Client, Response};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use url::Url;
use walkdir::WalkDir;

/// The `logdog` log requests that all variants have in common.
const COMMON_REQUESTS: &str = include_str!("../conf/logdog.common.conf");
/// The `logdog` log requests that are specific to the current variant.
const VARIANT_REQUESTS: &str = include_str!("../conf/current/logdog.conf");

/// Patterns to filter from settings output.  These follow the Unix shell style pattern outlined
/// here: https://docs.rs/glob/0.3.0/glob/struct.Pattern.html.
const SENSITIVE_SETTINGS_PATTERNS: &[&str] = &[
    "*.user-data",
    "settings.kubernetes.bootstrap-token",
    // Can contain a username:password component
    "settings.network.https-proxy",
    "settings.kubernetes.server-key",
    "settings.container-registry.credentials",
    // Can be stored in settings.aws.credentials, but user can also add creds here
    "settings.aws.config",
    "settings.aws.credentials",
];

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

/// A logdog `LogRequest` represents a line from the config file. It starts with a "mode" that
/// specifies what type of request it is, e.g. `exec ` for a command or `http` for an HTTP get
/// request. Some modes then require a `filename` that determines where the data will be saved in
/// the output tarball.  The final field is `instructions` which is any additional information
/// needed by the given mode.  For example, an `exec` requests' instructions will include the
/// program and program arguments. An `http` request's instructions will be the URL.
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
/// http example.txt http://example.com
/// ```
///
/// This request will copy a file from `/etc/some/conf` to a file name `some-conf`:
///
/// ```text
/// file some-conf /etc/some/conf
/// ```
///
/// This request will copy files with a known prefix into the tarball; this can be useful for dated
/// log files, for example.
///
/// ```text
/// glob /var/log/my-app.log*
/// ```
#[derive(Debug, Clone)]
struct LogRequest<'a> {
    /// The log request mode. For example `exec`, `http`, `file`, or `glob`.
    mode: &'a str,
    /// The filename that the logs will be written to, if appropriate for the mode.
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
pub(crate) async fn handle_log_request<S, P>(request: S, tempdir: P) -> Result<()>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let request = request.as_ref();
    let mut iter = request.splitn(3, ' ');
    let mode = iter.next().context(error::ModeMissingSnafu)?;
    let req = if mode == "glob" {
        // for glob request format is "glob <pattern>"
        LogRequest {
            mode,
            filename: "",
            instructions: iter.next().context(error::PatternMissingSnafu)?,
        }
    } else {
        // Get the second token (output filename) and put the remainder of the
        // log request into the instructions field (or default to an empty string).
        LogRequest {
            mode,
            filename: iter
                .next()
                .context(error::FilenameMissingSnafu { request })?,
            instructions: iter.next().unwrap_or(""),
        }
    };
    // execute the log request with the correct handler based on the mode field.
    match req.mode {
        "settings" => handle_settings_request(&req, tempdir).await?,
        "exec" => handle_exec_request(&req, tempdir)?,
        "http" | "https" => handle_http_request(&req, tempdir)?,
        "file" => handle_file_request(&req, tempdir)?,
        "glob" => handle_glob_request(&req, tempdir)?,
        unmatched => {
            return Err(error::Error::UnhandledRequest {
                mode: unmatched.into(),
                request: request.into(),
            })
        }
    }
    Ok(())
}

/// Requests settings from the API, filters them, and writes the output to `tempdir`
async fn handle_settings_request<P>(request: &LogRequest<'_>, tempdir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let settings = get_settings().await?;
    let mut settings_map = to_pairs(&settings).context(error::SerializeSettingsSnafu)?;

    // Filter all settings that match any of the "sensitive" patterns
    for pattern in SENSITIVE_SETTINGS_PATTERNS {
        let pattern =
            Pattern::new(pattern).context(error::ParseGlobPatternSnafu { pattern: *pattern })?;
        settings_map.retain(|k, _| !pattern.matches(k.name().as_str()))
    }

    // Serialize the map back to a `Settings` to remove the escaping so it writes nicely to file
    let settings: model::Settings =
        from_map(&settings_map).context(error::DeserializeSettingsSnafu)?;
    let outpath = tempdir.as_ref().join(request.filename);
    let outfile = File::create(&outpath).context(error::FileCreateSnafu { path: &outpath })?;
    serde_json::to_writer_pretty(&outfile, &settings)
        .context(error::FileWriteSnafu { path: &outpath })?;
    Ok(())
}

/// Uses `apiclient` to request all settings from the apiserver and deserializes into a `Settings`
async fn get_settings() -> Result<model::Settings> {
    let uri = constants::API_SETTINGS_URI;
    let (_status, response_body) = apiclient::raw_request(constants::API_SOCKET, uri, "GET", None)
        .await
        .context(error::ApiClientSnafu { uri })?;

    serde_json::from_str(&response_body).context(error::SettingsJsonSnafu)
}

/// Runs an `exec` `LogRequest`'s `instructions` and writes its output to to `tempdir`.
fn handle_exec_request<P>(request: &LogRequest<'_>, tempdir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let split =
        shell_words::split(request.instructions).with_context(|_| error::CommandParseSnafu {
            command: request.to_string(),
        })?;
    let (command, args) = split
        .split_first()
        .with_context(|| error::CommandMissingSnafu {
            request: request.to_string(),
        })?;
    let outpath = tempdir.as_ref().join(request.filename);
    let ofile = File::create(&outpath).context(error::CommandOutputFileSnafu { path: &outpath })?;
    let stderr_file = ofile
        .try_clone()
        .context(error::CommandErrFileSnafu { path: &outpath })?;
    Command::new(command)
        .args(args)
        .stdout(Stdio::from(ofile))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .with_context(|_| error::CommandSpawnSnafu {
            command: request.to_string(),
        })?
        .wait_with_output()
        .with_context(|_| error::CommandFinishSnafu {
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
        error::HttpMissingUrlSnafu {
            request: request.to_string(),
        }
    );
    let outpath = tempdir.as_ref().join(request.filename);
    let response = send_get_request(request.instructions)?;
    let data = response
        .bytes()
        .with_context(|_| error::HttpResponseBytesSnafu {
            request: request.to_string(),
        })?;
    fs::write(&outpath, &data).with_context(|_| error::HttpWriteBytesSnafu {
        request: request.to_string(),
        path: &outpath,
    })?;
    Ok(())
}

/// Uses the reqwest library to send a GET request to `URL` and returns the response.
fn send_get_request(url: &str) -> Result<Response> {
    let url = Url::parse(url).context(error::HttpUrlParseSnafu { url })?;
    let client = Client::builder()
        .build()
        .with_context(|_| error::HttpClientSnafu { url: url.clone() })?;
    let response = client
        .get(url.clone())
        .send()
        .with_context(|_| error::HttpSendSnafu { url: url.clone() })?;
    let response = response
        .error_for_status()
        .context(error::HttpResponseSnafu { url })?;
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
        error::FileFromEmptySnafu {
            request: request.to_string()
        }
    );
    let dest = tempdir.as_ref().join(request.filename);
    let _ = fs::copy(request.instructions, &dest).with_context(|_| error::FileCopySnafu {
        request: request.to_string(),
        from: request.instructions,
        to: &dest,
    })?;
    Ok(())
}

/// Copies all files matching the glob pattern given by `request.instructions` to the tempdir with filename and path
/// same as source file.
fn handle_glob_request<P>(request: &LogRequest<'_>, tempdir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut files = HashSet::new();
    let glob_paths = glob(request.instructions).context(error::ParseGlobPatternSnafu {
        pattern: request.instructions,
    })?;
    for path in glob_paths.flatten() {
        if path.is_dir() {
            // iterate the directory and sub-directory to get all file paths
            for e in WalkDir::new(&path).into_iter().flatten() {
                if e.path().is_file() {
                    files.insert(e.into_path());
                }
            }
        } else {
            files.insert(path);
        }
    }
    for src_filepath in &files {
        // with glob pattern there are chances of multiple targets with same name, therefore
        // we maintain source file path and name in destination directory.
        // Eg. src file path "/a/b/file" will be converted to "dest_dir/a/b/file"
        let relative_path = src_filepath
            .strip_prefix("/")
            .unwrap_or(src_filepath.as_path());
        let dest_filepath = tempdir.as_ref().join(relative_path);
        let dest_dir_path = dest_filepath.parent().context(error::RootAsFileSnafu)?;
        // create directories in dest file path if it does not exist
        fs::create_dir_all(dest_dir_path).context(error::CreateOutputDirectorySnafu {
            path: dest_dir_path,
        })?;
        let _ = fs::copy(src_filepath, &dest_filepath).with_context(|_| error::FileCopySnafu {
            request: request.to_string(),
            from: src_filepath.to_str().unwrap_or("<unknown>"),
            to: &dest_filepath,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::log_request::handle_log_request;
    use std::fs;
    use std::fs::write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // adds a sub directory and some files to temp directory for file request tests
    fn create_source_dir(dir: &TempDir) {
        let filenames_content = [
            ("foo.source", "1"),
            ("bar.source", "2"),
            ("for-bar.log", "3"),
        ];
        // add files to temp directory
        for entry in filenames_content.iter() {
            let filepath = dir.path().join(entry.0);
            write(filepath, entry.1).unwrap();
        }

        let subdir_name_depth1 = "depth1";
        // create sub directory
        let subdir_path_depth1 = dir.path().join(subdir_name_depth1);
        fs::create_dir(&subdir_path_depth1).unwrap();
        // Add files to sub directory
        for entry in filenames_content.iter() {
            let filepath = subdir_path_depth1.join(entry.0);
            write(filepath, entry.1).unwrap();
        }

        let subdir_name_depth2 = "depth2";
        // create sub directory
        let subdir_path_depth2 = subdir_path_depth1.join(subdir_name_depth2);
        fs::create_dir(&subdir_path_depth2).unwrap();
        // Add files to sub directory
        for entry in filenames_content.iter() {
            let filepath = subdir_path_depth2.join(entry.0);
            write(filepath, entry.1).unwrap();
        }
    }

    fn get_dest_filepath(src_dir: &TempDir, filepath: &str) -> PathBuf {
        src_dir.path().strip_prefix("/").unwrap().join(filepath)
    }

    fn assert_file_match(dest_dir: &TempDir, filepath: PathBuf, want: &str) {
        let outfile = dest_dir.path().join(filepath);
        let got = std::fs::read_to_string(outfile).unwrap();
        assert_eq!(got, want);
    }

    #[tokio::test]
    async fn file_request() {
        let source_dir = TempDir::new().unwrap();
        let source_filepath = source_dir.path().join("foo-bar.source");
        let want = "123";
        write(&source_filepath, want).unwrap();
        let request = format!("file foo-bar {}", source_filepath.display());
        let outdir = TempDir::new().unwrap();
        handle_log_request(&request, outdir.path()).await.unwrap();
        let outfile = outdir.path().join("foo-bar");
        let got = std::fs::read_to_string(outfile).unwrap();
        assert_eq!(got, want);
    }

    #[tokio::test]
    async fn exec_request() {
        let want = "hello world! \"quoted\"\n";
        let request = r#"exec output-file.txt echo 'hello' "world!" "\"quoted\"""#;
        let outdir = TempDir::new().unwrap();
        handle_log_request(&request, outdir.path()).await.unwrap();
        let outfile = outdir.path().join("output-file.txt");
        let got = std::fs::read_to_string(outfile).unwrap();
        assert_eq!(got, want);
    }

    #[tokio::test]
    // ensures single file pattern works
    async fn glob_single_file_pattern_request() {
        let source_dir = TempDir::new().unwrap();
        create_source_dir(&source_dir);
        let outdir = TempDir::new().unwrap();
        let request = format!("glob {}/foo.source", source_dir.path().display());
        handle_log_request(&request, outdir.path()).await.unwrap();
        assert_file_match(&outdir, get_dest_filepath(&source_dir, "foo.source"), "1");
    }

    #[tokio::test]
    // ensures multiple file pattern works
    async fn glob_multiple_files_pattern_request() {
        let source_dir = TempDir::new().unwrap();
        create_source_dir(&source_dir);
        let outdir = TempDir::new().unwrap();
        let request = format!("glob {}/*.source", source_dir.path().display());
        handle_log_request(&request, outdir.path()).await.unwrap();
        assert_file_match(&outdir, get_dest_filepath(&source_dir, "foo.source"), "1");
        assert_file_match(&outdir, get_dest_filepath(&source_dir, "bar.source"), "2");
    }

    #[tokio::test]
    // ensures multiple file in nested directory pattern works
    async fn glob_nested_file_pattern_request() {
        let source_dir = TempDir::new().unwrap();
        create_source_dir(&source_dir);
        let outdir = TempDir::new().unwrap();
        let request = format!("glob {}/**/*.source", source_dir.path().display());
        handle_log_request(&request, outdir.path()).await.unwrap();
        assert_file_match(&outdir, get_dest_filepath(&source_dir, "foo.source"), "1");
        assert_file_match(&outdir, get_dest_filepath(&source_dir, "bar.source"), "2");
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/foo.source"),
            "1",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/bar.source"),
            "2",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/depth2/foo.source"),
            "1",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/depth2/bar.source"),
            "2",
        );
    }

    #[tokio::test]
    // ensures directory pattern works
    async fn glob_dir_pattern_request() {
        let source_dir = TempDir::new().unwrap();
        create_source_dir(&source_dir);
        let outdir = TempDir::new().unwrap();
        let request = format!("glob {}/**/", source_dir.path().display());
        handle_log_request(&request, outdir.path()).await.unwrap();
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/foo.source"),
            "1",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/bar.source"),
            "2",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/for-bar.log"),
            "3",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/depth2/foo.source"),
            "1",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/depth2/bar.source"),
            "2",
        );
        assert_file_match(
            &outdir,
            get_dest_filepath(&source_dir, "depth1/depth2/for-bar.log"),
            "3",
        );
    }

    #[tokio::test]
    // ensure if pattern is empty it should not panic
    async fn glob_empty_pattern_request() {
        let outdir = TempDir::new().unwrap();
        let request = "glob";
        let err = handle_log_request(&request, outdir.path())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::PatternMissing {}));
    }
}
