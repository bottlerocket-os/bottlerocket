/*!
This module handles the calls to Docker needed to execute package and variant
builds. The actual build steps and the expected parameters are defined in
the repository's top-level Dockerfile.

*/
pub(crate) mod error;
use error::Result;

use duct::cmd;
use rand::Rng;
use sha2::{Digest, Sha512};
use snafu::ResultExt;
use std::env;
use std::process::Output;

pub(crate) struct PackageBuilder;

impl PackageBuilder {
    /// Build RPMs for the specified package.
    pub(crate) fn build(package: &str) -> Result<(Self)> {
        let arch = getenv("BUILDSYS_ARCH")?;

        let target = "package";
        let build_args = format!(
            "--build-arg PACKAGE={package} \
             --build-arg ARCH={arch}",
            package = package,
            arch = arch,
        );
        let tag = format!(
            "buildsys-pkg-{package}-{arch}",
            package = package,
            arch = arch
        );

        build(&target, &build_args, &tag)?;

        Ok(Self)
    }
}

pub(crate) struct VariantBuilder;

impl VariantBuilder {
    /// Build a variant with the specified packages installed.
    pub(crate) fn build(packages: &[String]) -> Result<(Self)> {
        // We want PACKAGES to be a value that contains spaces, since that's
        // easier to work with in the shell than other forms of structured data.
        let packages = packages.join("|");
        let arch = getenv("BUILDSYS_ARCH")?;
        let variant = getenv("BUILDSYS_VARIANT")?;

        // Always rebuild variants since they are located in a different workspace,
        // and don't directly track changes in the underlying packages.
        getenv("BUILDSYS_TIMESTAMP")?;

        let target = "variant";
        let build_args = format!(
            "--build-arg PACKAGES={packages} \
             --build-arg ARCH={arch} \
             --build-arg VARIANT={variant}",
            packages = packages,
            arch = arch,
            variant = variant,
        );
        let tag = format!("buildsys-var-{variant}-{arch}", variant = variant, arch = arch);

        build(&target, &build_args, &tag)?;

        Ok(Self)
    }
}

/// Invoke a series of `docker` commands to drive a package or variant build.
fn build(target: &str, build_args: &str, tag: &str) -> Result<()> {
    // Our Dockerfile is in the top-level directory.
    let root = getenv("BUILDSYS_ROOT_DIR")?;
    std::env::set_current_dir(&root).context(error::DirectoryChange { path: &root })?;

    // Compute a per-checkout prefix for the tag to avoid collisions.
    let mut d = Sha512::new();
    d.input(&root);
    let digest = hex::encode(d.result());
    let suffix = &digest[..12];
    let tag = format!("{}-{}", tag, suffix);

    // Our SDK image is picked by the external `cargo make` invocation.
    let sdk = getenv("BUILDSYS_SDK_IMAGE")?;
    let sdk_args = format!("--build-arg SDK={}", sdk);

    // Avoid using a cached layer from a previous build.
    let nocache = rand::thread_rng().gen::<u32>();
    let nocache_args = format!("--build-arg NOCACHE={}", nocache);

    let build = args(format!(
        "build . \
         --network none \
         --target {target} \
         {build_args} \
         {sdk_args} \
         {nocache_args} \
         --tag {tag}",
        target = target,
        build_args = build_args,
        sdk_args = sdk_args,
        nocache_args = nocache_args,
        tag = tag,
    ));

    let output = getenv("BUILDSYS_OUTPUT_DIR")?;
    let create = args(format!("create --name {tag} {tag} true", tag = tag));
    let cp = args(format!("cp {}:/output/. {}", tag, output));
    let rm = args(format!("rm --force {}", tag));
    let rmi = args(format!("rmi --force {}", tag));

    // Clean up the stopped container if it exists.
    let _ = docker(&rm);

    // Clean up the previous image if it exists.
    let _ = docker(&rmi);

    // Build the image, which builds the artifacts we want.
    docker(&build)?;

    // Create a stopped container so we can copy artifacts out.
    docker(&create)?;

    // Copy artifacts into our output directory.
    docker(&cp)?;

    // Clean up our stopped container after copying artifacts out.
    docker(&rm)?;

    // Clean up our image now that we're done.
    docker(&rmi)?;

    Ok(())
}

/// Run `docker` with the specified arguments.
fn docker(args: &[String]) -> Result<Output> {
    cmd("docker", args)
        .stderr_to_stdout()
        .run()
        .context(error::CommandExecution)
}

/// Convert an argument string into a collection of positional arguments.
fn args<S>(input: S) -> Vec<String>
where
    S: AsRef<str>,
{
    // Treat "|" as a placeholder that indicates where the argument should
    // contain spaces after we split on whitespace.
    input
        .as_ref()
        .split_whitespace()
        .map(|s| s.replace("|", " "))
        .collect()
}

/// Retrieve a BUILDSYS_* variable that we expect to be set in the environment,
/// and ensure that we track it for changes, since it will directly affect the
/// output.
fn getenv(var: &str) -> Result<String> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var(var).context(error::Environment { var })
}
