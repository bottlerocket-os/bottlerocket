use super::{error, Result};
use snafu::ResultExt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Whether the [`generate_readme`] function should take its doc comments from `lib.rs`, `main.rs`,
/// or somewhere else.
#[derive(Debug, Copy, Clone)]
pub enum ReadmeSource {
    /// The README text is taken from `src/lib.rs`.
    Lib,

    /// The README text is taken from `src/main.rs`.
    Main,

    /// The README text is taken from the file at the given path.
    Other(&'static str),
}

impl Display for ReadmeSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadmeSource::Lib => Display::fmt("src/lib.rs", f),
            ReadmeSource::Main => Display::fmt("src/main.rs", f),
            ReadmeSource::Other(s) => Display::fmt(s, f),
        }
    }
}

/// When this function is called in a `build.rs` file, it will generate a `README.md` (as a sibling
/// to `build.rs`). It uses the `cargo-readme` crate to do so. It takes the doc comment from either
/// `src/lib.rs`, `src/main.rs`, or any file you choose (depending on the value of `source`).
/// The template for `cargo-readme` is expected to be `README.tpl` as a sibling file to `build.rs`.
pub fn generate_readme(source: ReadmeSource) -> Result<()> {
    // Check for environment variable "SKIP_README". If it is set,
    // skip README generation
    if std::env::var_os("SKIP_README").is_some() {
        return Ok(());
    }

    let mut source =
        File::open(source.to_string()).context(error::ReadmeSourceOpenSnafu { file: source })?;
    let mut template = File::open("README.tpl").context(error::ReadmeTemplateOpenSnafu)?;

    let content = cargo_readme::generate_readme(
        &PathBuf::from("."), // root
        &mut source,         // source
        Some(&mut template), // template
        // The "add x" arguments don't apply when using a template.
        true,  // add title
        false, // add badges
        false, // add license
        true,  // indent headings
    )
    .map_err(|e| error::ReadmeGenerateSnafu { error: e }.build())?;

    let mut readme = File::create("README.md").context(error::ReadmeCreateSnafu)?;
    readme
        .write_all(content.as_bytes())
        .context(error::ReadmeWriteSnafu)?;
    Ok(())
}
