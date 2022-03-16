/*!
Packages using the Go programming language may have upstream tar archives that
include only the source code of the project, but not the source code of any
dependencies.  The Go programming language promotes the use of "modules" for
dependencies and projects adopting modules will provide go.mod and go.sum
files.

This module provides the ability to retrieve and validate the dependencies
declared using Go modules given a tar archive containing a go.mod and go.sum.

The location where dependencies are retrieved from are controlled by the
standard environment variables employed by the Go tool: GOPROXY, GOSUMDB, and
GOPRIVATE.

 */

pub(crate) mod error;
use error::Result;

use super::manifest;
use duct::cmd;
use snafu::{OptionExt, ResultExt};
use std::env;
use std::path::PathBuf;
use std::process::Output;

pub(crate) struct GoMod;

impl GoMod {
    pub(crate) fn vendor(
        root_dir: &PathBuf,
        package_dir: &PathBuf,
        gomods: &[manifest::GoModule],
    ) -> Result<Self> {
        for g in gomods {
            let input_path_arg = g.input.as_ref().context(error::InputFileSnafu)?;
            let input_path = package_dir.join(input_path_arg);
            if !input_path.is_file() {
                return Err(error::Error::InputFileBad);
            }
            let mod_dir = g.mod_dir.as_ref().context(error::ModDirSnafu)?;
            let output_dir_arg = g.output_dir.as_ref().context(error::OutputDirSnafu)?;
            let output_dir = package_dir.join(output_dir_arg);
            if output_dir.exists() && !output_dir.is_dir() {
                return Err(error::Error::OutputDirBad);
            }

            // Our SDK and toolchain are picked by the external `cargo make` invocation.
            let sdk = getenv("BUILDSYS_SDK_IMAGE")?;

            // Several Go variables control proxying
            let goproxy = go_env("GOPROXY").unwrap_or("".to_string());
            let gosumdb = go_env("GOSUMDB").unwrap_or("".to_string());
            let goprivate = go_env("GOPRIVATE").unwrap_or("".to_string());

            let args = DockerGoArgs {
                module_path: package_dir.clone(),
                sdk_image: sdk,
                go_mod_cache: root_dir.join(".gomodcache".to_string()),
                command: format!(
                    "mkdir -p {outdir}
                    tar zxf {input} -C {outdir} &&
                    cd {outdir}/{moddir} &&
                    export GOPROXY={goproxy} &&
                    export GOSUMDB={gosumdb} &&
                    export GOPRIVATE={goprivate} &&
                    go list -mod=readonly ./... >/dev/null && go mod vendor",
                    input = input_path_arg.to_string_lossy(),
                    outdir = output_dir_arg.to_string_lossy(),
                    moddir = mod_dir.to_string_lossy(),
                    goproxy = goproxy,
                    gosumdb = gosumdb,
                    goprivate = goprivate,
                ),
            };
            match docker_go(root_dir, &args) {
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        return Ok(Self);
    }
}

struct DockerGoArgs {
    module_path: PathBuf,
    sdk_image: String,
    go_mod_cache: PathBuf,
    command: String,
}

/// Run `docker-go` with the specified arguments.
fn docker_go(root_dir: &PathBuf, dg_args: &DockerGoArgs) -> Result<Output> {
    let args = vec![
        "--module-path",
        dg_args
            .module_path
            .to_str()
            .context(error::InputFileSnafu)
            .unwrap(),
        "--sdk-image",
        &dg_args.sdk_image,
        "--go-mod-cache",
        dg_args
            .go_mod_cache
            .to_str()
            .context(error::InputFileSnafu)
            .unwrap(),
        "--command",
        &dg_args.command,
    ];
    let program = root_dir.join("tools/docker-go");
    println!("program: {}", program.to_string_lossy());
    let output = cmd(program, args)
        .stderr_to_stdout()
        .stdout_capture()
        .unchecked()
        .run()
        .context(error::CommandStartSnafu)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", &stdout);
    return if output.status.success() {
        Ok(output)
    } else {
        Err(error::Error::DockerExecution {
            args: "".to_string(),
        })
    };
}

/// Run `go env` with the specified argument.
fn go_env(var: &str) -> Option<String> {
    let args = vec!["env", var];
    let output = match cmd("go", args)
        .stderr_to_stdout()
        .stdout_capture()
        .unchecked()
        .run()
    {
        Ok(v) => v,
        Err(_) => return None,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", &stdout);
    return if output.status.success() {
        Some(stdout.to_string())
    } else {
        None
    };
}

/// Retrieve a BUILDSYS_* variable that we expect to be set in the environment,
/// and ensure that we track it for changes, since it will directly affect the
/// output.
fn getenv(var: &str) -> Result<String> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var(var).context(error::EnvironmentSnafu { var })
}
