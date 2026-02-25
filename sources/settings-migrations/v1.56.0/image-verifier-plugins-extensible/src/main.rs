use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const PREFIX: &str = "settings.image-verifier-plugins.";
/// Keys known to the old model that should be preserved on downgrade.
const KNOWN_KEYS: &[&str] = &["enabled", "notation"];

/// Image verifier plugins changed from a fixed `notation` field to an extensible plugin map.
/// On downgrade, remove any plugin keys that the old model doesn't recognize.
pub struct ImageVerifierPluginsExtensible;

impl Migration for ImageVerifierPluginsExtensible {
    /// New model is a superset of the old; existing data is compatible.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("ImageVerifierPluginsExtensible has no work to do on upgrade.");
        Ok(input)
    }

    /// Remove plugin keys that older versions don't understand.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let keys: Vec<String> = input
            .data
            .keys()
            .filter(|k| {
                k.starts_with(PREFIX)
                    && !KNOWN_KEYS.iter().any(|known| {
                        let rest = &k[PREFIX.len()..];
                        rest == *known || rest.starts_with(&format!("{known}."))
                    })
            })
            .cloned()
            .collect();

        for key in keys {
            if let Some(data) = input.data.remove(&key) {
                println!("Removed {key}, which was set to '{data}'");
            }
        }

        Ok(input)
    }
}

fn run() -> Result<()> {
    migrate(ImageVerifierPluginsExtensible)
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
