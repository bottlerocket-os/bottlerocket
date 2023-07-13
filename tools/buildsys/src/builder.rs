/*!
This module handles the calls to Docker needed to execute package and variant
builds. The actual build steps and the expected parameters are defined in
the repository's top-level Dockerfile.

*/
pub(crate) mod error;
use error::Result;

use duct::cmd;
use lazy_static::lazy_static;
use nonzero_ext::nonzero;
use rand::Rng;
use regex::Regex;
use sha2::{Digest, Sha512};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashSet;
use std::env;
use std::fs::{self, read_dir, File};
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::process::Output;
use walkdir::{DirEntry, WalkDir};

use buildsys::manifest::{ImageFeature, ImageFormat, ImageLayout, PartitionPlan, SupportedArch};

/*
There's a bug in BuildKit that can lead to a build failure during parallel
`docker build` executions:
   https://github.com/moby/buildkit/issues/1090

Unfortunately we can't do much to control the concurrency here, and even when
the bug is fixed there will be many older versions of Docker in the wild.

The failure has an exit code of 1, which is too generic to be helpful. All we
can do is check the output for the error's signature, and retry if we find it.
*/
lazy_static! {
    static ref DOCKER_BUILD_FRONTEND_ERROR: Regex = Regex::new(concat!(
        r#"failed to solve with frontend dockerfile.v0: "#,
        r#"failed to solve with frontend gateway.v0: "#,
        r#"frontend grpc server closed unexpectedly"#
    ))
    .unwrap();
}

/*
There's a similar bug that's fixed in new releases of BuildKit but still in the wild in popular
versions of Docker/BuildKit:
   https://github.com/moby/buildkit/issues/1468
*/
lazy_static! {
    static ref DOCKER_BUILD_DEAD_RECORD_ERROR: Regex = Regex::new(concat!(
        r#"failed to solve with frontend dockerfile.v0: "#,
        r#"failed to solve with frontend gateway.v0: "#,
        r#"rpc error: code = Unknown desc = failed to build LLB: "#,
        r#"failed to get dead record"#,
    ))
    .unwrap();
}

/*
We also see sporadic CI failures with only this error message.
We use (?m) for multi-line mode so we can match the message on a line of its own without splitting
the output ourselves; we match the regexes against the whole of stdout.
*/
lazy_static! {
    static ref UNEXPECTED_EOF_ERROR: Regex = Regex::new("(?m)unexpected EOF$").unwrap();
}

/*
Sometimes new RPMs are not fully written to the host directory before another build starts, which
exposes `createrepo_c` to partially-written RPMs that cannot be added to the repo metadata. Retry
these errors by restarting the build since the alternatives are to ignore the `createrepo_c` exit
code (masking other problems) or aggressively `sync()` the host directory (hurting performance).
*/
lazy_static! {
    static ref CREATEREPO_C_READ_HEADER_ERROR: Regex = Regex::new(&regex::escape(
        r#"C_CREATEREPOLIB: Warning: read_header: rpmReadPackageFile() error"#
    ))
    .unwrap();
}

static DOCKER_BUILD_MAX_ATTEMPTS: NonZeroU16 = nonzero!(10u16);

pub(crate) struct PackageBuilder;

impl PackageBuilder {
    /// Build RPMs for the specified package.
    pub(crate) fn build(
        package: &str,
        image_features: Option<HashSet<&ImageFeature>>,
    ) -> Result<Self> {
        let output_dir: PathBuf = getenv("BUILDSYS_PACKAGES_DIR")?.into();
        let arch = getenv("BUILDSYS_ARCH")?;
        let goarch = serde_plain::from_str::<SupportedArch>(&arch)
            .context(error::UnsupportedArchSnafu { arch: &arch })?
            .goarch();

        let mut args = Vec::new();
        args.push("--network".into());
        args.push("none".into());
        args.build_arg("PACKAGE", package);
        args.build_arg("ARCH", &arch);
        args.build_arg("GOARCH", goarch);

        // Pass certain environment variables into the build environment. These variables aren't
        // automatically used to trigger rebuilds when they change, because most packages aren't
        // affected. Packages that care should "echo cargo:rerun-if-env-changed=VAR" in their
        // build.rs build script.
        for (src_env_var, target_env_var) in [
            ("BUILDSYS_VARIANT", "VARIANT"),
            ("BUILDSYS_VARIANT_PLATFORM", "VARIANT_PLATFORM"),
            ("BUILDSYS_VARIANT_RUNTIME", "VARIANT_RUNTIME"),
            ("BUILDSYS_VARIANT_FAMILY", "VARIANT_FAMILY"),
            ("BUILDSYS_VARIANT_FLAVOR", "VARIANT_FLAVOR"),
            ("PUBLISH_REPO", "REPO"),
        ] {
            let src_env_val =
                env::var(src_env_var).context(error::EnvironmentSnafu { var: src_env_var })?;
            args.build_arg(target_env_var, src_env_val);
        }

        let tag = format!(
            "buildsys-pkg-{package}-{arch}",
            package = package,
            arch = arch,
        );

        if let Some(image_features) = image_features {
            for image_feature in image_features.iter() {
                args.build_arg(format!("{}", image_feature), "1");
            }
        }

        build(BuildType::Package, package, &arch, args, &tag, &output_dir)?;

        Ok(Self)
    }
}

pub(crate) struct VariantBuilder;

impl VariantBuilder {
    /// Build a variant with the specified packages installed.
    pub(crate) fn build(
        packages: &[String],
        image_format: Option<&ImageFormat>,
        image_layout: Option<&ImageLayout>,
        kernel_parameters: Option<&Vec<String>>,
        image_features: Option<HashSet<&ImageFeature>>,
    ) -> Result<Self> {
        let output_dir: PathBuf = getenv("BUILDSYS_OUTPUT_DIR")?.into();

        let variant = getenv("BUILDSYS_VARIANT")?;
        let arch = getenv("BUILDSYS_ARCH")?;
        let goarch = serde_plain::from_str::<SupportedArch>(&arch)
            .context(error::UnsupportedArchSnafu { arch: &arch })?
            .goarch();

        let image_layout = image_layout.cloned().unwrap_or_default();
        let ImageLayout {
            os_image_size_gib,
            data_image_size_gib,
            partition_plan,
            ..
        } = image_layout;

        let (os_image_publish_size_gib, data_image_publish_size_gib) =
            image_layout.publish_image_sizes_gib();

        let mut args = Vec::new();
        args.push("--network".into());
        args.push("host".into());
        args.build_arg("PACKAGES", packages.join(" "));
        args.build_arg("ARCH", &arch);
        args.build_arg("GOARCH", goarch);
        args.build_arg("VARIANT", &variant);
        args.build_arg("VERSION_ID", getenv("BUILDSYS_VERSION_IMAGE")?);
        args.build_arg("BUILD_ID", getenv("BUILDSYS_VERSION_BUILD")?);
        args.build_arg("PRETTY_NAME", getenv("BUILDSYS_PRETTY_NAME")?);
        args.build_arg("IMAGE_NAME", getenv("BUILDSYS_NAME")?);
        args.build_arg(
            "IMAGE_FORMAT",
            match image_format {
                Some(ImageFormat::Raw) | None => "raw",
                Some(ImageFormat::Qcow2) => "qcow2",
                Some(ImageFormat::Vmdk) => "vmdk",
            },
        );
        args.build_arg("OS_IMAGE_SIZE_GIB", format!("{}", os_image_size_gib));
        args.build_arg("DATA_IMAGE_SIZE_GIB", format!("{}", data_image_size_gib));
        args.build_arg(
            "OS_IMAGE_PUBLISH_SIZE_GIB",
            format!("{}", os_image_publish_size_gib),
        );
        args.build_arg(
            "DATA_IMAGE_PUBLISH_SIZE_GIB",
            format!("{}", data_image_publish_size_gib),
        );
        args.build_arg(
            "PARTITION_PLAN",
            match partition_plan {
                PartitionPlan::Split => "split",
                PartitionPlan::Unified => "unified",
            },
        );
        args.build_arg(
            "KERNEL_PARAMETERS",
            kernel_parameters
                .map(|v| v.join(" "))
                .unwrap_or_else(|| "".to_string()),
        );

        if let Some(image_features) = image_features {
            for image_feature in image_features.iter() {
                args.build_arg(format!("{}", image_feature), "1");
            }
        }

        // Add known secrets to the build argments.
        add_secrets(&mut args)?;

        // Always rebuild variants since they are located in a different workspace,
        // and don't directly track changes in the underlying packages.
        getenv("BUILDSYS_TIMESTAMP")?;

        let tag = format!(
            "buildsys-var-{variant}-{arch}",
            variant = variant,
            arch = arch
        );

        build(BuildType::Variant, &variant, &arch, args, &tag, &output_dir)?;

        Ok(Self)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

enum BuildType {
    Package,
    Variant,
}

/// Invoke a series of `docker` commands to drive a package or variant build.
fn build(
    kind: BuildType,
    what: &str,
    arch: &str,
    build_args: Vec<String>,
    tag: &str,
    output_dir: &PathBuf,
) -> Result<()> {
    // Our Dockerfile is in the top-level directory.
    let root = getenv("BUILDSYS_ROOT_DIR")?;
    env::set_current_dir(&root).context(error::DirectoryChangeSnafu { path: &root })?;

    // Compute a per-checkout prefix for the tag to avoid collisions.
    let mut d = Sha512::new();
    d.update(&root);
    let digest = hex::encode(d.finalize());
    let token = &digest[..12];
    let tag = format!("{}-{}", tag, token);

    // Our SDK and toolchain are picked by the external `cargo make` invocation.
    let sdk = getenv("BUILDSYS_SDK_IMAGE")?;
    let toolchain = getenv("BUILDSYS_TOOLCHAIN")?;

    // Avoid using a cached layer from a previous build.
    let nocache = rand::thread_rng().gen::<u32>();

    // Create a directory for tracking outputs before we move them into position.
    let build_dir = create_build_dir(&kind, what, arch)?;

    // Clean up any previous outputs we have tracked.
    clean_build_files(&build_dir, output_dir)?;

    let target = match kind {
        BuildType::Package => "package",
        BuildType::Variant => "variant",
    };

    let mut build = format!(
        "build . \
        --target {target} \
        --tag {tag}",
        target = target,
        tag = tag,
    )
    .split_string();

    build.extend(build_args);
    build.build_arg("SDK", sdk);
    build.build_arg("TOOLCHAIN", toolchain);
    build.build_arg("NOCACHE", nocache.to_string());
    // Avoid using a cached layer from a concurrent build in another checkout.
    build.build_arg("TOKEN", token);

    let create = format!("create --name {} {} true", tag, tag).split_string();
    let cp = format!("cp {}:/output/. {}", tag, build_dir.display()).split_string();
    let rm = format!("rm --force {}", tag).split_string();
    let rmi = format!("rmi --force {}", tag).split_string();

    // Clean up the stopped container if it exists.
    let _ = docker(&rm, Retry::No);

    // Clean up the previous image if it exists.
    let _ = docker(&rmi, Retry::No);

    // Build the image, which builds the artifacts we want.
    // Work around transient, known failure cases with Docker.
    docker(
        &build,
        Retry::Yes {
            attempts: DOCKER_BUILD_MAX_ATTEMPTS,
            messages: &[
                &*DOCKER_BUILD_FRONTEND_ERROR,
                &*DOCKER_BUILD_DEAD_RECORD_ERROR,
                &*UNEXPECTED_EOF_ERROR,
                &*CREATEREPO_C_READ_HEADER_ERROR,
            ],
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
    copy_build_files(&build_dir, output_dir)?;

    Ok(())
}

/// Run `docker` with the specified arguments.
fn docker(args: &[String], retry: Retry) -> Result<Output> {
    let mut max_attempts: u16 = 1;
    let mut retry_messages: &[&Regex] = &[];
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
            .context(error::CommandStartSnafu)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", &stdout);
        if output.status.success() {
            return Ok(output);
        }

        ensure!(
            retry_messages.iter().any(|m| m.is_match(&stdout)) && attempt < max_attempts,
            error::DockerExecutionSnafu {
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
        messages: &'a [&'static Regex],
    },
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Add secrets that might be needed for builds. Since most builds won't use
/// them, they are not automatically tracked for changes. If necessary, builds
/// can emit the relevant cargo directives for tracking in their build script.
fn add_secrets(args: &mut Vec<String>) -> Result<()> {
    let sbkeys_var = "BUILDSYS_SBKEYS_PROFILE_DIR";
    let sbkeys_dir = env::var(sbkeys_var).context(error::EnvironmentSnafu { var: sbkeys_var })?;

    let sbkeys = read_dir(&sbkeys_dir).context(error::DirectoryReadSnafu { path: &sbkeys_dir })?;
    for s in sbkeys {
        let s = s.context(error::DirectoryReadSnafu { path: &sbkeys_dir })?;
        args.build_secret(
            "file",
            &s.file_name().to_string_lossy(),
            &s.path().to_string_lossy(),
        );
    }

    for var in &[
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
    ] {
        let id = format!("{}.env", var.to_lowercase().replace('_', "-"));
        args.build_secret("env", &id, var);
    }

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Create a directory for build artifacts.
fn create_build_dir(kind: &BuildType, name: &str, arch: &str) -> Result<PathBuf> {
    let prefix = match kind {
        BuildType::Package => "packages",
        BuildType::Variant => "variants",
    };

    let path = [&getenv("BUILDSYS_STATE_DIR")?, arch, prefix, name]
        .iter()
        .collect();

    fs::create_dir_all(&path).context(error::DirectoryCreateSnafu { path: &path })?;

    Ok(path)
}

const MARKER_EXTENSION: &str = ".buildsys_marker";

/// Copy build artifacts to the output directory.
/// Before we copy each file, we create a corresponding marker file to record its existence.
fn copy_build_files<P>(build_dir: P, output_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    fn has_artifacts(entry: &DirEntry) -> bool {
        let is_dir = entry.path().is_dir();
        let is_file = entry.file_type().is_file();
        let is_not_marker = is_file
            && entry
                .file_name()
                .to_str()
                .map(|s| !s.ends_with(MARKER_EXTENSION))
                .unwrap_or(false);
        let is_symlink = entry.file_type().is_symlink();
        is_dir || is_not_marker || is_symlink
    }

    for artifact_file in find_files(&build_dir, has_artifacts) {
        let mut marker_file = artifact_file.clone().into_os_string();
        marker_file.push(MARKER_EXTENSION);
        File::create(&marker_file).context(error::FileCreateSnafu { path: &marker_file })?;

        let mut output_file: PathBuf = output_dir.as_ref().into();
        output_file.push(artifact_file.strip_prefix(&build_dir).context(
            error::StripPathPrefixSnafu {
                path: &marker_file,
                prefix: build_dir.as_ref(),
            },
        )?);

        let parent_dir = output_file
            .parent()
            .context(error::BadDirectorySnafu { path: &output_file })?;
        fs::create_dir_all(parent_dir)
            .context(error::DirectoryCreateSnafu { path: &parent_dir })?;

        fs::rename(&artifact_file, &output_file).context(error::FileRenameSnafu {
            old_path: &artifact_file,
            new_path: &output_file,
        })?;
    }

    Ok(())
}

/// Remove build artifacts from the output directory.
/// Any marker file we find could have a corresponding file that should be cleaned up.
/// We also clean up the marker files so they do not accumulate across builds.
/// For the same reason, if a directory is empty after build artifacts, marker files, and other
/// empty directories have been removed, then that directory will also be removed.
fn clean_build_files<P>(build_dir: P, output_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let build_dir = build_dir.as_ref();
    let output_dir = output_dir.as_ref();

    fn has_markers(entry: &DirEntry) -> bool {
        let is_dir = entry.path().is_dir();
        let is_file = entry.file_type().is_file();
        let is_marker = is_file
            && entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(MARKER_EXTENSION))
                .unwrap_or(false);
        is_dir || is_marker
    }

    fn cleanup(path: &Path, top: &Path, dirs: &mut HashSet<PathBuf>) -> Result<()> {
        if !path.exists() && !path.is_symlink() {
            return Ok(());
        }
        std::fs::remove_file(path).context(error::FileRemoveSnafu { path })?;
        let mut parent = path.parent();
        while let Some(p) = parent {
            if p == top || dirs.contains(p) {
                break;
            }
            dirs.insert(p.into());
            parent = p.parent()
        }
        Ok(())
    }

    fn is_empty_dir(path: &Path) -> Result<bool> {
        Ok(path.is_dir()
            && path
                .read_dir()
                .context(error::DirectoryReadSnafu { path })?
                .next()
                .is_none())
    }

    let mut clean_dirs: HashSet<PathBuf> = HashSet::new();

    for marker_file in find_files(&build_dir, has_markers) {
        let mut output_file: PathBuf = output_dir.into();
        output_file.push(marker_file.strip_prefix(build_dir).context(
            error::StripPathPrefixSnafu {
                path: &marker_file,
                prefix: build_dir,
            },
        )?);
        output_file.set_extension("");
        cleanup(&output_file, output_dir, &mut clean_dirs)?;
        cleanup(&marker_file, build_dir, &mut clean_dirs)?;
    }

    // Clean up directories in reverse order, so that empty child directories don't stop an
    // otherwise empty parent directory from being removed.
    let mut clean_dirs = clean_dirs.into_iter().collect::<Vec<PathBuf>>();
    clean_dirs.sort_by(|a, b| b.cmp(a));

    for clean_dir in clean_dirs {
        if is_empty_dir(&clean_dir)? {
            std::fs::remove_dir(&clean_dir)
                .context(error::DirectoryRemoveSnafu { path: &clean_dir })?;
        }
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
        .into_iter()
        .filter_entry(filter)
        .flat_map(|e| e.context(error::DirectoryWalkSnafu))
        .map(|e| e.into_path())
        .filter(|e| e.is_file() || e.is_symlink())
}

/// Retrieve a BUILDSYS_* variable that we expect to be set in the environment,
/// and ensure that we track it for changes, since it will directly affect the
/// output.
fn getenv(var: &str) -> Result<String> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var(var).context(error::EnvironmentSnafu { var })
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Helper trait for constructing buildkit --build-arg arguments.
trait BuildArg {
    fn build_arg<S1, S2>(&mut self, key: S1, value: S2)
    where
        S1: AsRef<str>,
        S2: AsRef<str>;
}

impl BuildArg for Vec<String> {
    fn build_arg<S1, S2>(&mut self, key: S1, value: S2)
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        self.push("--build-arg".to_string());
        self.push(format!("{}={}", key.as_ref(), value.as_ref()));
    }
}

/// Helper trait for constructing buildkit --secret arguments.
trait BuildSecret {
    fn build_secret<S>(&mut self, typ: S, id: S, src: S)
    where
        S: AsRef<str>;
}

impl BuildSecret for Vec<String> {
    fn build_secret<S>(&mut self, typ: S, id: S, src: S)
    where
        S: AsRef<str>,
    {
        self.push("--secret".to_string());
        self.push(format!(
            "type={},id={},src={}",
            typ.as_ref(),
            id.as_ref(),
            src.as_ref()
        ));
    }
}

/// Helper trait for splitting a string on spaces into owned Strings.
///
/// If you need an element with internal spaces, you should handle that separately, for example
/// with BuildArg.
trait SplitString {
    fn split_string(&self) -> Vec<String>;
}

impl<S> SplitString for S
where
    S: AsRef<str>,
{
    fn split_string(&self) -> Vec<String> {
        self.as_ref().split(' ').map(String::from).collect()
    }
}
