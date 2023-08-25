/*!
Packages using the Go programming language may have upstream tar archives that
include only the source code of the project, but not the source code of any
dependencies. The Go programming language promotes the use of "modules" for
dependencies. Projects adopting modules will provide `go.mod` and `go.sum` files.

This Rust module extends the functionality of `packages.metadata.build-package.external-files`
and provides the ability to retrieve and validate dependencies
declared using Go modules given a tar archive containing a `go.mod` and `go.sum`.

The location where dependencies are retrieved from are controlled by the
standard environment variables employed by the Go tool: `GOPROXY`, `GOSUMDB`, and
`GOPRIVATE`. These variables are automatically retrieved from the host environment
when the docker-go script is invoked.

 */

pub(crate) mod error;
use error::Result;

use buildsys::manifest;
use duct::cmd;
use snafu::{ensure, OptionExt, ResultExt};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub(crate) struct GoMod;

const GO_MOD_DOCKER_SCRIPT_NAME: &str = "docker-go-script.sh";

// The following bash template script is intended to be run within a container
// using the docker-go tool found in this codebase under `tools/docker-go`.
//
// This script inspects the top level directory found in the package upstream
// archive and uses that as the default Go module path if no explicit module
// path was provided. It will then untar the archive, vendor the Go
// dependencies, create a new archive using the {module-path}/vendor directory
// and name it the output path provided. If no output path was given, it
// defaults to "bundled-{package-file-name}". Finally, it cleans up by removing
// the untar'd source code. The upstream archive remains intact and both tar
// files can then be used during packaging.
//
// This script exists as an in memory template string literal and is populated
// into a temporary file in the package directory itself to enable buildsys to
// be as portable as possible and have no dependency on runtime paths. Since
// buildsys is executed from the context of many different package directories,
// managing a temporary file via this Rust module prevents having to acquire the
// path of some static script file on the host system.
const GO_MOD_SCRIPT_TMPL: &str = r#".#!/bin/bash

set -e

toplevel=$(tar tf __LOCAL_FILE_NAME__ | head -1)
if [ -z __MOD_DIR__ ] ; then
    targetdir="${toplevel}"
else
    targetdir="__MOD_DIR__"
fi

tar xf __LOCAL_FILE_NAME__

pushd "${targetdir}"
    go list -mod=readonly ./... >/dev/null && go mod vendor
popd

tar czf __OUTPUT__ "${targetdir}"/vendor
rm -rf "${targetdir}"
touch -r __LOCAL_FILE_NAME__ __OUTPUT__
"#;

impl GoMod {
    pub(crate) fn vendor(
        root_dir: &Path,
        package_dir: &Path,
        external_file: &manifest::ExternalFile,
    ) -> Result<()> {
        let url_file_name = extract_file_name(&external_file.url)?;
        let local_file_name = &external_file.path.as_ref().unwrap_or(&url_file_name);
        ensure!(
            local_file_name.components().count() == 1,
            error::InputFileSnafu
        );

        let full_path = package_dir.join(local_file_name);
        ensure!(
            full_path.is_file(),
            error::InputFileBadSnafu { path: full_path }
        );

        // If a module directory was not provided, set as an empty path.
        // By default, without a provided module directory, tar will be passed
        // the first directory found in the archives as the top level Go module
        let default_empty_path = PathBuf::from("");
        let mod_dir = external_file
            .bundle_root_path
            .as_ref()
            .unwrap_or(&default_empty_path);

        // Use a default "bundle-{name-of-file}" if no output path was provided
        let default_output_path =
            PathBuf::from(format!("bundled-{}", local_file_name.to_string_lossy()));
        let output_path_arg = external_file
            .bundle_output_path
            .as_ref()
            .unwrap_or(&default_output_path);
        println!(
            "cargo:rerun-if-changed={}",
            output_path_arg.to_string_lossy()
        );

        // Our SDK and toolchain are picked by the external `cargo make` invocation.
        let sdk = env::var("BUILDSYS_SDK_IMAGE").context(error::EnvironmentSnafu {
            var: "BUILDSYS_SDK_IMAGE",
        })?;

        let args = DockerGoArgs {
            module_path: package_dir,
            sdk_image: sdk,
            go_mod_cache: &root_dir.join(".gomodcache"),
            command: format!("./{}", GO_MOD_DOCKER_SCRIPT_NAME),
        };

        // Create and/or write the temporary script file to the package directory
        // using the script template string and placeholder variables
        let script_contents = GO_MOD_SCRIPT_TMPL
            .replace("__LOCAL_FILE_NAME__", &local_file_name.to_string_lossy())
            .replace("__MOD_DIR__", &mod_dir.to_string_lossy())
            .replace("__OUTPUT__", &output_path_arg.to_string_lossy());
        let script_path = format!(
            "{}/{}",
            package_dir.to_string_lossy(),
            GO_MOD_DOCKER_SCRIPT_NAME
        );

        // Drop the reference after writing the file to avoid a "text busy" error
        // when attempting to execute it.
        {
            let mut script_file = fs::File::create(&script_path)
                .context(error::CreateFileSnafu { path: &script_path })?;
            fs::set_permissions(&script_path, fs::Permissions::from_mode(0o777))
                .context(error::SetFilePermissionsSnafu { path: &script_path })?;
            script_file
                .write_all(script_contents.as_bytes())
                .context(error::WriteFileSnafu { path: &script_path })?;
        }

        let res = docker_go(root_dir, &args);
        fs::remove_file(&script_path).context(error::RemoveFileSnafu { path: &script_path })?;
        res
    }
}

fn extract_file_name(url: &str) -> Result<PathBuf> {
    let parsed = reqwest::Url::parse(url).context(error::InputUrlSnafu { url })?;
    let name = parsed
        .path_segments()
        .context(error::InputFileBadSnafu { path: url })?
        .last()
        .context(error::InputFileBadSnafu { path: url })?;
    Ok(name.into())
}

struct DockerGoArgs<'a> {
    module_path: &'a Path,
    sdk_image: String,
    go_mod_cache: &'a Path,
    command: String,
}

/// Run `docker-go` with the specified arguments.
fn docker_go(root_dir: &Path, dg_args: &DockerGoArgs) -> Result<()> {
    let args = vec![
        "--module-path",
        dg_args
            .module_path
            .to_str()
            .context(error::InputFileSnafu)?,
        "--sdk-image",
        &dg_args.sdk_image,
        "--go-mod-cache",
        dg_args
            .go_mod_cache
            .to_str()
            .context(error::InputFileSnafu)?,
        "--command",
        &dg_args.command,
    ];
    let arg_string = args.join(" ");
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
    ensure!(
        output.status.success(),
        error::DockerExecutionSnafu { args: arg_string }
    );
    Ok(())
}
