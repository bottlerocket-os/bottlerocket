/*!
This small lib is used to generate README files for the crates in the `sources` workspace. These
functions are called in a crate's build.rs file to generate a README from Rust doc comments.
!*/

use snafu::ResultExt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, error::Error>;

pub mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Unable to create the 'README.md' file: {}", source))]
        ReadmeCreate { source: std::io::Error },

        #[snafu(display("Unable to generate the 'README.md' file contents: {}", error))]
        ReadmeGenerate { error: String },

        #[snafu(display("Unable to open '{}': {}", file.display(), source))]
        ReadmeSourceOpen {
            file: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Unable to open 'README.tpl': {}", source))]
        ReadmeTemplateOpen { source: std::io::Error },

        #[snafu(display("Unable to write to the 'README.md' file: {}", source))]
        ReadmeWrite { source: std::io::Error },
    }
}

/// When this function is called in a `build.rs` file, it will generate a `README.md` (as a sibling
/// to `build.rs`). It uses the doc comments found in `src/main.rs` and the `cargo-readme` crate to
/// do so. The template for `cargo-readme` is expected to be `README.tpl` as a sibling file to
/// `build.rs`.
pub fn from_main() -> Result<()> {
    from_file("src/main.rs")
}

/// When this function is called in a `build.rs` file, it will generate a `README.md` (as a sibling
/// to `build.rs`). It uses the doc comments found in `src/lib.rs` and the `cargo-readme` crate to
/// do so. The template for `cargo-readme` is expected to be `README.tpl` as a sibling file to
/// `build.rs`.
pub fn from_lib() -> Result<()> {
    from_file("src/lib.rs")
}

/// When this function is called in a `build.rs` file, it will generate a `README.md` (as a sibling
/// to `build.rs`). It uses the doc comments found in `rust_file` and the `cargo-readme` crate to do
/// so. The template for `cargo-readme` is expected to be `README.tpl` as a sibling file to
/// `build.rs`.
pub fn from_file<P>(rust_file: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Check for environment variable "SKIP_README". If it is set,
    // skip README generation
    if std::env::var_os("SKIP_README").is_some() {
        return Ok(());
    }

    let mut source = File::open(rust_file.as_ref()).context(error::ReadmeSourceOpenSnafu {
        file: rust_file.as_ref(),
    })?;
    let mut template = File::open("README.tpl").context(error::ReadmeTemplateOpenSnafu)?;

    let mut content = cargo_readme::generate_readme(
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

    // Make sure the end of the file has a newline
    if content.chars().last().unwrap_or_default() != '\n' {
        content += "\n";
    }

    let mut readme = File::create("README.md").context(error::ReadmeCreateSnafu)?;
    readme
        .write_all(content.as_bytes())
        .context(error::ReadmeWriteSnafu)?;
    Ok(())
}
