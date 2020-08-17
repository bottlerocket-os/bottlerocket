/*!
This module handles the calls to Docker needed to execute package and variant
builds. The actual build steps and the expected parameters are defined in
the repository's top-level Dockerfile.

*/
pub(crate) mod error;
use error::Result;

use duct::cmd;
use nonzero_ext::nonzero;
use rand::Rng;
use sha2::{Digest, Sha512};
use snafu::{ensure, ResultExt};
use std::env;
use std::num::NonZeroU16;
use std::process::Output;

/*
There's a bug in BuildKit that can lead to a build failure during parallel
`docker build` executions:
   https://github.com/moby/buildkit/issues/1090

Unfortunately we can't do much to control the concurrency here, and even when
the bug is fixed there will be many older versions of Docker in the wild.

The failure has an exit code of 1, which is too generic to be helpful. All we
can do is check the output for the error's signature, and retry if we find it.
*/
static DOCKER_BUILD_FRONTEND_ERROR: &str = concat!(
    r#"failed to solve with frontend dockerfile.v0: "#,
    r#"failed to solve with frontend gateway.v0: "#,
    r#"frontend grpc server closed unexpectedly"#
);

static DOCKER_BUILD_MAX_ATTEMPTS: NonZeroU16 = nonzero!(10u16);

pub(crate) struct PackageBuilder;

impl PackageBuilder {
    /// Build RPMs for the specified package.
    pub(crate) fn build(package: &str) -> Result<Self> {
        let arch = getenv("BUILDSYS_ARCH")?;
        let output = getenv("BUILDSYS_PACKAGES_DIR")?;

        // We do *not* want to rebuild most packages when the variant changes, becauses most aren't
        // affected; packages that care about variant should "echo cargo:rerun-if-env-changed=VAR"
        // themselves in the package's spec file.
        let var = "BUILDSYS_VARIANT";
        let variant = env::var(var).context(error::Environment { var })?;

        let target = "package";
        let build_args = format!(
            "--build-arg PACKAGE={package} \
             --build-arg ARCH={arch} \
             --build-arg VARIANT={variant}",
            package = package,
            arch = arch,
            variant = variant,
        );
        let tag = format!(
            "buildsys-pkg-{package}-{arch}",
            package = package,
            arch = arch,
        );

        build(&target, &build_args, &tag, &output)?;

        Ok(Self)
    }
}

pub(crate) struct VariantBuilder;

impl VariantBuilder {
    /// Build a variant with the specified packages installed.
    pub(crate) fn build(packages: &[String]) -> Result<Self> {
        // We want PACKAGES to be a value that contains spaces, since that's
        // easier to work with in the shell than other forms of structured data.
        let packages = packages.join("|");
        let arch = getenv("BUILDSYS_ARCH")?;
        let variant = getenv("BUILDSYS_VARIANT")?;
        let version_image = getenv("BUILDSYS_VERSION_IMAGE")?;
        let version_build = getenv("BUILDSYS_VERSION_BUILD")?;
        let output = getenv("BUILDSYS_OUTPUT_DIR")?;

        // Always rebuild variants since they are located in a different workspace,
        // and don't directly track changes in the underlying packages.
        getenv("BUILDSYS_TIMESTAMP")?;

        let target = "variant";
        let build_args = format!(
            "--build-arg PACKAGES={packages} \
             --build-arg ARCH={arch} \
             --build-arg VARIANT={variant} \
             --build-arg VERSION_ID={version_image} \
             --build-arg BUILD_ID={version_build}",
            packages = packages,
            arch = arch,
            variant = variant,
            version_image = version_image,
            version_build = version_build,
        );
        let tag = format!(
            "buildsys-var-{variant}-{arch}",
            variant = variant,
            arch = arch
        );

        build(&target, &build_args, &tag, &output)?;

        Ok(Self)
    }
}

/// Invoke a series of `docker` commands to drive a package or variant build.
fn build(target: &str, build_args: &str, tag: &str, output: &str) -> Result<()> {
    // Our Dockerfile is in the top-level directory.
    let root = getenv("BUILDSYS_ROOT_DIR")?;
    std::env::set_current_dir(&root).context(error::DirectoryChange { path: &root })?;

    // Compute a per-checkout prefix for the tag to avoid collisions.
    let mut d = Sha512::new();
    d.update(&root);
    let digest = hex::encode(d.finalize());
    let token = &digest[..12];
    let tag = format!("{}-{}", tag, token);

    // Our SDK image is picked by the external `cargo make` invocation.
    let sdk = getenv("BUILDSYS_SDK_IMAGE")?;
    let sdk_args = format!("--build-arg SDK={}", sdk);

    // Avoid using a cached layer from a previous build.
    let nocache = rand::thread_rng().gen::<u32>();
    let nocache_args = format!("--build-arg NOCACHE={}", nocache);

    // Avoid using a cached layer from a concurrent build in another checkout.
    let token_args = format!("--build-arg TOKEN={}", token);

    let build = args(format!(
        "build . \
         --network none \
         --target {target} \
         {build_args} \
         {sdk_args} \
         {nocache_args} \
         {token_args} \
         --tag {tag}",
        target = target,
        build_args = build_args,
        sdk_args = sdk_args,
        nocache_args = nocache_args,
        token_args = token_args,
        tag = tag,
    ));

    let create = args(format!("create --name {tag} {tag} true", tag = tag));
    let cp = args(format!("cp {}:/output/. {}", tag, output));
    let rm = args(format!("rm --force {}", tag));
    let rmi = args(format!("rmi --force {}", tag));

    // Clean up the stopped container if it exists.
    let _ = docker(&rm, Retry::No);

    // Clean up the previous image if it exists.
    let _ = docker(&rmi, Retry::No);

    // Build the image, which builds the artifacts we want.
    // Work around a transient, known failure case with Docker.
    docker(
        &build,
        Retry::Yes {
            attempts: DOCKER_BUILD_MAX_ATTEMPTS,
            messages: &[DOCKER_BUILD_FRONTEND_ERROR],
        },
    )?;

    // Create a stopped container so we can copy artifacts out.
    docker(&create, Retry::No)?;

    // Copy artifacts into our output directory.
    docker(&cp, Retry::No)?;

    // Clean up our stopped container after copying artifacts out.
    docker(&rm, Retry::No)?;

    // Clean up our image now that we're done.
    docker(&rmi, Retry::No)?;

    Ok(())
}

/// Run `docker` with the specified arguments.
fn docker(args: &[String], retry: Retry) -> Result<Output> {
    let mut max_attempts: u16 = 1;
    let mut retry_messages: &[&str] = &[];
    if let Retry::Yes { attempts, messages } = retry {
        max_attempts = attempts.into();
        retry_messages = messages;
    }

    let mut attempt = 1;
    loop {
        let output = cmd("docker", args)
            .stderr_to_stdout()
            .stdout_capture()
            .unchecked()
            .run()
            .context(error::CommandStart)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", &stdout);
        if output.status.success() {
            return Ok(output);
        }

        ensure!(
            retry_messages.iter().any(|&m| stdout.contains(m)) && attempt < max_attempts,
            error::DockerExecution {
                args: &args.join(" ")
            }
        );

        attempt += 1;
    }
}

/// Allow the caller to configure retry behavior, since the command may fail
/// for spurious reasons that should not be treated as an error.
enum Retry<'a> {
    No,
    Yes {
        attempts: NonZeroU16,
        messages: &'a [&'a str],
    },
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
