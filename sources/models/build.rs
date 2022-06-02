// src/variant/current is a link to the API model we actually want to build; this build.rs creates
// that symlink based on the VARIANT environment variable, which either comes from the build
// system or the user, if doing a local `cargo build`.
//
// See README.md to understand the symlink setup.

use bottlerocket_variant::{Variant, VARIANT_ENV};
use filetime::{set_symlink_file_times, FileTime};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process;

/// We create a link from 'current' to the variant selected by the environment variable above.
const VARIANT_LINK: &str = "src/variant/current";

/// We create a link for the 'variant' module's mod.rs; this can't be checked into the repo because
/// the `src/variant` directory is a cache mount created by Docker before building a package.
/// This isn't variant-specific, so we can have a fixed link target.  The file has top-level
/// definitions that apply to all models, and defines a 'current' submodule (that Rust will be able
/// to find through the 'current' link mentioned above) and re-exports everything in 'current' so
/// that consumers of the model don't have to care what the current variant is.
const MOD_LINK: &str = "src/variant/mod.rs";
const MOD_LINK_TARGET: &str = "../variant_mod.rs";

fn main() {
    // The VARIANT variable is originally BUILDSYS_VARIANT, set in the top-level Makefile.toml,
    // and is passed through as VARIANT by the top-level Dockerfile. It represents which OS variant
    // we're building, and therefore which API model to use.
    let variant = match Variant::from_env() {
        Ok(variant) => variant,
        Err(e) => {
            eprintln!(
                "For local builds, you must set the '{}' environment variable so we know which API \
                model to build against. Valid values are the directories in variants/, for example \
                'aws-ecs-1': {}",
                VARIANT_ENV, e
            );
            std::process::exit(1);
        }
    };
    // Tell cargo when we have to rerun; we always want variant links to be correct, especially
    // after changing the variant we're building for.
    Variant::rerun_if_changed();
    println!("cargo:rerun-if-changed={}", VARIANT_LINK);
    println!("cargo:rerun-if-changed={}", MOD_LINK);

    generate_readme::from_lib().unwrap();
    link_current_variant(variant);
}

fn link_current_variant(variant: Variant) {
    // Point to the source for the requested variant
    let variant_target = format!("../{}", variant);

    // Make sure requested variant exists
    let variant_path = format!("src/{}", variant);
    if !Path::new(&variant_path).exists() {
        eprintln!("The environment variable {} should refer to a directory under sources/models/src with an API model, but it's set to '{}' which doesn't exist", VARIANT_ENV, variant);
        process::exit(1);
    }

    // Create the symlink for the following `cargo build` to use for its source code
    symlink_safe(&variant_target, VARIANT_LINK).unwrap_or_else(|e| {
        eprintln!("Failed to create symlink at '{}' pointing to '{}' - we need this to support different API models for different variants.  Error: {}", VARIANT_LINK, variant_target, e);
        process::exit(1);
    });

    // Also create the link for mod.rs so Rust can import source from the "current" link
    // created above.
    symlink_safe(MOD_LINK_TARGET, MOD_LINK).unwrap_or_else(|e| {
        eprintln!("Failed to create symlink at '{}' pointing to '{}' - we need this to build a Rust module structure through the `current` link.  Error: {}", MOD_LINK, MOD_LINK_TARGET, e);
        process::exit(1);
    });

    // Set the mtime of the links to a fixed time, the epoch.  This is because cargo decides
    // whether to rerun build.rs based on the "rerun-if-changed" statements printed above and the
    // mtime of the files they reference.  If the mtime of the file doesn't match the mtime of the
    // "output" file in the build directory (which contains the output of the rerun-if prints) then
    // it rebuilds.  Those times won't match because we don't control when they happen, meaning
    // we'd rebuild every time.  Setting to a consistent time means we only rebuild when the other
    // rerun-if statements apply, the important one being the variant changing.
    //
    // Note that we still use rerun-if-changed for these links in case someone changes them outside
    // of this build.rs.  If they really want to get around our system, they'd also need to set the
    // mtime to epoch, and then hopefully they know what they're doing.
    for link in &[VARIANT_LINK, MOD_LINK] {
        // Do our best, but if we fail, rebuilding isn't the end of the world.
        // Note: set_symlink_file_times is the only method that operates on the symlink rather than
        // its target, and it also updates atime, which we don't care about but isn't harmful.
        if let Err(e) = set_symlink_file_times(link, FileTime::zero(), FileTime::zero()) {
            eprintln!(
                "Warning: unable to set mtime on {}; crate may rebuild unnecessarily: {}",
                link, e
            );
        }
    }
}

// Creates the requested symlink through an atomic swap, so it doesn't matter if the link path
// already exists or not; like --force but fewer worries about reentrancy and retries.
fn symlink_safe<P1, P2>(target: P1, link: P2) -> io::Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // Create the link at a temporary path.
    let temp_link = link.as_ref().with_file_name(format!(".{}", rando()));
    symlink(&target, &temp_link)?;

    // Swap the temporary link into the real location
    if let Err(e) = fs::rename(&temp_link, &link) {
        // If we couldn't, for whatever reason, clean up the temporary path and return the error.
        let _ = fs::remove_file(&temp_link);
        return Err(e);
    }

    Ok(())
}

// Generates a random ID, affectionately known as a 'rando'.
fn rando() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
