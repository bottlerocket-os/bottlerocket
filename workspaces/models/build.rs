// The src/ directory is a link to the API model we actually want to build; this build.rs creates
// that symlink based on the VARIANT environment variable, which either comes from the build
// system or the user, if doing a local `cargo build`.
//
// See README.md to understand the symlink setup.

use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process;

fn symlink_force<P1, P2>(target: P1, link: P2) -> io::Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // Remove link if it already exists
    if let Err(e) = fs::remove_file(&link) {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(e);
        }
    }
    // Link to requested target
    symlink(&target, &link)
}

fn main() {
    // The VARIANT variable is originally BUILDSYS_VARIANT, set in the top-level Makefile.toml,
    // and is passed through as VARIANT by the top-level Dockerfile.  It represents which OS
    // variant we're building, and therefore which API model to use.
    let var = "VARIANT";
    println!("cargo:rerun-if-env-changed={}", var);
    let variant = env::var(var).unwrap_or_else(|_| {
        eprintln!("For local builds, you must set the {} environment variable so we know which API model to build against.  Valid values are the directories in workspaces/models, for example \"aws-k8s\".", var);
        process::exit(1);
    });

    // Point to source directory for requested variant
    let link = "current/src";
    let target = format!("../{}", variant);

    // Make sure requested variant exists
    // (note: the "../" in `target` is because the link goes into `current/` - we're checking at
    // the same level here
    if !Path::new(&variant).exists() {
        eprintln!("The environment variable {} should refer to a directory under workspaces/models with an API model, but it's set to '{}' which doesn't exist", var, variant);
        process::exit(1);
    }

    // Create the symlink for the following `cargo build` to use for its source code
    symlink_force(&target, link).unwrap_or_else(|e| {
        eprintln!("Failed to create symlink at '{}' pointing to '{}' - we need this to support different API models for different variants.  Error: {}", link, target, e);
        process::exit(1);
    });
}
