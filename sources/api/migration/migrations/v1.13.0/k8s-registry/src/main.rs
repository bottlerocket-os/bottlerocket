use migration_helpers::common_migrations::ReplaceStringMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_K8S_PAUSE_IMAGE: &str = "k8s.gcr.io/pause:3.2";
const NEW_K8S_PAUSE_IMAGE: &str = "registry.k8s.io/pause:3.2";

// The `k8s.gcr.io registry`, as of April 2023 will be frozen and
// images will no longer be pushed to that registry.
// Instead, Kubernetes consumers are expected to use `registry.k8s.io`
// For further details: https://kubernetes.io/blog/2023/02/06/k8s-gcr-io-freeze-announcement/
//
// In this migration, we move image references from k8s.gcr.io to registry.k8s.io
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "settings.kubernetes.pod-infra-container-image",
        old_val: OLD_K8S_PAUSE_IMAGE,
        new_val: NEW_K8S_PAUSE_IMAGE,
    })
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
