use std::io::{self, BufRead, BufReader};
use std::{ffi::OsStr, fs::File, process::Command};

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

/// Executes a command and checks if the give `search_str` is in the output.
pub fn look_for_string_in_output<I, S>(cmd: &str, args: I, search_str: &str) -> Option<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if let Ok(output) = Command::new(cmd).args(args).output() {
        if output.status.success() {
            let mut found = false;
            let mp_output = String::from_utf8_lossy(&output.stdout).to_string();
            for line in mp_output.lines() {
                found |= line.contains(search_str);
            }

            Some(found)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod test_utils {
    use std::io::Write;
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
}
