use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Failed to canonicalize link for '{}': {}", path.display(), source))]
    CanonicalizeLink {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to find block device for '{}': {}", path.display(), source))]
    FindBlockDevice {
        path: std::path::PathBuf,
        source: block_party::Error,
    },

    #[snafu(display("Failed to find disk for '{}': {}", path.display(), source))]
    FindDisk {
        path: std::path::PathBuf,
        source: block_party::Error,
    },

    #[snafu(display("Expected partition for '{}'", path.display()))]
    NotPartition { path: std::path::PathBuf },

    #[snafu(display("Failed to open '{}': {}", path.display(), source))]
    DeviceOpen {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to stat '{}': {}", path.display(), source))]
    DeviceStat {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to read partition table from '{}': {}", path.display(), source))]
    ReadPartitionTable {
        path: std::path::PathBuf,
        source: gptman::Error,
    },

    #[snafu(display("Found {} partitions on '{}', can only resize with 1", count, path.display()))]
    MultiplePartitions {
        path: std::path::PathBuf,
        count: usize,
    },

    #[snafu(display("Failed to write partition table to '{}': {}", path.display(), source))]
    WritePartitionTable {
        path: std::path::PathBuf,
        source: gptman::Error,
    },

    #[snafu(display("Failed to reload partition table from '{}': {}", path.display(), source))]
    ReloadPartitionTable {
        path: std::path::PathBuf,
        source: gptman::linux::BlockError,
    },

    #[snafu(display("Failed to remove partition {} from '{}': {}", part, path.display(), source))]
    RemovePartition {
        part: u32,
        path: std::path::PathBuf,
        source: gptman::Error,
    },

    #[snafu(display("Failed to find maximum partition size for '{}': {}", path.display(), source))]
    FindMaxSize {
        path: std::path::PathBuf,
        source: gptman::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
