//! The govc module handles the process of building and executing the calls to Docker in order to
//! run specific `govc` commands.
use duct::cmd;
use log::trace;
use pubsys_config::vmware::{Datacenter, DatacenterCreds};
use snafu::ResultExt;
use std::env;
use std::path::Path;
use std::process::Output;

pub(crate) struct Govc {
    env_config: Vec<String>,
}

impl Govc {
    const GOVC: &'static str = "govc";

    /// Make a new instance of `Govc`, creating all of the environment variables required to run
    /// `govc` as Docker `--env` arguments
    pub(crate) fn new(dc: Datacenter, creds: DatacenterCreds) -> Self {
        let mut env_config = Vec::new();
        env_config.env_arg("GOVC_USERNAME", creds.username);
        env_config.env_arg("GOVC_PASSWORD", creds.password);
        env_config.env_arg("GOVC_URL", dc.vsphere_url);
        env_config.env_arg("GOVC_DATACENTER", dc.datacenter);
        env_config.env_arg("GOVC_DATASTORE", dc.datastore);
        env_config.env_arg("GOVC_NETWORK", dc.network);
        env_config.env_arg("GOVC_RESOURCE_POOL", dc.resource_pool);
        env_config.env_arg("GOVC_FOLDER", dc.folder);

        Self { env_config }
    }

    /// Run `govc import.ova` using Docker.
    ///
    /// Using the given name, OVA path, and import spec path, this function builds the `govc
    /// import.ova` command as it will be used in the container.  It also builds the necessary bind
    /// mount arguments to mount the import spec and OVA into the container.  Finally, it calls
    /// `govc` via `docker run` invocation using these arguments.
    pub(crate) fn upload_ova<S, P1, P2>(
        self,
        name: S,
        ova_path: P1,
        import_spec_path: P2,
    ) -> Result<Output>
    where
        S: AsRef<str>,
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let name = name.as_ref();
        let ova_host_path = ova_path.as_ref();
        let import_spec_host_path = import_spec_path.as_ref();

        // Define the paths to the OVA and import spec we will use for the bind mounts into the
        // container
        let ova_container_path = "/tmp/bottlerocket.ova";
        let import_spec_container_path = "/tmp/import.spec";

        //--mount type=bind,source="path/to/thing",target=/tmp/thing,readonly
        let mount_config = &[
            // Mount the import spec file
            "--mount",
            &format!(
                "type=bind,source={},target={},readonly",
                import_spec_host_path.display(),
                import_spec_container_path
            ),
            // Mount the OVA
            "--mount",
            &format!(
                "type=bind,source={},target={},readonly",
                ova_host_path.display(),
                ova_container_path
            ),
        ];

        // govc import.ova -options=/path/to/spec -name bottlerocket_vm_name /path/to/ova
        let govc_cmd = &[
            Self::GOVC,
            "import.ova",
            &format!("-options={}", import_spec_container_path),
            "-name",
            name,
            ova_container_path,
        ];

        let env_config: Vec<&str> = self.env_config.iter().map(|s| s.as_ref()).collect();

        docker_run(&env_config, Some(mount_config), govc_cmd)
    }
}

/// Execute `docker run` using the SDK container with the specified environment, mount, and command
/// arguments.
///
/// This builds the entire `docker run` command string using a list of Docker `--env FOO=BAR`
/// strings, an optional list of `--mount` strings, and a list of strings meant to be the command
/// to run in the container.
// The arguments are `&[&str]` in an attempt to be as flexible as possible for the caller
fn docker_run(docker_env: &[&str], mount: Option<&[&str]>, command: &[&str]) -> Result<Output> {
    let sdk = env::var("BUILDSYS_SDK_IMAGE").context(error::EnvironmentSnafu {
        var: "BUILDSYS_SDK_IMAGE",
    })?;
    trace!("SDK image: {}", sdk);

    let mut args = vec!["run"];
    args.push("--net=host");
    args.extend(docker_env);

    if let Some(mount_cfg) = mount {
        args.extend(mount_cfg)
    }

    args.push(&sdk);
    args.extend(command);

    let output = cmd("docker", args)
        .stderr_to_stdout()
        .stdout_capture()
        .unchecked()
        .run()
        .context(error::CommandStartSnafu)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    trace!("{}", stdout);
    if output.status.success() {
        Ok(output)
    } else {
        error::DockerSnafu { output: stdout }.fail()
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Helper trait for constructing Docker `--env` arguments
trait EnvArg {
    fn env_arg<S1, S2>(&mut self, key: S1, value: S2)
    where
        S1: AsRef<str>,
        S2: AsRef<str>;
}

impl EnvArg for Vec<String> {
    fn env_arg<S1, S2>(&mut self, key: S1, value: S2)
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        self.push("--env".to_string());
        self.push(format!("{}={}", key.as_ref(), value.as_ref()))
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

pub(crate) mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to start command: {}", source))]
        CommandStart { source: std::io::Error },

        #[snafu(display("Docker invocation failed: {}", output))]
        Docker { output: String },

        #[snafu(display("Missing environment variable '{}'", var))]
        Environment {
            var: String,
            source: std::env::VarError,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
