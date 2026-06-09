use migration_helpers::common_migrations::{AddPrefixesMigration, NoOpMigration};
use migration_helpers::{migrate, MigrationData, Result};
use std::process;

/// Added new image-verifier-plugins settings.
/// For k8s variants: remove the settings on downgrade since they didn't exist before.
/// For ecs-3 variants: no migration needed since these settings already exist.
fn run() -> Result<()> {
    // Create a custom migration that checks variant at runtime
    migrate(VariantSpecificMigration)
}

struct VariantSpecificMigration;

impl migration_helpers::Migration for VariantSpecificMigration {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        // No work needed on upgrade for any variant
        println!("VariantSpecificMigration has no work to do on upgrade.");
        Ok(input)
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        // Check variant from runtime data
        if let Some(variant_value) = input.data.get("os.variant_id") {
            if let Some(variant_str) = variant_value.as_str() {
                if variant_str.starts_with("aws-ecs-") {
                    // For ECS variants, no migration needed
                    println!(
                        "ECS variant detected ({}), no migration needed",
                        variant_str
                    );
                    return NoOpMigration.backward(input);
                }
            }
        }

        println!("Using default behavior (remove settings on downgrade)");
        AddPrefixesMigration(vec!["settings.image-verifier-plugins"]).backward(input)
    }
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
