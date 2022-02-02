//! Provides a function for compressing a directory's contents into a tarball.

use crate::error::{self, Result};
use std::fs::{self, File};
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use snafu::{ensure, OptionExt, ResultExt};

/// Creates a tarball with all the contents of directory `dir`.
pub(crate) fn create_tarball<P1, P2>(indir: P1, outfile: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let indir = indir.as_ref();
    let outfile = outfile.as_ref();

    // ensure the output directory exists.
    let outdir = outfile.parent().context(error::RootAsFileSnafu)?;
    fs::create_dir_all(outdir).context(error::CreateOutputDirectorySnafu { path: outdir })?;

    // ensure the outfile will not be written to the input dir.
    ensure!(
        indir != outdir,
        error::TarballOutputIsInInputDirSnafu { indir, outfile }
    );

    // compress files and create the tarball.
    let tarfile = File::create(outfile).context(error::TarballFileCreateSnafu { path: outfile })?;
    let encoder = GzEncoder::new(tarfile, Compression::default());
    let mut tarball = tar::Builder::new(encoder);
    tarball
        .append_dir_all(crate::TARBALL_DIRNAME, indir)
        .context(error::TarballWriteSnafu { path: outfile })?;
    tarball
        .finish()
        .context(error::TarballCloseSnafu { path: indir })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use std::path::{Path, PathBuf};

    use flate2::read::GzDecoder;
    use tar::Archive;
    use tempfile::TempDir;

    #[test]
    fn tarball_test() {
        // create an input directory with one file in it.
        let indir = TempDir::new().unwrap();
        let mut file = File::create(indir.path().to_path_buf().join("hello.txt")).unwrap();
        file.write_all(b"Hello World!").unwrap();
        drop(file);

        // create an output directory into which our function will produce a tarball.
        let outdir = TempDir::new().unwrap();
        let outfilepath = outdir.path().join("somefile.tar.gz");

        // run the function under test.
        create_tarball(&indir.path().to_path_buf(), &outfilepath).unwrap();

        // assert that the output tarball exists.
        assert!(Path::new(&outfilepath).is_file());

        // open the output tarball and check that it has the expected top level directory in it.
        let tar_gz = File::open(outfilepath).unwrap();
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        let mut entries = archive.entries().unwrap();
        let entry = entries.next().unwrap().unwrap();
        let actual_path = PathBuf::from(entry.path().unwrap());
        let expected_path = PathBuf::from(crate::TARBALL_DIRNAME);
        assert!(actual_path == expected_path);

        // check that the tarball also contains our hello.txt file.
        let entry = entries.next().unwrap().unwrap();
        let actual_path = PathBuf::from(entry.path().unwrap());
        let expected_path = PathBuf::from(crate::TARBALL_DIRNAME).join("hello.txt");
        assert!(actual_path == expected_path);
    }
}
