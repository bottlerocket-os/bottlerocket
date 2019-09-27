use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Copylike {
    Copy,
    Hardlink,
    Symlink,
}

impl Copylike {
    pub(crate) fn run<P: AsRef<Path>, Q: AsRef<Path>>(self, src: P, dst: Q) -> io::Result<()> {
        if let Some(parent) = dst.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        // unlinking the file is required before symlink/hardlink
        if let Err(err) = fs::remove_file(&dst) {
            if err.kind() != io::ErrorKind::NotFound {
                return Err(err);
            }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[cfg(test)]
mod tests {
    use super::Copylike;
    use std::fs;
    use std::io;

    #[test]
    fn clobber() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let a = dir.path().join("a");
        let b = dir.path().join("b");

        {
            fs::File::create(&a)?;
            fs::File::create(&b)?;
        }

        for copy_action in &[Copylike::Copy, Copylike::Hardlink, Copylike::Symlink] {
            eprintln!("{:?}", copy_action);
            let target = dir.path().join(copy_action.to_string());
            copy_action.run(&a, &target)?;
            // copying the same file should work
            copy_action.run(&a, &target)?;
            // copying a different file should clobber
            copy_action.run(&b, &target)?;
        }

        Ok(())
    }
}
