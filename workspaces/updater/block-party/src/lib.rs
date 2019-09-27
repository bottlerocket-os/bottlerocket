//! block-party is a library for getting information about Linux block devices.
//!
//! It supports:
//!
//! * Getting the disk for a partition device
//! * Getting a numbered partition on a disk
//! * Getting the devices that are combined as a block device, e.g. a dm-verity device

#![deny(missing_docs, rust_2018_idioms)]
#![warn(clippy::pedantic)]

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
        let link_target = fs::read_link(&path)?;
        let device_name = link_target
            .file_name()
            .ok_or_else(|| ErrorShim::LinkTargetFileName(&link_target))?
            .to_owned();

        Ok(Self {
            major,
            minor,
            device_name,
        })
    }

    /// Creates a `BlockDevice` from a path residing on a block device.
    pub fn from_device_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let metadata = fs::metadata(&path)?;
        let major = metadata.st_dev() >> 8;
        let minor = metadata.st_dev() & 0xff;
        Ok(Self::from_major_minor(major, minor)?)
    }

    /// Creates a `BlockDevice` from a special block device node.
    pub fn from_device_node<P: AsRef<Path>>(path: P) -> Result<Self> {
        let metadata = fs::metadata(&path)?;
        let major = metadata.st_rdev() >> 8;
        let minor = metadata.st_rdev() & 0xff;
        Ok(Self::from_major_minor(major, minor)?)
    }

    /// Creates a `BlockDevice` from the major:minor string from the file at `path`.
    fn from_major_minor_in_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = fs::read_to_string(path.as_ref())?;
        let parts = s
            .trim()
            .splitn(2, ':')
            .map(u64::from_str)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|err| ErrorShim::MajorMinorParseInt(path.as_ref(), err))?;
        if parts.len() != 2 {
            Err(ErrorShim::MajorMinorLen(path.as_ref()))?;
        }
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
        for entry in fs::read_dir("/sys/block")? {
            let entry = entry?;
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
        for entry in fs::read_dir(&sys_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let partition_path = entry.path().join("partition");
                let partition_str = match fs::read_to_string(&partition_path) {
                    Ok(s) => s,
                    Err(err) => match err.kind() {
                        ErrorKind::NotFound => continue,
                        _ => return Err(err),
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
        fs::read_dir(&self.sys_path().join("slaves")).map(|iter| LowerIter { iter })
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
    iter: fs::ReadDir,
}

impl Iterator for LowerIter {
    type Item = Result<BlockDevice>;

    fn next(&mut self) -> Option<Result<BlockDevice>> {
        self.iter
            .next()
            .map(|entry| BlockDevice::from_major_minor_in_file(entry?.path().join("dev")))
    }
}

enum ErrorShim<'a> {
    LinkTargetFileName(&'a Path),
    MajorMinorParseInt(&'a Path, std::num::ParseIntError),
    MajorMinorLen(&'a Path),
}

impl<'a> From<ErrorShim<'a>> for Error {
    fn from(err: ErrorShim<'_>) -> Self {
        match err {
            ErrorShim::LinkTargetFileName(path) => Self::new(
                ErrorKind::InvalidData,
                format!("target of {} ends in `..`", path.display()),
            ),
            ErrorShim::MajorMinorParseInt(path, err) => Self::new(
                ErrorKind::InvalidData,
                format!(
                    "cannot parse {} as major/minor numbers: {}",
                    path.display(),
                    err
                ),
            ),
            ErrorShim::MajorMinorLen(path) => Self::new(
                ErrorKind::InvalidData,
                format!(
                    "cannot parse {} as major/minor numbers: invalid number of colons",
                    path.display()
                ),
            ),
        }
    }
}
