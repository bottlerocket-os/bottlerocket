/*!
driverdog is a tool to link kernel modules at runtime. It uses a toml configuration file with the following shape:

`lib-modules-path`: destination path for the .ko linked files
`objects-source`: path where the objects used to link the kernel module are
`object-files`: hash with the object files to be linked, each object in the map should include the files used to link it
`kernel-modules`: hash with the kernel modules to be linked, each kernel module in the map should include the files used to link it
*/

#[macro_use]
extern crate log;

use argh::FromArgs;
use serde::Deserialize;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

/// Path to the drivers configuration to use
const DEFAULT_DRIVER_CONFIG_PATH: &str = "/etc/drivers/";

/// Path where the linked kernel modules will be created
const LIB_MODULES_PATH: &str = "/lib/modules";
/// Path to the kernel sources
const KERNEL_SOURCES: &str = "/usr/src/kernels";
/// Path to the uname bin
const UNAME_BIN_PATH: &str = "/usr/bin/uname";

/// Paths to kmod utilities
const DEPMOD_BIN_PATH: &str = "/usr/bin/depmod";
const MODPROBE_BIN_PATH: &str = "/usr/bin/modprobe";

/// Paths to binutils tools
const LD_BIN_PATH: &str = "/usr/bin/ld";
const STRIP_BIN_PATH: &str = "/usr/bin/strip";

/// Stores arguments
#[derive(FromArgs, PartialEq, Debug)]
struct Args {
    /// log-level trace|debug|info|warn|error
    #[argh(option)]
    log_level: Option<LevelFilter>,
    /// configuration file with the modules to be linked
    #[argh(
        option,
        default = "DEFAULT_DRIVER_CONFIG_PATH.to_string()",
        short = 'd'
    )]
    driver_config_path: String,
    #[argh(subcommand)]
    subcommand: Subcommand,
    /// the modules set used to operate
    #[argh(option)]
    modules_set: Option<String>,
}

/// Stores the subcommand to be executed
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand)]
enum Subcommand {
    LinkModules(LinkModulesArgs),
    LoadModules(LoadModulesArgs),
}

/// Links the kernel modules
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "link-modules")]
struct LinkModulesArgs {}

/// Loads the kernel modules
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "load-modules")]
struct LoadModulesArgs {}

/// Holds the configuration used to link kernel modules
#[derive(Deserialize, Debug)]
struct DriverConfig {
    #[serde(rename(deserialize = "lib-modules-path"))]
    lib_modules_path: String,
    #[serde(rename(deserialize = "objects-source"))]
    objects_source: String,
    #[serde(rename(deserialize = "kernel-modules"))]
    kernel_modules: HashMap<String, Linkable>,
    #[serde(rename(deserialize = "object-files"))]
    object_files: HashMap<String, Linkable>,
}

/// Holds the objects to be linked for the object/kernel module
#[derive(Deserialize, Debug)]
struct Linkable {
    #[serde(rename(deserialize = "link-objects"))]
    link_objects: Vec<String>,
}

// Links the modules in the modules sets
fn link_modules_sets(
    modules_sets: &HashMap<String, DriverConfig>,
    target: Option<String>,
) -> Result<()> {
    // Get current kernel version
    let kernel_version = get_kernel_version()?;

    // If the target module set was given, link the kernel modules in it
    if let Some(target) = target {
        let driver_config = modules_sets
            .get(&target)
            .context(error::MissingModuleSetSnafu { target })?;
        link_modules(driver_config, &kernel_version)?;
    } else {
        // Link all the modules sets if no target module was given
        for driver_config in modules_sets.values() {
            link_modules(driver_config, &kernel_version)?;
        }
    }

    Ok(())
}

// Links the kernel modules for the given configuration, and for the given kernel version
fn link_modules<S>(driver_config: &DriverConfig, kernel_version: S) -> Result<()>
where
    S: AsRef<str>,
{
    let kernel_version = kernel_version.as_ref();
    // The directory with the module's objects
    let driver_path = Path::new(&driver_config.objects_source).to_path_buf();
    // Destination for the kernel modules
    let modules_path = Path::new(LIB_MODULES_PATH)
        .join(kernel_version)
        .join(&driver_config.lib_modules_path);
    // Directory to store temp artifacts
    let build_dir = tempfile::tempdir().context(error::TmpDirSnafu)?;
    // This script is used to link the kernel module
    let common_module_script = Path::new(KERNEL_SOURCES)
        .join(kernel_version)
        .join("scripts/module.lds");

    // First, link the object files, and store them in the temp directory
    for (name, object_file) in driver_config.object_files.iter() {
        link_object_file(name, object_file, &build_dir, &driver_path)?;
    }

    // Next, link the kernel modules
    for (name, kernel_module) in driver_config.kernel_modules.iter() {
        link_kernel_module(
            name,
            kernel_module,
            &modules_path,
            &driver_path,
            &build_dir,
            &common_module_script,
        )?;
    }

    Ok(())
}

// Links the given kernel module
fn link_kernel_module<P, B, S>(
    name: S,
    kernel_module: &Linkable,
    modules_path: P,
    driver_path: P,
    build_dir: B,
    common_module_script_path: P,
) -> Result<()>
where
    S: AsRef<str>,
    B: AsRef<Path>,
    P: AsRef<Path>,
{
    let name = name.as_ref();
    let modules_path = modules_path.as_ref();
    let driver_path = driver_path.as_ref();
    let build_dir = build_dir.as_ref();
    let common_module_script_path = common_module_script_path.as_ref();
    // Destination for this kernel module
    let kernel_module_path = modules_path.join(name);

    // We make sure the dependencies are present in the build directory, otherwise attempt
    // to copy them from `driver_path`
    let mut dependencies_paths: Vec<String> = Vec::new();
    for object_file in kernel_module.link_objects.iter() {
        let object_file_path = build_dir.join(object_file);
        if !object_file_path.exists() {
            let from = driver_path.join(object_file);
            fs::copy(&from, &object_file_path).context(error::CopySnafu {
                from: &from,
                to: &object_file_path,
            })?;
        }
        dependencies_paths.push(object_file_path.to_string_lossy().into_owned());
    }

    // Link the kernel module
    let mut args = vec![
        "-r".to_string(),
        "-T".to_string(),
        common_module_script_path.display().to_string(),
        "--build-id".to_string(),
        "-o".to_string(),
        kernel_module_path
            .to_str()
            .context(error::InvalidModulePathSnafu {
                path: &kernel_module_path,
            })?
            .to_string(),
    ];
    args.append(&mut dependencies_paths);

    command(LD_BIN_PATH, &args)?;
    info!("Linked {}", name);

    Ok(())
}

/// Links the given object file
fn link_object_file<P, B, S>(
    name: S,
    object_file: &Linkable,
    build_dir: B,
    driver_path: P,
) -> Result<()>
where
    S: AsRef<str>,
    B: AsRef<Path>,
    P: AsRef<Path>,
{
    let name = name.as_ref();
    let build_dir = build_dir.as_ref();
    let driver_path = driver_path.as_ref();

    // Temporary files are created in build_dir
    let object_path = Path::new(build_dir)
        .join(name)
        .to_string_lossy()
        .into_owned();
    // Paths to the dependencies for this object file
    let mut dependencies = object_file
        .link_objects
        .iter()
        .map(|d| {
            Path::new(driver_path)
                .join(d)
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    // Link the object file
    let mut args = vec!["-r".to_string(), "-o".to_string(), object_path.clone()];
    args.append(&mut dependencies);

    command(LD_BIN_PATH, &args)?;
    info!("Linked object '{}'", name);

    // Strip the object file
    command(
        STRIP_BIN_PATH,
        [
            "-g",
            "--strip-unneeded",
            "--keep-symbol",
            "init_module",
            "--keep-symbol",
            "cleanup_module",
            &object_path,
        ],
    )?;
    info!("Stripped object '{}'", name);

    Ok(())
}

// Loads the modules in the modules sets
fn load_modules_sets(
    modules_sets: &HashMap<String, DriverConfig>,
    target: Option<String>,
) -> Result<()> {
    // Update the modules.dep before we attempt to load kernel modules
    let args: Vec<String> = Vec::new();
    command(DEPMOD_BIN_PATH, args)?;
    info!("Updated modules dependencies");

    // If the target module set was given, load the kernel modules in it
    if let Some(target) = target {
        let driver_config = modules_sets
            .get(&target)
            .context(error::MissingModuleSetSnafu { target })?;
        load_modules(driver_config)?
    }
    // Load all the modules sets if no target module was given
    for driver_config in modules_sets.values() {
        load_modules(driver_config)?;
    }

    Ok(())
}

fn load_modules(driver_config: &DriverConfig) -> Result<()> {
    let mut kernel_modules: Vec<String> = driver_config
        .kernel_modules
        .keys()
        .map(|k| k.split('.').collect::<Vec<&str>>()[0].to_string())
        .collect();

    // Load kernel modules
    let mut args = vec!["-a".to_string()];
    args.append(&mut kernel_modules);
    command(MODPROBE_BIN_PATH, &args)?;
    info!("Loaded kernel modules");

    Ok(())
}

/// Returns the kernel version
fn get_kernel_version() -> Result<String> {
    Ok(command(UNAME_BIN_PATH, ["-r"])?.trim().to_string())
}

/// Wrapper around process::Command that adds error checking.
fn command<I, S>(bin_path: &str, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(bin_path);
    command.args(args);
    let output = command
        .output()
        .context(error::ExecutionFailureSnafu { command })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    trace!("stdout: {}", stdout);
    trace!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    ensure!(
        output.status.success(),
        error::CommandFailureSnafu { bin_path, output }
    );

    Ok(stdout)
}

fn setup_logger(args: &Args) -> Result<()> {
    let log_level = args.log_level.unwrap_or(LevelFilter::Info);
    SimpleLogger::init(log_level, LogConfig::default()).context(error::LoggerSnafu)
}

fn run() -> Result<()> {
    let args: Args = argh::from_env();
    setup_logger(&args)?;
    let driver_config_path = Path::new(&args.driver_config_path);
    let mut all_modules_sets: HashMap<String, DriverConfig> = HashMap::new();

    for entry in (driver_config_path
        .read_dir()
        .context(error::ReadPathSnafu {
            path: driver_config_path,
        })?)
    .flatten()
    {
        let path = entry.path();
        let modules_sets: HashMap<String, DriverConfig> =
            toml::from_slice(&fs::read(&path).context(error::ReadPathSnafu { path: &path })?)
                .context(error::DeserializeSnafu { path: &path })?;

        all_modules_sets.extend(modules_sets);
    }

    match args.subcommand {
        Subcommand::LinkModules(_) => link_modules_sets(&all_modules_sets, args.modules_set),
        Subcommand::LoadModules(_) => load_modules_sets(&all_modules_sets, args.modules_set),
    }
}

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        process::exit(1);
    }
}

/// ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡 ＜コ：ミ くコ:彡
mod error {
    use snafu::Snafu;
    use std::path::PathBuf;
    use std::process::{Command, Output};

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("'{}' failed - stderr: {}",
                        bin_path, String::from_utf8_lossy(&output.stderr)))]
        CommandFailure { bin_path: String, output: Output },

        #[snafu(display("Failed to copy '{}' to '{}': {}", from.display(), to.display(), source))]
        Copy {
            from: PathBuf,
            to: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to deserialize '{}': {}", path.display(), source))]
        Deserialize {
            path: PathBuf,
            source: toml::de::Error,
        },

        #[snafu(display("Failed to execute '{:?}': {}", command, source))]
        ExecutionFailure {
            command: Command,
            source: std::io::Error,
        },

        #[snafu(display("Module path '{}' is not UTF-8", path.display()))]
        InvalidModulePath { path: PathBuf },

        #[snafu(display("Failed to setup logger: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Invalid log level '{}'", log_level))]
        LogLevel {
            log_level: String,
            source: log::ParseLevelError,
        },

        #[snafu(display("Missing module set '{}'", target))]
        MissingModuleSet { target: String },

        #[snafu(display("Failed to read path '{}': '{}'", path.display(), source))]
        ReadPath {
            path: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Failed to create temporary directory: {}", source))]
        TmpDir { source: std::io::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
