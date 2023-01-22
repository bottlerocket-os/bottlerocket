use crate::set::PartitionSet;
use snafu::Snafu;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display(
        "Active partition {} not in either detected partition set ({:?})",
        active_partition.display(),
        sets
    ))]
    ActiveNotInSet {
        active_partition: PathBuf,
        sets: Vec<PartitionSet>,
    },

    #[snafu(display("Failed to get block device from path {}: {}", device.display(), source))]
    BlockDeviceFromPath {
        device: PathBuf,
        source: block_party::Error,
    },

    #[snafu(display("Failed to get disk from partition {}: {}", device.display(), source))]
    DiskFromPartition {
        device: PathBuf,
        source: block_party::Error,
    },

    #[snafu(display("Failed to get partition on disk {}: {}", device.display(), source))]
    PartitionFromDisk {
        device: PathBuf,
        source: block_party::Error,
    },

    #[snafu(display("Failed to find GPT on device {}: {}", device.display(), source))]
    GPTFind { device: PathBuf, source: GPTError },

    #[snafu(display("Failed to write GPT onto device {}: {}", device.display(), source))]
    GPTWrite { device: PathBuf, source: GPTError },

    #[snafu(display("Inactive partition {} is already marked for upgrade", inactive.display()))]
    InactiveAlreadyMarked { inactive: PathBuf },

    #[snafu(display("Inactive partition {} has not been marked valid for upgrade", inactive.display()))]
    InactiveNotValid { inactive: PathBuf },

    #[snafu(display(
        "Inactive partition is not valid to roll back to (priority={} tries_left={} successful={})",
        priority,
        tries_left,
        successful
    ))]
    InactiveInvalidRollback {
        priority: u64,
        tries_left: u64,
        successful: bool,
    },

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

    #[snafu(display("Root device {} has no lower root devices", root.display()))]
    RootHasNoLowerDevices { root: PathBuf },

    #[snafu(display("Failed to get lower devices for {}: {}", root.display(), source))]
    RootLowerDevices {
        root: PathBuf,
        source: block_party::Error,
    },

    #[snafu(display("Block device {} is not a partition", device.display()))]
    RootNotPartition { device: PathBuf },
}

#[derive(Debug)]
pub struct GPTError(pub gptman::Error);

impl fmt::Display for GPTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for GPTError {}
