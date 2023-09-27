pub mod error;
mod source_processor;

use clap::Parser;
use snafu::ResultExt;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub(crate) struct SourcePathArgs {
    /// The base path of the Bottlerocket repo.
    #[clap(short, long = "repo-root")]
    repo_root: PathBuf,
    #[clap(short, long)]
    variants: Vec<String>,
    /// The output directory to write files for each variant.
    #[clap(short, long = "output-path")]
    output_path: PathBuf,
}

impl SourcePathArgs {
    /// Parse the variants dependency chain to get a list of repo source paths.
    pub(crate) fn run(self) -> Result<()> {
        // Make sure we have valid paths
        let repo_root = self
            .repo_root
            .canonicalize()
            .context(error::InvalidRepoRootSnafu {
                path: &self.repo_root,
            })?;

        // Make sure the output directory exists.
        create_dir_all(&self.output_path).context(error::InvalidOutputDirSnafu {
            path: &self.output_path,
        })?;
        let output_path =
            self.output_path
                .canonicalize()
                .context(error::InvalidOutputDirSnafu {
                    path: &self.output_path,
                })?;

        let root_length = repo_root.to_string_lossy().len() + 1;
        let processor = source_processor::SourceProcessor { repo_root };

        // Parse the local `source` paths to get a look up for packages that refer to them by name
        let mut source_cache = processor.source_cache();

        // Then get our variant package information
        for variant in &self.variants {
            let variant_path = self
                .repo_root
                .join("variants")
                .join(variant)
                .join("Cargo.toml")
                .canonicalize()
                .context(error::VariantNotFoundSnafu {
                    variant: variant.to_string(),
                })?;
            let variant_info = processor.process_variant(variant_path, &mut source_cache)?;

            // Write output with sanitized repo file paths
            let output_file = output_path.join(variant);
            let mut file = File::create(&output_file)
                .context(error::OutputFileWriteSnafu { path: &output_file })?;
            let variant_paths = variant_info.local_dependency_paths(true);
            for p in &variant_paths {
                writeln!(file, "{}", &p[root_length..])
                    .context(error::OutputFileWriteSnafu { path: &output_file })?;
            }
        }
        Ok(())
    }
}

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
