// Automatically generate README.md from rustdoc and generate variant symlink

use bottlerocket_variant::{Variant, VARIANT_ENV};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{fs, io, process};

/// Creates a file, `conf/current/logdog.conf` which is a symlink to a file with `logdog` commands
/// for the current variant. Whatever the value of the `VARIANT` environment variable is, this
/// function requires a file at `conf/logdog.$VARIANT.conf` and points to it from the `logdog.conf`
/// symlink. For example, if the variant is `aws-ecs-1` then `conf/current/logdog.conf` will
/// point to `conf/logdog.aws-ecs-1.conf`.
fn symlink_variant() {
    Variant::rerun_if_changed();
    let variant = match Variant::from_env() {
        Ok(variant) => variant,
        Err(e) => {
            eprintln!(
                "For local builds, you must set the '{}' environment variable so we know which \
                logdog commands to build. Valid values are the directories in \
                models/src/variants/, for example 'aws-ecs-1': {}",
                VARIANT_ENV, e
            );
            std::process::exit(1);
        }
    };
    let variant_filename = format!("logdog.{}.conf", variant);
    if !PathBuf::from("conf").join(&variant_filename).is_file() {
        eprintln!(
            "There is no file named '{}' in the 'conf' directory for the current variant (given \
            by the '{}' environment variable) Each variant must have a file representing the \
            variant-specific commands that logdog will run.",
            variant, VARIANT_ENV
        );
        process::exit(1);
    }
    // create the symlink from conf/current/logdog.conf to the variant-specific file
    let target = format!("../{}", variant_filename);
    let link = "conf/current/logdog.conf";
    symlink_force(&target, link).unwrap_or_else(|e| {
        eprintln!(
            "Failed to create symlink at '{}' pointing to '{}' - we need this to \
            support different logdog commands for variants.  Error: {}",
            link, target, e
        );
        process::exit(1);
    });
}

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
    symlink_variant();
    generate_readme::from_main().unwrap();
}
