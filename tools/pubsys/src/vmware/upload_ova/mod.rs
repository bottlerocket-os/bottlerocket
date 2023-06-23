//! The upload_ova module owns the 'upload_ova' subcommand and is responsible for collating all of
//! the config necessary to upload an OVA bundle to VMware datacenters.
use crate::vmware::govc::Govc;
use crate::Args;
use clap::Parser;
use log::{debug, info, trace};
use pubsys_config::vmware::{
    Datacenter, DatacenterBuilder, DatacenterCreds, DatacenterCredsBuilder, DatacenterCredsConfig,
    VMWARE_CREDS_PATH,
};
use pubsys_config::InfraConfig;
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tinytemplate::TinyTemplate;

const SPEC_TEMPLATE_NAME: &str = "spec_template";

/// Uploads a Bottlerocket OVA to VMware datacenters
#[derive(Debug, Parser)]
pub(crate) struct UploadArgs {
    /// Path to the OVA image
    #[arg(short = 'o', long)]
    ova: PathBuf,

    /// Path to the import spec
    #[arg(short = 's', long)]
    spec: PathBuf,

    /// The desired VM name
    #[arg(short = 'n', long)]
    name: String,

    /// Make the uploaded OVA a VM template
    #[arg(long)]
    mark_as_template: bool,

    /// Datacenters to which you want to upload the OVA
    #[arg(long, value_delimiter = ',')]
    datacenters: Vec<String>,
}

/// Common entrypoint from main()
pub(crate) fn run(args: &Args, upload_args: &UploadArgs) -> Result<()> {
    // If a lock file exists, use that, otherwise use Infra.toml or default
    let infra_config = InfraConfig::from_path_or_lock(&args.infra_config_path, true)
        .context(error::InfraConfigSnafu)?;
    trace!("Using infra config: {:?}", infra_config);

    let vmware = infra_config
        .vmware
        .context(error::MissingConfigSnafu { missing: "vmware" })?;

    // If the user gave an override list of datacenters, use it, otherwise use what's in the config
    let upload_datacenters = if !upload_args.datacenters.is_empty() {
        &upload_args.datacenters
    } else {
        &vmware.datacenters
    };
    ensure!(
        !upload_datacenters.is_empty(),
        error::MissingConfigSnafu {
            missing: "vmware.datacenters"
        }
    );

    // Retrieve credentials from GOVC_ environment variables
    let creds_env = DatacenterCredsBuilder::from_env();
    // Retrieve credentials from file. The `home` crate is used to construct the VMWARE_CREDS_PATH,
    // and it's possible (however unlikely) that it is unable to determine the user's home folder.
    let creds_file = if let Some(ref creds_file) = *VMWARE_CREDS_PATH {
        if creds_file.exists() {
            info!("Using vSphere credentials file at {}", creds_file.display());
            DatacenterCredsConfig::from_path(creds_file).context(error::VmwareConfigSnafu)?
        } else {
            info!("vSphere credentials file not found, will attempt to use environment");
            DatacenterCredsConfig::default()
        }
    } else {
        info!("Unable to determine vSphere credentials file location, will attempt to use environment");
        DatacenterCredsConfig::default()
    };

    // Retrieve datacenter-related GOVC_ environment variables and any common configuration given
    // via Infra.toml
    let dc_env = DatacenterBuilder::from_env();
    let dc_common = vmware.common.as_ref();

    // Read the import spec as a template
    let import_spec_str = fs::read_to_string(&upload_args.spec).context(error::FileSnafu {
        action: "read",
        path: &upload_args.spec,
    })?;
    let mut tt = TinyTemplate::new();
    tt.add_template(SPEC_TEMPLATE_NAME, &import_spec_str)
        .context(error::AddTemplateSnafu {
            path: &upload_args.spec,
        })?;

    info!(
        "Uploading to datacenters: {}",
        &upload_datacenters.join(", ")
    );
    for dc in upload_datacenters {
        debug!("Building config for {}", &dc);
        // If any specific configuration exists for this datacenter, retrieve it from VMware
        // config.  Then build out a complete datacenter config with all values necessary to
        // interact with VMware.  Environment variables trump all others, so start with those, then
        // fill in any missing items with datacenter-specific configuration and any common
        // configuration.
        let dc_config = vmware.datacenter.get(dc);
        trace!("{} config: {:?}", &dc, &dc_config);
        let datacenter: Datacenter = dc_env
            .take_missing_from(dc_config)
            .take_missing_from(dc_common)
            .build()
            .context(error::DatacenterBuildSnafu)?;

        // Use a similar pattern here for credentials; start with environment variables and fill in
        // any missing items with the datacenter-specific credentials from file.
        let dc_creds = creds_file.datacenter.get(dc);
        let creds: DatacenterCreds = creds_env
            .take_missing_from(dc_creds)
            .build()
            .context(error::CredsBuildSnafu)?;

        // Render the import spec with this datacenter's details and write to temp file
        let rendered_spec = render_spec(&tt, &datacenter.network, upload_args.mark_as_template)?;
        let import_spec = NamedTempFile::new().context(error::TempFileSnafu)?;
        fs::write(import_spec.path(), &rendered_spec).context(error::FileSnafu {
            action: "write",
            path: import_spec.path(),
        })?;
        trace!("Import spec: {}", &rendered_spec);

        if upload_args.mark_as_template {
            info!(
                "Uploading OVA to datacenter '{}' as template with name: '{}'",
                &dc, &upload_args.name
            );
        } else {
            info!(
                "Uploading OVA to datacenter '{}' with name '{}'",
                &dc, &upload_args.name
            );
        }

        Govc::new(datacenter, creds)
            .upload_ova(&upload_args.name, &upload_args.ova, import_spec)
            .context(error::UploadOvaSnafu)?;
    }

    Ok(())
}

/// Render the import spec template given the current network and template setting.
// This exists primarily to abstract the creation of the Context struct that is required by
// TinyTemplate; it's pretty ugly to do inline with the rest of the code.
fn render_spec<S>(tt: &TinyTemplate<'_>, network: S, mark_as_template: bool) -> Result<String>
where
    S: AsRef<str>,
{
    #[derive(Debug, Serialize)]
    struct Context {
        network: String,
        mark_as_template: bool,
    }

    let context = Context {
        network: network.as_ref().to_string(),
        mark_as_template,
    };

    tt.render(SPEC_TEMPLATE_NAME, &context)
        .context(error::RenderTemplateSnafu)
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Error building template from '{}': {}", path.display(), source))]
        AddTemplate {
            path: PathBuf,
            source: tinytemplate::error::Error,
        },

        #[snafu(display("Unable to build datacenter credentials: {}", source))]
        CredsBuild {
            source: pubsys_config::vmware::Error,
        },

        #[snafu(display("Unable to build datacenter config: {}", source))]
        DatacenterBuild {
            source: pubsys_config::vmware::Error,
        },

        #[snafu(display("Missing environment variable '{}'", var))]
        Environment {
            var: String,
            source: std::env::VarError,
        },

        #[snafu(display("Failed to {} '{}': {}", action, path.display(), source))]
        File {
            action: String,
            path: PathBuf,
            source: io::Error,
        },

        #[snafu(display("Error reading config: {}", source))]
        InfraConfig { source: pubsys_config::Error },

        #[snafu(display("Infra.toml is missing {}", missing))]
        MissingConfig { missing: String },

        #[snafu(display("Error rendering template: {}", source))]
        RenderTemplate { source: tinytemplate::error::Error },

        #[snafu(display("Failed to create temporary file: {}", source))]
        TempFile { source: io::Error },

        #[snafu(display("Error reading config: {}", source))]
        VmwareConfig {
            source: pubsys_config::vmware::Error,
        },

        #[snafu(display("Failed to upload OVA: {}", source))]
        UploadOva { source: crate::vmware::govc::Error },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
