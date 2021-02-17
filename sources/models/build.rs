// src/variant/current is a link to the API model we actually want to build; this build.rs creates
// that symlink based on the VARIANT environment variable, which either comes from the build
// system or the user, if doing a local `cargo build`.
//
// See README.md to understand the symlink setup.

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;

const VARIANT_LINK: &str = "src/variant/current";
const MOD_LINK: &str = "src/variant/mod.rs";
const VARIANT_ENV: &str = "VARIANT";

fn main() {
    // Tell cargo when we have to rerun, regardless of early-exit below.
    println!("cargo:rerun-if-env-changed={}", VARIANT_ENV);
    println!("cargo:rerun-if-changed={}", VARIANT_LINK);
    println!("cargo:rerun-if-changed={}", MOD_LINK);

    // This build.rs runs once as a build-dependency of storewolf, and again as a (regular)
    // dependency of storewolf.  There's no reason to do this work twice.
    if env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_else(|_| String::new()) == "bottlerocket" {
        println!("cargo:warning=Already ran model build.rs for host, skipping for target");
        process::exit(0);
    }

    generate_readme();
    link_current_variant();
}

fn link_current_variant() {
    // The VARIANT variable is originally BUILDSYS_VARIANT, set in the top-level Makefile.toml,
    // and is passed through as VARIANT by the top-level Dockerfile.  It represents which OS
    // variant we're building, and therefore which API model to use.
    let variant = env::var(VARIANT_ENV).unwrap_or_else(|_| {
        eprintln!("For local builds, you must set the {} environment variable so we know which API model to build against.  Valid values are the directories in variants/, for example \"aws-k8s-1.17\".", VARIANT_ENV);
        process::exit(1);
    });

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
    let mod_target = "../variant_mod.rs";
    symlink_safe(&mod_target, MOD_LINK).unwrap_or_else(|e| {
        eprintln!("Failed to create symlink at '{}' pointing to '{}' - we need this to build a Rust module structure through the `current` link.  Error: {}", MOD_LINK, mod_target, e);
        process::exit(1);
    });
}

fn generate_readme() {
    // Check for environment variable "SKIP_README". If it is set,
    // skip README generation
    if env::var_os("SKIP_README").is_some() {
        return;
    }

    let mut lib = File::open("src/lib.rs").unwrap();
    let mut template = File::open("README.tpl").unwrap();

    let content = cargo_readme::generate_readme(
        &PathBuf::from("."), // root
        &mut lib,            // source
        Some(&mut template), // template
        // The "add x" arguments don't apply when using a template.
        true,  // add title
        false, // add badges
        false, // add license
        true,  // indent headings
    )
    .unwrap();

    let mut readme = File::create("README.md").unwrap();
    readme.write_all(content.as_bytes()).unwrap();
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
