use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub(crate) enum Error {
    #[snafu(display("Failed to walk directory to find project files: {}", source))]
    DirectoryWalk { source: walkdir::Error },
}

pub(super) type Result<T> = std::result::Result<T, Error>;
