/*!
This module handles the calls to the BuildKit server needed to execute package
and image builds. The actual build steps and the expected parameters are defined
in the repository's top-level Dockerfile.

*/
pub(crate) mod error;
use error::Result;

use duct::cmd;
use rand::Rng;
use snafu::ResultExt;
use std::env;
use std::process::Output;
use users::get_effective_uid;

pub(crate) struct PackageBuilder;

impl PackageBuilder {
    /// Call `buildctl` to produce RPMs for the specified package.
    pub(crate) fn build(package: &str) -> Result<(Self)> {
        let arch = getenv("BUILDSYS_ARCH")?;
        let opts = format!(
            "--opt target=rpm \
             --opt build-arg:PACKAGE={package} \
             --opt build-arg:ARCH={arch}",
            package = package,
            arch = arch,
        );

        let result = buildctl(&opts)?;
        if !result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            return error::PackageBuild { package, output }.fail();
        }

        Ok(Self)
    }
}

pub(crate) struct ImageBuilder;

impl ImageBuilder {
    /// Call `buildctl` to create an image with the specified packages installed.
    pub(crate) fn build(packages: &[String]) -> Result<(Self)> {
        // We want PACKAGES to be a value that contains spaces, since that's
        // easier to work with in the shell than other forms of structured data.
        let packages = packages.join("|");

        let arch = getenv("BUILDSYS_ARCH")?;
        let opts = format!(
            "--opt target=image \
             --opt build-arg:PACKAGES={packages} \
             --opt build-arg:ARCH={arch}",
            packages = packages,
            arch = arch,
        );

        // Always rebuild images since they are located in a different workspace,
        // and don't directly track changes in the underlying packages.
        getenv("BUILDSYS_TIMESTAMP")?;

        let result = buildctl(&opts)?;
        if !result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            return error::ImageBuild { packages, output }.fail();
        }

        Ok(Self)
    }
}

/// Invoke `buildctl` by way of `docker` with the arguments for a specific
/// package or image build.
fn buildctl(opts: &str) -> Result<Output> {
    let docker_args = docker_args()?;
    let buildctl_args = buildctl_args()?;

    // Avoid using a cached layer from a previous build.
    let nocache = format!(
        "--opt build-arg:NOCACHE={}",
        rand::thread_rng().gen::<u32>(),
    );

    // Build the giant chain of args. Treat "|" as a placeholder that indicates
    // where the argument should contain spaces after we split on whitespace.
    let args = docker_args
        .split_whitespace()
        .chain(buildctl_args.split_whitespace())
        .chain(opts.split_whitespace())
        .chain(nocache.split_whitespace())
        .map(|s| s.replace("|", " "));

    // Run the giant docker invocation
    cmd("docker", args)
        .stderr_to_stdout()
        .run()
        .context(error::CommandExecution)
}

/// Prepare the arguments for docker
fn docker_args() -> Result<String> {
    // Gather the user context.
    let uid = get_effective_uid();

    // Gather the environment context.
    let root_dir = getenv("BUILDSYS_ROOT_DIR")?;
    let buildkit_client = getenv("BUILDSYS_BUILDKIT_CLIENT")?;

    let docker_args = format!(
        "run --init --rm --network host --user {uid}:{uid} \
         --volume {root_dir}:{root_dir} --workdir {root_dir} \
         --entrypoint /usr/bin/buildctl {buildkit_client}",
        uid = uid,
        root_dir = root_dir,
        buildkit_client = buildkit_client
    );

    Ok(docker_args)
}

fn buildctl_args() -> Result<String> {
    // Gather the environment context.
    let output_dir = getenv("BUILDSYS_OUTPUT_DIR")?;
    let buildkit_server = getenv("BUILDSYS_BUILDKIT_SERVER")?;

    let buildctl_args = format!(
        "--addr {buildkit_server} build --progress=plain \
         --frontend=dockerfile.v0 --local context=. --local dockerfile=. \
         --output type=local,dest={output_dir}",
        buildkit_server = buildkit_server,
        output_dir = output_dir
    );

    Ok(buildctl_args)
}

/// Retrieve a BUILDSYS_* variable that we expect to be set in the environment,
/// and ensure that we track it for changes, since it will directly affect the
/// output.
fn getenv(var: &str) -> Result<String> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var(var).context(error::Environment { var })
}
