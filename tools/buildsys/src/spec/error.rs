use snafu::Snafu;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(crate) enum Error {
    #[snafu(display("Failed to read spec file '{}': {}", path.display(), source))]
    SpecFileRead { path: PathBuf, source: io::Error },
}

pub(super) type Result<T> = std::result::Result<T, Error>;
