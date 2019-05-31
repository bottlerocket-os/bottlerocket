use crate::error::{self, Error};
use snafu::{ensure, OptionExt, ResultExt};
use std::convert::TryFrom;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

fn sys_path(major: u64, minor: u64) -> PathBuf {
    PathBuf::from("/sys/dev/block").join(format!("{}:{}", major, minor))
}

/// A Linux block device with a major and minor number.
#[derive(Debug)]
pub(crate) struct BlockDevice {
    major: u64,
    minor: u64,
    device_name: OsString,
}

impl BlockDevice {
    fn new(major: u64, minor: u64) -> Result<Self, Error> {
        let path = sys_path(major, minor);
        let link_target = fs::read_link(&path).context(error::ReadLink { path: &path })?;
        let device_name = link_target
            .file_name()
            .context(error::LinkWithoutFinalComponent {
                path: &path,
                link_target: &link_target,
                expected: "a device name",
            })?
            .to_owned();

        Ok(Self {
            major,
            minor,
            device_name,
        })
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_str(&fs::read_to_string(path.as_ref()).context(error::ReadFile {
            path: path.as_ref(),
        })?)
    }

    fn sys_path(&self) -> PathBuf {
        sys_path(self.major, self.minor)
    }

    /// Get the path in `/dev` to the block device.
    pub(crate) fn path(&self) -> PathBuf {
        PathBuf::from("/dev").join(&self.device_name)
    }

    /// If this device is a partition, get the disk it belongs to.
    #[allow(clippy::identity_conversion)] // https://github.com/rust-lang/rust-clippy/issues/4133
    pub(crate) fn disk(&self) -> Result<Self, Error> {
        for entry in fs::read_dir("/sys/block").context(error::ReadDir { path: "/sys/block" })? {
            let entry = entry.context(error::ReadDir { path: "/sys/block" })?;
            if entry.path().join(&self.device_name).exists() {
                return Self::from_file(entry.path().join("dev"));
            }
        }

        error::NoBlockDeviceForPartition {
            device_name: &self.device_name,
        }
        .fail()
    }

    /// If this device is a disk, get one of its partitions by number.
    #[allow(clippy::identity_conversion)] // https://github.com/rust-lang/rust-clippy/issues/4133
    pub(crate) fn partition(&self, part_num: u32) -> Result<Option<Self>, Error> {
        let sys_path = self.sys_path();
        for entry in fs::read_dir(&sys_path).context(error::ReadDir { path: &sys_path })? {
            let entry = entry.context(error::ReadDir { path: &sys_path })?;
            if entry.path().is_dir() {
                let partition_path = entry.path().join("partition");
                let partition_str = match fs::read_to_string(&partition_path) {
                    Ok(s) => s,
                    Err(err) => match err.kind() {
                        ErrorKind::NotFound => continue,
                        _ => Err(err).context(error::ReadFile {
                            path: partition_path,
                        })?,
                    },
                };
                let partition_str = partition_str.trim();
                let partition = u32::from_str(partition_str)
                    .context(error::PartitionParseInt { s: partition_str })?;
                if partition == part_num {
                    return Self::from_file(entry.path().join("dev")).map(Some);
                }
            }
        }
        Ok(None)
    }

    /// An iterator over the lower devices that make up this device.
    ///
    /// For example, given a dm-verity device, this iterator would return the data device and the
    /// hash device.
    pub(crate) fn lower_devices(&self) -> impl Iterator<Item = Result<Self, Error>> {
        let lower_path = self.sys_path().join("slaves");
        match fs::read_dir(&lower_path).context(error::ReadDir { path: &lower_path }) {
            Ok(iter) => Box::new(iter.map(move |entry_result| {
                let entry = entry_result.context(error::ReadDir { path: &lower_path })?;
                Self::from_file(entry.path().join("dev"))
            })) as Box<dyn Iterator<Item = Result<Self, Error>>>,
            Err(err) => Box::new(vec![Err(err)].into_iter())
                as Box<dyn Iterator<Item = Result<Self, Error>>>,
        }
    }
}

impl fmt::Display for BlockDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.device_name.to_string_lossy().fmt(f)
    }
}

impl PartialEq for BlockDevice {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }
}

impl TryFrom<fs::Metadata> for BlockDevice {
    type Error = Error;

    fn try_from(metadata: fs::Metadata) -> Result<Self, Error> {
        // see /usr/include/linux/kdev_t.h
        let major = metadata.dev() >> 8;
        let minor = metadata.dev() & 0xff;
        Ok(Self::new(major, minor)?)
    }
}

impl FromStr for BlockDevice {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let parts = s
            .trim()
            .splitn(2, ':')
            .map(u64::from_str)
            .collect::<Result<Vec<_>, _>>()
            .context(error::MajorMinorParseInt { s })?;
        ensure!(parts.len() == 2, error::MajorMinorLen { s });
        Self::new(parts[0], parts[1])
    }
}
