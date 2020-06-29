use std::fmt;
use std::ops::Not;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PartitionSet {
    /// The partition containing the kernel and GRUB configuration for this partition set.
    pub boot: PathBuf,
    /// The partition containing the root filesystem for this partition set.
    pub root: PathBuf,
    /// The partition containing the dm-verity hashes for this partition set.
    pub hash: PathBuf,
}

impl PartitionSet {
    pub(crate) fn contains<P: AsRef<Path>>(&self, device: P) -> bool {
        self.boot == device.as_ref() || self.root == device.as_ref() || self.hash == device.as_ref()
    }
}

impl fmt::Display for PartitionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "boot={} root={} hash={}",
            self.boot.display(),
            self.root.display(),
            self.hash.display(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SetSelect {
    A,
    B,
}

impl SetSelect {
    pub(crate) fn idx(self) -> usize {
        match self {
            SetSelect::A => 0,
            SetSelect::B => 1,
        }
    }
}

impl Not for SetSelect {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            SetSelect::A => SetSelect::B,
            SetSelect::B => SetSelect::A,
        }
    }
}

impl fmt::Display for SetSelect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetSelect::A => "A",
            SetSelect::B => "B",
        }
        .fmt(f)
    }
}
