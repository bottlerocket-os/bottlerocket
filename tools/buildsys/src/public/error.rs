use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display(
        "The 'VARIANT' environment variable is missing or unable to be read: {}",
        source
    ))]
    VariantEnv { source: std::env::VarError },

    #[snafu(display("The '{}' segment of the variant '{}' is missing", part_name, variant))]
    VariantPart { part_name: String, variant: String },

    #[snafu(display("The '{}' segment of the variant '{}' is empty", part_name, variant))]
    VariantPartEmpty { part_name: String, variant: String },
}
