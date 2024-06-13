use migration_helpers::common_migrations::{RemoveMetadataMigration, SettingMetadata};
use migration_helpers::{migrate, Result};
use std::process;
fn run() -> Result<()> {
    migrate(RemoveMetadataMigration(&[
        SettingMetadata {
            setting: "settings.kubernetes.max-pods",
            metadata: &["setting-generator"],
        },
        SettingMetadata {
            setting: "settings.kubernetes.cluster-dns-ip",
            metadata: &["setting-generator"],
        },
        SettingMetadata {
            setting: "settings.kubernetes.node-ip",
            metadata: &["setting-generator"],
        },
        SettingMetadata {
            setting: "settings.kubernetes.provider-id",
            metadata: &["setting-generator"],
        },
        SettingMetadata {
            setting: "settings.kubernetes.hostname-override",
            metadata: &["setting-generator"],
        },
    ]))
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
