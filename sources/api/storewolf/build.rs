/// This build script generates README.md from rustdoc, like our other crates, but also generates
/// a unified TOML file representing the default settings for the system.  The contents of that
/// file are used by storewolf to populate the defaults into the data store on a new system.
///
/// The goal of generating the defaults file here is to allow variants to break up and share
/// groups of default settings, without having to ship those files in the OS image.  Specifically,
/// we read any number of files from a defaults.d directory in the variant's model directory and
/// merge later entries into earlier entries, so later files take precedence.
use bottlerocket_variant::Variant;
use merge_toml::merge_values;
use snafu::ResultExt;
use std::fs;
use std::path::Path;
use toml::{map::Map, Value};
use walkdir::WalkDir;

/// A variant stores its default settings in .toml files in this directory.  It can link to shared
/// files if desired.  Entries are sorted by filename, and later entries take precedence.
const DEFAULTS_DIR: &str = "../../models/src/variant/current/defaults.d";

fn main() -> Result<()> {
    generate_readme();
    generate_defaults_toml()?;

    // Reflect that we need to rerun if variant has changed to pick up the new default settings.
    Variant::rerun_if_changed();

    Ok(())
}

fn generate_readme() {
    generate_readme::from_main().unwrap();
}

/// Merge the variant's default settings files into a single TOML value.  The result is serialized
/// to a file in OUT_DIR for storewolf to read.
fn generate_defaults_toml() -> Result<()> {
    // Find TOML config files specified by the variant.
    let walker = WalkDir::new(DEFAULTS_DIR)
        .follow_links(true) // we expect users to link to shared files
        .min_depth(1) // only read files in defaults.d, not doing inheritance yet
        .max_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name())) // allow ordering by prefix
        .into_iter()
        .filter_entry(|e| e.file_name().to_string_lossy().ends_with(".toml")); // looking for TOML config

    // Merge the files into a single TOML value, in order.
    let mut defaults = Value::Table(Map::new());
    for entry in walker {
        let entry = entry.context(error::ListFilesSnafu { dir: DEFAULTS_DIR })?;

        // Reflect that we need to rerun if any of the default settings files have changed.
        println!("cargo:rerun-if-changed={}", entry.path().display());

        let data = fs::read_to_string(entry.path()).context(error::FileSnafu {
            op: "read",
            path: entry.path(),
        })?;
        let value =
            toml::from_str(&data).context(error::TomlDeserializeSnafu { path: entry.path() })?;
        merge_values(&mut defaults, &value).context(error::TomlMergeSnafu)?;
    }

    // Serialize to disk for storewolf to read.
    let data = toml::to_string(&defaults).context(error::TomlSerializeSnafu)?;
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set; are you not using cargo?");
    let path = Path::new(&out_dir).join("defaults.toml");
    fs::write(&path, data).context(error::FileSnafu { op: "write", path })?;

    Ok(())
}

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Failed to {} {}: {}", op, path.display(), source))]
        File {
            op: String,
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to list files in {}: {}", dir.display(), source))]
        ListFiles {
            dir: PathBuf,
            source: walkdir::Error,
        },

        #[snafu(display("{} is not valid TOML: {}", path.display(), source))]
        TomlDeserialize {
            path: PathBuf,
            source: toml::de::Error,
        },

        #[snafu(display("Failed to merge TOML: {}", source))]
        TomlMerge { source: merge_toml::Error },

        #[snafu(display("Failed to serialize default settings: {}", source))]
        TomlSerialize { source: toml::ser::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
