use results::{CheckStatus, CheckerResult};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::{ffi::OsStr, process::Command};

pub mod args;
pub mod output;
pub mod results;

/// Reads a file and checks if the given `search_str` is present in its contents.
pub fn look_for_string_in_file(path: &str, search_str: &str) -> Result<bool, io::Error> {
    let reader = BufReader::new(File::open(path)?);
    Ok(reader
        .lines()
        .any(|line| line.unwrap_or_default().contains(search_str)))
}

/// Reads a file and checks if the given `search_strs` are present in its contents.
pub fn look_for_strings_in_file(path: &str, search_strs: &[&str]) -> Result<bool, io::Error> {
    let mut matched = 0;

    let reader = BufReader::new(File::open(path)?);
    for line in reader.lines() {
        let content = line.unwrap_or_default();
        for search_str in search_strs {
            if content.contains(search_str) {
                matched += 1;
            }
        }
    }

    Ok(search_strs.len() <= matched)
}

/// Check if a given file contains all provided strings, getting a `CheckerResult` of its findings.
///
/// If all `strings_to_match` are found in `path`, the `CheckerResult` returned will have a `CheckStatus::PASS`.
///
/// If one or more of `strings_to_match` are not found in `path`, then even if some are found the `CheckerResult`
/// returned will have a `CheckStatus::FAIL` and the `error` field will contain `unable_to_find_error` as its content.
///
/// If `path` cannot be read or there is some other error checking the content of the file, returned status will be
/// `CheckStatus::SKIP` indicating a manual check will need to be performed and the `error` field will contain the
/// `unable_to_check_error` value.
#[macro_export]
macro_rules! check_file_contains {
    ($path:expr, $strings_to_match:expr, $unable_to_check_error:expr, $unable_to_find_error:expr) => {{
        let mut result = CheckerResult::default();

        if let Ok(found) = look_for_strings_in_file($path, $strings_to_match) {
            if found {
                result.status = CheckStatus::PASS;
            } else {
                result.error = $unable_to_find_error.to_string();
                result.status = CheckStatus::FAIL;
            }
        } else {
            result.error = $unable_to_check_error.to_string();
        }

        result
    }};
}

/// Executes a command and checks if the given `search_str` is in the output.
pub fn look_for_string_in_output<I, S>(cmd: &str, args: I, search_str: &str) -> Option<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let search_strs = [search_str];
    look_for_strings_in_output(cmd, args, &search_strs)
}

/// Executes a command and checks if the given `search_strs` are in the output.
pub fn look_for_strings_in_output<I, S>(cmd: &str, args: I, search_strs: &[&str]) -> Option<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if let Ok(output) = Command::new(cmd).args(args).output() {
        if output.status.success() {
            let mut matched = 0;

            let mp_output = String::from_utf8_lossy(&output.stdout).to_string();
            for line in mp_output.lines() {
                for search_str in search_strs {
                    if line.contains(search_str) {
                        matched += 1;
                    }
                }
            }

            Some(search_strs.len() <= matched)
        } else {
            None
        }
    } else {
        None
    }
}

/// Execute a command and check if the output contains all provided strings, getting a `CheckerResult` of its findings.
///
/// If all `strings_to_match` are found in the output, the `CheckerResult` returned will have a `CheckStatus::PASS`.
///
/// If one or more of `strings_to_match` are not found in the output, then even if some are found the `CheckerResult`
/// returned will have a `CheckStatus::FAIL` and the `error` field will contain `unable_to_find_error` as its content.
///
/// If execution fails or the output cannot be read, returned status will be `CheckStatus::SKIP` indicating a manual
/// check will need to be performed and the `error` field will contain the `unable_to_check_error` value.
#[macro_export]
macro_rules! check_output_contains {
    ($cmd:expr, $args:expr, $strings_to_match:expr, $unable_to_check_error:expr, $unable_to_find_error:expr) => {{
        let mut result = CheckerResult::default();

        if let Some(found) = look_for_strings_in_output($cmd, $args, $strings_to_match) {
            if found {
                result.status = CheckStatus::PASS;
            } else {
                result.error = $unable_to_find_error.to_string();
                result.status = CheckStatus::FAIL;
            }
        } else {
            result.error = $unable_to_check_error.to_string();
        }

        result
    }};
}

/// Tests whether a file has the given permission mode bits set, returning a `Result` based on the results.
pub fn check_file_not_mode(file_path: &str, mode: u32) -> CheckerResult {
    let mut result = CheckerResult::default();

    if let Ok(file) = File::open(file_path) {
        if let Ok(metadata) = file.metadata() {
            // Extract just the permission bits
            let file_mode = metadata.permissions().mode() & 0o777;

            if (file_mode & mode) > 0 {
                result.error = format!(
                    "file {} has extra permissions: 0x{:o}",
                    file_path, file_mode
                );
                result.status = CheckStatus::FAIL;
            } else {
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error = "unable to get file metadata information".to_string();
        }
    } else {
        result.error = format!("unable to inspect '{}'", file_path);
    }

    result
}

/// Verifies the file at the given path is owned by root:root, returning a `Results` based on the results.
pub fn ensure_file_owner_and_group_root(file_path: &str) -> CheckerResult {
    let mut result = CheckerResult::default();

    if let Ok(metadata) = fs::metadata(file_path) {
        if metadata.st_uid() != 0 || metadata.st_gid() != 0 {
            result.error = "File owner is not root:root".to_string();
            result.status = CheckStatus::FAIL;
        } else {
            result.status = CheckStatus::PASS;
        }
    } else {
        result.error = "unable to get file metadata".to_string();
    }

    result
}

#[cfg(test)]
mod test_utils {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use tempfile::NamedTempFile;

    use super::*;

    macro_rules! temp_file_path {
        ($path:expr) => {{
            $path
                .into_temp_path()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        }};
    }

    #[test]
    fn test_string_in_file_found() {
        let mut test_file = NamedTempFile::new().unwrap();
        writeln!(
            test_file,
            concat!(
                "udf 139264 0 - Live 0xffffffffc05e1000\n",
                "crc_itu_t 16384 1 udf, Live 0xffffffffc05dc000\n",
                "configfs 57344 1 - Live 0xffffffffc0320000\n"
            )
        )
        .unwrap();

        let found = look_for_string_in_file(temp_file_path!(test_file), " udf,").unwrap();
        assert!(found);
    }

    #[test]
    fn test_string_in_file_not_found() {
        let mut test_file = NamedTempFile::new().unwrap();
        writeln!(
            test_file,
            concat!(
                "crypto_simd 16384 1 aesni_intel, Live 0xffffffffc034f000\n",
                "cryptd 28672 2 ghash_clmulni_intel,crypto_simd, Live 0xffffffffc0335000\n",
                "configfs 57344 1 - Live 0xffffffffc0320000\n"
            )
        )
        .unwrap();

        let found = look_for_string_in_file(temp_file_path!(test_file), " udf,").unwrap();
        assert!(!found);
    }

    #[test]
    fn test_string_in_file_bad_path() {
        let result = look_for_string_in_file("/not/a/real/path", "search_str");
        assert!(result.is_err());
    }

    #[test]
    fn test_strings_in_file_found() {
        let mut test_file = NamedTempFile::new().unwrap();
        writeln!(
            test_file,
            concat!(
                "udf 139264 0 - Live 0xffffffffc05e1000\n",
                "crc_itu_t 16384 1 udf, Live 0xffffffffc05dc000\n",
                "configfs 57344 1 - Live 0xffffffffc0320000\n"
            )
        )
        .unwrap();

        let found =
            look_for_strings_in_file(temp_file_path!(test_file), &[" udf,", "57344 1"]).unwrap();
        assert!(found);
    }

    #[test]
    fn test_strings_in_file_found_one_line() {
        let mut test_file = NamedTempFile::new().unwrap();
        writeln!(test_file, "udf 139264 0 - Live 0xffffffffc05e1000, crc_itu_t 16384 1 udf, Live 0xffffffffc05dc000, configfs 57344 1 - Live 0xffffffffc0320000").unwrap();

        let found =
            look_for_strings_in_file(temp_file_path!(test_file), &[" udf,", "57344 1"]).unwrap();
        assert!(found);
    }

    #[test]
    fn test_strings_in_file_not_found() {
        let mut test_file = NamedTempFile::new().unwrap();
        writeln!(
            test_file,
            concat!(
                "crypto_simd 16384 1 aesni_intel, Live 0xffffffffc034f000\n",
                "cryptd 28672 2 ghash_clmulni_intel,crypto_simd, Live 0xffffffffc0335000\n",
                "configfs 57344 1 - Live 0xffffffffc0320000\n"
            )
        )
        .unwrap();

        let found =
            look_for_strings_in_file(temp_file_path!(test_file), &[" udf,", "57344 1"]).unwrap();
        assert!(!found);
    }

    #[test]
    fn test_strings_in_file_bad_path() {
        let result = look_for_strings_in_file("/not/a/real/path", &["foo", "search_str"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_in_output_found() {
        let cmd_output = "'insmod /lib/modules/5.15.90/kernel/drivers/cdrom/cdrom.ko.xz
        insmod /lib/modules/5.15.90/kernel/lib/crc-itu-t.ko.xz
        install /bin/true'";

        let found = look_for_string_in_output("echo", [cmd_output], "install /bin/true").unwrap();
        assert!(found);
    }

    #[test]
    fn test_string_in_output_not_found() {
        let cmd_output = "'insmod /lib/modules/5.15.90/kernel/fs/udf/udf.ko.xz'";

        let found = look_for_string_in_output("echo", [cmd_output], "install /bin/true").unwrap();
        assert!(!found);
    }

    #[test]
    fn test_string_in_output_bad_cmd() {
        let result = look_for_string_in_output("ekko", [""], "blah");
        assert!(result.is_none());
    }

    #[test]
    fn test_strings_in_output_found() {
        let cmd_output = "'insmod /lib/modules/5.15.90/kernel/drivers/cdrom/cdrom.ko.xz
        insmod /lib/modules/5.15.90/kernel/lib/crc-itu-t.ko.xz
        install /bin/true'";

        let found =
            look_for_strings_in_output("echo", [cmd_output], &["5.15.90", "install /bin/true"])
                .unwrap();
        assert!(found);
    }

    #[test]
    fn test_strings_in_output_not_found() {
        let cmd_output = "'insmod /lib/modules/5.15.90/kernel/fs/udf/udf.ko.xz'";

        let found =
            look_for_strings_in_output("echo", [cmd_output], &["5.15.90", "install /bin/true"])
                .unwrap();
        assert!(!found);
    }

    #[test]
    fn test_strings_in_output_bad_cmd() {
        let result = look_for_strings_in_output("ekko", [""], &["blah"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_file_not_mode() {
        // Generate a temp file that will be dropped as unused
        let test_file_path = NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .canonicalize()
            .unwrap()
            .display()
            .to_string();

        // Then use that name to create test file with the expected permission mode
        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o700)
            .open(&test_file_path)
            .unwrap();

        let result = check_file_not_mode(test_file_path.as_str(), 0b111111);
        assert_eq!(result.status, CheckStatus::PASS);

        // Clean up, fine to ignore errors
        let _ = fs::remove_file(test_file_path);
    }

    #[test]
    fn test_check_file_not_mode_wrong() {
        // Generate a temp file that will be dropped as unused
        let test_file_path = NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .canonicalize()
            .unwrap()
            .display()
            .to_string();

        // Then us that name to create test file with the expected (wrong) permission mode
        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o750)
            .open(&test_file_path)
            .unwrap();

        let result = check_file_not_mode(test_file_path.as_str(), 0b111111);
        assert_eq!(result.status, CheckStatus::FAIL);

        // Clean up, fine to ignore errors
        let _ = fs::remove_file(test_file_path);
    }

    #[test]
    fn test_check_file_not_mode_file_not_found() {
        let test_file_path = "/tmp/whatdoyouwant";

        let result = check_file_not_mode(test_file_path, 0b111111);
        assert_eq!(result.status, CheckStatus::SKIP);

        // Clean up, fine to ignore errors
        let _ = fs::remove_file(test_file_path);
    }
}
