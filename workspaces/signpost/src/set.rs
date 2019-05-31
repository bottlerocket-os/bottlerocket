use std::fmt;
use std::ops::Not;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct PartitionSet<T> {
    /// The partition containing the kernel and GRUB configuration for this partition set.
    pub(crate) boot: T,
    /// The partition containing the root filesystem for this partition set.
    pub(crate) root: T,
    /// The partition containing the dm-verity hashes for this partition set.
    pub(crate) hash: T,
}

impl<T: PartialEq> PartitionSet<T> {
    pub(crate) fn contains(&self, device: &T) -> bool {
        &self.boot == device || &self.root == device || &self.hash == device
    }
}

impl fmt::Display for PartitionSet<&Path> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "boot={} root={} hash={}",
            self.boot.display(),
            self.root.display(),
            self.hash.display(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SetSelect {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SetSelect::A => "A",
            SetSelect::B => "B",
        }
        .fmt(f)
    }
}
