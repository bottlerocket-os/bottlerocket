use std::fmt::{self, Display};
use std::fs::{self};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Copylike {
    Copy,
    Hardlink,
    Symlink,
}

impl Copylike {
    pub(crate) fn run<P: AsRef<Path>, Q: AsRef<Path>>(self, src: P, dst: Q) -> std::io::Result<()> {
        if let Some(parent) = dst.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        match self {
            Copylike::Copy => fs::copy(src, dst).map(|_| ()),
            Copylike::Hardlink => fs::hard_link(src, dst),
            Copylike::Symlink => {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(src, dst)
                }

                #[cfg(windows)]
                {
                    std::os::windows::fs::symlink_file(src, dst)
                }
            }
        }
    }
}

impl Display for Copylike {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Copylike::Copy => "copy",
                Copylike::Hardlink => "hardlink",
                Copylike::Symlink => "symlink",
            }
        )
    }
}
