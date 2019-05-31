use snafu::Snafu;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Active partition not in either detected partition set"))]
    ActiveNotInSet,
    #[snafu(display("Failed to find GPT on device {}: {}", device.display(), source))]
    GPTFind { device: PathBuf, source: GPTError },
    #[snafu(display("Failed to write GPT onto device {}: {}", device.display(), source))]
    GPTWrite { device: PathBuf, source: GPTError },
    #[snafu(display(
        "Path {} is a link to {} which does not have a final component (expected {})",
        path.display(),
        link_target.display(),
        expected
    ))]
    LinkWithoutFinalComponent {
        path: PathBuf,
        link_target: PathBuf,
        expected: &'static str,
    },
    #[snafu(display("Failed to parse major:minor integers from string {:?}: {}", s, source))]
    MajorMinorParseInt {
        s: String,
        source: std::num::ParseIntError,
    },
    #[snafu(display(
        "Failed to parse major:minor integers from string {:?}: does not have exactly one colon",
        s
    ))]
    MajorMinorLen { s: String },
    #[snafu(display("No block device with partition {} found", device_name.to_string_lossy()))]
    NoBlockDeviceForPartition { device_name: OsString },
    #[snafu(display("Failed to open {} for {}: {}", path.display(), what, source))]
    Open {
        path: PathBuf,
        what: &'static str,
        source: std::io::Error,
    },
    #[snafu(display("Failed to find {} partition for set {}", part_type, set))]
    PartitionMissingFromSet {
        part_type: &'static str,
        set: &'static str,
    },
    #[snafu(display("Failed to find device for partition {} on {}", num, device.display()))]
    PartitionNotFoundOnDevice { num: u32, device: PathBuf },
    #[snafu(display("Failed to parse partition number {:?} as integer: {}", s, source))]
    PartitionParseInt {
        s: String,
        source: std::num::ParseIntError,
    },
    #[snafu(display("Failed to read directory {}: {}", path.display(), source))]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Failed to read from file {}: {}", path.display(), source))]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Failed to read link {}: {}", path.display(), source))]
    ReadLink {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Root device {} has no lower root devices", root_major_minor))]
    RootHasNoLowerDevices { root_major_minor: String },
    #[snafu(display("Failed to stat {}: {}", path.display(), source))]
    Stat {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug)]
pub(crate) struct GPTError(pub gptman::Error);

impl fmt::Display for GPTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for GPTError {}
