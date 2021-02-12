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
use snafu::{ensure, OptionExt, ResultExt};
use std::env;
use std::fs::{self, File};
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::process::Output;
use walkdir::{DirEntry, WalkDir};

use crate::manifest::ImageFormat;

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
        let output_dir: PathBuf = getenv("BUILDSYS_PACKAGES_DIR")?.into();

        // We do *not* want to rebuild most packages when the variant changes, because most aren't
        // affected; packages that care about variant should "echo cargo:rerun-if-env-changed=VAR"
        // themselves in the package's spec file.
        let var = "BUILDSYS_VARIANT";
        let variant = env::var(var).context(error::Environment { var })?;
        // Same for repo, which is used to determine the correct root.json, which is only included
        // in the os package.
        let var = "PUBLISH_REPO";
        let repo = env::var(var).context(error::Environment { var })?;

        let build_args = format!(
            "--build-arg PACKAGE={package} \
             --build-arg ARCH={arch} \
             --build-arg VARIANT={variant} \
             --build-arg REPO={repo}",
            package = package,
            arch = arch,
            variant = variant,
            repo = repo,
        );
        let tag = format!(
            "buildsys-pkg-{package}-{arch}",
            package = package,
            arch = arch,
        );

        build(BuildType::Package, &package, &build_args, &tag, &output_dir)?;

        Ok(Self)
    }
}

pub(crate) struct VariantBuilder;

impl VariantBuilder {
    /// Build a variant with the specified packages installed.
    pub(crate) fn build(packages: &[String], image_format: Option<&ImageFormat>) -> Result<Self> {
        // We want PACKAGES to be a value that contains spaces, since that's
        // easier to work with in the shell than other forms of structured data.
        let packages = packages.join("|");
        let arch = getenv("BUILDSYS_ARCH")?;
        let variant = getenv("BUILDSYS_VARIANT")?;
        let version_image = getenv("BUILDSYS_VERSION_IMAGE")?;
        let version_build = getenv("BUILDSYS_VERSION_BUILD")?;
        let output_dir: PathBuf = getenv("BUILDSYS_OUTPUT_DIR")?.into();
        // We expect users' PRETTY_NAME values to contain spaces for things like "Bottlerocket OS"
        // and so we need to transform them the same way as PACKAGES above.
        let pretty_name = getenv("BUILDSYS_PRETTY_NAME")?.replace(' ', "|");
        let image_name = getenv("BUILDSYS_NAME")?;
        let image_format = match image_format {
            Some(ImageFormat::Raw) | None => String::from("raw"),
            Some(ImageFormat::Vmdk) => String::from("vmdk"),
        };

        // Always rebuild variants since they are located in a different workspace,
        // and don't directly track changes in the underlying packages.
        getenv("BUILDSYS_TIMESTAMP")?;

        let build_args = format!(
            "--build-arg PACKAGES={packages} \
             --build-arg ARCH={arch} \
             --build-arg VARIANT={variant} \
             --build-arg VERSION_ID={version_image} \
             --build-arg BUILD_ID={version_build} \
             --build-arg PRETTY_NAME={pretty_name} \
             --build-arg IMAGE_NAME={image_name} \
             --build-arg IMAGE_FORMAT={image_format}",
            packages = packages,
            arch = arch,
            variant = variant,
            version_image = version_image,
            version_build = version_build,
            pretty_name = pretty_name,
            image_name = image_name,
            image_format = image_format,
        );
        let tag = format!(
            "buildsys-var-{variant}-{arch}",
            variant = variant,
            arch = arch
        );

        build(BuildType::Variant, &variant, &build_args, &tag, &output_dir)?;

        Ok(Self)
    }
}

enum BuildType {
    Package,
    Variant,
}

/// Invoke a series of `docker` commands to drive a package or variant build.
fn build(
    kind: BuildType,
    what: &str,
    build_args: &str,
    tag: &str,
    output_dir: &PathBuf,
) -> Result<()> {
    // Our Dockerfile is in the top-level directory.
    let root = getenv("BUILDSYS_ROOT_DIR")?;
    env::set_current_dir(&root).context(error::DirectoryChange { path: &root })?;

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

    // Create a directory for tracking outputs before we move them into position.
    let build_dir = create_build_dir(&kind, &what)?;

    // Clean up any previous outputs we have tracked.
    clean_build_files(&build_dir, &output_dir)?;

    let target = match kind {
        BuildType::Package => "package",
        BuildType::Variant => "variant",
    };

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
    let cp = args(format!("cp {}:/output/. {}", tag, build_dir.display()));
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

    // Copy artifacts to the expected directory and write markers to track them.
    copy_build_files(&build_dir, &output_dir)?;

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

/// Create a directory for build artifacts.
fn create_build_dir(kind: &BuildType, name: &str) -> Result<PathBuf> {
    let prefix = match kind {
        BuildType::Package => "packages",
        BuildType::Variant => "variants",
    };

    let path = [&getenv("BUILDSYS_STATE_DIR")?, prefix, name]
        .iter()
        .collect();

    fs::create_dir_all(&path).context(error::DirectoryCreate { path: &path })?;

    Ok(path)
}

const MARKER_EXTENSION: &str = ".buildsys_marker";

/// Copy build artifacts to the output directory.
/// Currently we expect a "flat" structure where all files are in the same directory.
/// Before we copy each file, we create a corresponding marker file to record its existence.
fn copy_build_files<P>(build_dir: P, output_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    fn is_artifact(entry: &DirEntry) -> bool {
        entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .map(|s| !s.ends_with(MARKER_EXTENSION))
                .unwrap_or(false)
    }

    for artifact_file in find_files(&build_dir, is_artifact) {
        let mut marker_file = artifact_file.clone().into_os_string();
        marker_file.push(MARKER_EXTENSION);
        File::create(&marker_file).context(error::FileCreate { path: &marker_file })?;

        let mut output_file: PathBuf = output_dir.as_ref().into();
        output_file.push(
            artifact_file
                .file_name()
                .context(error::BadFilename { path: &output_file })?,
        );

        fs::rename(&artifact_file, &output_file).context(error::FileRename {
            old_path: &artifact_file,
            new_path: &output_file,
        })?;
    }

    Ok(())
}

/// Remove build artifacts from the output directory.
/// Any marker file we find could have a corresponding file that should be cleaned up.
/// We also clean up the marker files so they do not accumulate across builds.
fn clean_build_files<P>(build_dir: P, output_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    fn is_marker(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(MARKER_EXTENSION))
            .unwrap_or(false)
    }

    for marker_file in find_files(&build_dir, is_marker) {
        let mut output_file: PathBuf = output_dir.as_ref().into();
        output_file.push(
            marker_file
                .file_name()
                .context(error::BadFilename { path: &marker_file })?,
        );

        output_file.set_extension("");
        if output_file.exists() {
            std::fs::remove_file(&output_file).context(error::FileRemove { path: &output_file })?;
        }

        std::fs::remove_file(&marker_file).context(error::FileRemove { path: &marker_file })?;
    }

    Ok(())
}

/// Create an iterator over files matching the supplied filter.
fn find_files<P>(
    dir: P,
    filter: for<'r> fn(&'r walkdir::DirEntry) -> bool,
) -> impl Iterator<Item = PathBuf>
where
    P: AsRef<Path>,
{
    WalkDir::new(&dir)
        .follow_links(false)
        .same_file_system(true)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(move |e| filter(e))
        .flat_map(|e| e.context(error::DirectoryWalk))
        .map(|e| e.into_path())
}

/// Retrieve a BUILDSYS_* variable that we expect to be set in the environment,
/// and ensure that we track it for changes, since it will directly affect the
/// output.
fn getenv(var: &str) -> Result<String> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var(var).context(error::Environment { var })
}
