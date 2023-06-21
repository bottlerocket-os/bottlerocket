mod config;
mod devices;

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Unable to create '{}', missing name or MAC", what))]
        ConfigMissingName { what: String },

        #[snafu(display("Unable to write {} to {}: {}", what, path.display(), source))]
        NetworkDConfigWrite {
            what: String,
            path: PathBuf,
            source: io::Error,
        },
    }
}
pub(crate) type Result<T> = std::result::Result<T, error::Error>;
