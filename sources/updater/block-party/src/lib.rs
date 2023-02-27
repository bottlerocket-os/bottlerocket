//! block-party is a library for getting information about Linux block devices.
//!
//! It supports:
//!
//! * Getting the disk for a partition device
//! * Getting a numbered partition on a disk
//! * Getting the devices that are combined as a block device, e.g. a dm-verity device

#![deny(missing_docs)]

use snafu::{ensure, OptionExt, ResultExt};
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    /// The error type for this library.
    pub enum Error {
        #[snafu(display("Target of {} ends in `..`", path.display()))]
        /// The target of a link ends in `..`
        LinkTargetFileName {
            /// Contains the invalid link path.
            path: PathBuf,
        },

        #[snafu(display("Cannot parse {} as major/minor numbers: {}", path.display(), source))]
        /// Can't parse the given path as major/minor numbers.
        MajorMinorParseInt {
            /// Contains the path we failed to parse as major/minor.
            path: PathBuf,
            /// The source error describing the parse failure.
            source: std::num::ParseIntError,
        },

        #[snafu(display(
                "Cannot parse {} as major/minor numbers: invalid number of colons",
                path.display())
            )]
        /// Can't parse the given string as major/minor numbers because it has an invalid number
        /// of colons.
        MajorMinorLen {
            /// Contains the path which in turn contains an invalid major/minor string.
            path: PathBuf,
        },

        #[snafu(display("Unable to read device name through link at {}: {} ", path.display(), source))]
        /// Unable to read device name through the given link.
        SysPathLinkRead {
            /// Contains the path we failed to read.
            path: PathBuf,
            /// The source error describing the read failure.
            source: io::Error,
        },

        #[snafu(display("Unable to read filesystem metadata of {}: {} ", path.display(), source))]
        /// Unable to read filesystem metadata of a given path.
        PathMetadata {
            /// Contains the path for which we failed to read metadata.
            path: PathBuf,
            /// The source error describing the read failure.
            source: io::Error,
        },

        #[snafu(display("Unable to read file {}: {} ", path.display(), source))]
        /// Unable to read a given file.
        FileRead {
            /// Contains the path we failed to read.
            path: PathBuf,
            /// The source error describing the read failure.
            source: io::Error,
        },

        #[snafu(display("Unable to list directory {}: {} ", path.display(), source))]
        /// Unable to list a given directory.
        ListDirectory {
            /// Contains the directory we failed to list.
            path: PathBuf,
            /// The source error describing the list failure.
            source: io::Error,
        },

        #[snafu(display("Unable to read directory entry in {}: {} ", path.display(), source))]
        /// Unable to read a listed directory entry.
        ReadDirectoryEntry {
            /// Contains the directory with an entry we failed to read.
            path: PathBuf,
            /// The source error describing the read failure.
            source: io::Error,
        },
    }
}
pub use error::Error;
/// Convenience alias pointing to our Error type.
pub type Result<T> = std::result::Result<T, error::Error>;

/// Get the path in `/sys/dev/block` for a major/minor number.
fn sys_path(major: u64, minor: u64) -> PathBuf {
    PathBuf::from("/sys/dev/block").join(format!("{}:{}", major, minor))
}

/// A Linux block device with a major and minor number.
#[derive(Debug, Clone)]
pub struct BlockDevice {
    major: u64,
    minor: u64,
    device_name: OsString,
}

impl BlockDevice {
    /// Creates a `BlockDevice` for a major/minor number.
    pub fn from_major_minor(major: u64, minor: u64) -> Result<Self> {
        let path = sys_path(major, minor);
        let link_target = fs::read_link(&path).context(error::SysPathLinkReadSnafu { path })?;
        let device_name = link_target
            .file_name()
            .context(error::LinkTargetFileNameSnafu { path: &link_target })?
            .to_owned();

        Ok(Self {
            major,
            minor,
            device_name,
        })
    }

    /// Creates a `BlockDevice` from a path residing on a block device.
    pub fn from_device_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path).context(error::PathMetadataSnafu { path })?;
        let major = metadata.st_dev() >> 8;
        let minor = metadata.st_dev() & 0xff;
        Self::from_major_minor(major, minor)
    }

    /// Creates a `BlockDevice` from a special block device node.
    pub fn from_device_node<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path).context(error::PathMetadataSnafu { path })?;
        let major = metadata.st_rdev() >> 8;
        let minor = metadata.st_rdev() & 0xff;
        Self::from_major_minor(major, minor)
    }

    /// Creates a `BlockDevice` from the major:minor string from the file at `path`.
    fn from_major_minor_in_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let s = fs::read_to_string(path).context(error::FileReadSnafu { path })?;
        let parts = s
            .trim()
            .splitn(2, ':')
            .map(u64::from_str)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context(error::MajorMinorParseIntSnafu { path })?;

        ensure!(parts.len() == 2, error::MajorMinorLenSnafu { path });

        Self::from_major_minor(parts[0], parts[1])
    }

    /// Get the path in `/sys/dev/block` for this device.
    fn sys_path(&self) -> PathBuf {
        sys_path(self.major, self.minor)
    }

    /// Returns the path in `/dev` to the block device.
    pub fn path(&self) -> PathBuf {
        PathBuf::from("/dev").join(&self.device_name)
    }

    /// If this device is a partition, get the disk it belongs to. Returns `Ok(None)` if this
    /// device is not a partition.
    //#[allow(clippy::identity_conversion)] // https://github.com/rust-lang/rust-clippy/issues/4133
    pub fn disk(&self) -> Result<Option<Self>> {
        // Globbing for /sys/block/*/{self.device_name}/dev
        for entry in
            fs::read_dir("/sys/block").context(error::ListDirectorySnafu { path: "/sys/block" })?
        {
            let entry = entry.context(error::ReadDirectoryEntrySnafu { path: "/sys/block" })?;
            if entry.path().join(&self.device_name).exists() {
                return Self::from_major_minor_in_file(entry.path().join("dev")).map(Some);
            }
        }

        Ok(None)
    }

    /// If this device is a disk, get one of its partitions by number.
    ///
    /// This fails if the device is not a disk, and returns `Ok(None)` if this device is a disk
    /// but there is no partition of that number.
    //#[allow(clippy::identity_conversion)] // https://github.com/rust-lang/rust-clippy/issues/4133
    pub fn partition(&self, part_num: u32) -> Result<Option<Self>> {
        let sys_path = self.sys_path();
        // Globbing for /sys/dev/block/{major}:{minor}/*/partition
        for entry in
            fs::read_dir(&sys_path).context(error::ListDirectorySnafu { path: &sys_path })?
        {
            let entry = entry.context(error::ReadDirectoryEntrySnafu { path: &sys_path })?;
            if entry.path().is_dir() {
                let partition_path = entry.path().join("partition");
                let partition_str = match fs::read_to_string(&partition_path) {
                    Ok(s) => s,
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => continue,
                        _ => {
                            return Err(err).context(error::FileReadSnafu {
                                path: partition_path,
                            })
                        }
                    },
                };
                if partition_str.trim() == part_num.to_string() {
                    return Self::from_major_minor_in_file(entry.path().join("dev")).map(Some);
                }
            }
        }
        Ok(None)
    }

    /// An iterator over the lower devices that make up this device.
    ///
    /// For example, given a dm-verity device, this iterator would return the data device and the
    /// hash device.
    pub fn lower_devices(&self) -> Result<LowerIter> {
        let path = self.sys_path().join("slaves");
        fs::read_dir(&path)
            .context(error::ListDirectorySnafu { path: &path })
            .map(move |iter| LowerIter { path, iter })
    }
}

impl fmt::Display for BlockDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.device_name.to_string_lossy().fmt(f)
    }
}

impl PartialEq for BlockDevice {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }
}

/// Iterator over lower devices.
///
/// This struct is created by [`BlockDevice::lower_devices`].
pub struct LowerIter {
    path: PathBuf,
    iter: fs::ReadDir,
}

impl Iterator for LowerIter {
    type Item = Result<BlockDevice>;

    fn next(&mut self) -> Option<Result<BlockDevice>> {
        self.iter.next().map(|entry| {
            let entry = entry.context(error::ReadDirectoryEntrySnafu { path: &self.path })?;
            BlockDevice::from_major_minor_in_file(entry.path().join("dev"))
        })
    }
}
