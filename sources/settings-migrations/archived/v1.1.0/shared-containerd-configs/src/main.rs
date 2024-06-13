use lazy_static::lazy_static;
use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const SETTING: &'static str = "configuration-files.containerd-config-toml.template-path";

lazy_static! {
    static ref TEMPLATE_CHANGES: &'static [(&'static str, &'static str)] = &[
        (
            "/usr/share/templates/containerd-config-toml_aws-dev",
            "/usr/share/templates/containerd-config-toml_basic"
        ),
        (
            "/usr/share/templates/containerd-config-toml_aws-ecs-1",
            "/usr/share/templates/containerd-config-toml_basic"
        ),
        (
            "/usr/share/templates/containerd-config-toml_aws-k8s",
            "/usr/share/templates/containerd-config-toml_k8s"
        ),
        (
            "/usr/share/templates/containerd-config-toml_vmware-dev",
            "/usr/share/templates/containerd-config-toml_basic"
        ),
    ];
}

/// We refactored containerd config file templates to share data where possible, instead of
/// duplicating them for variants with identical configs.  thar-be-settings runs at startup and
/// regenerates all files based on templates, so if we change the source during migration (early in
/// boot) it'll automatically be written out based on the new template.
fn run() -> Result<()> {
    migrate(SharedContainerdConfigs {})
}

pub struct SharedContainerdConfigs {}

impl SharedContainerdConfigs {
    fn migrate(
        &mut self,
        mut input: MigrationData,
        transforms: &[(&str, &str)],
        action: &'static str,
    ) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(SETTING) {
            match data {
                serde_json::Value::String(string) => {
                    for (outgoing, incoming) in transforms {
                        if string == outgoing {
                            *data = (*incoming).into();
                            println!(
                                "Changed '{}' from {:?} to {:?} on {}",
                                SETTING, outgoing, incoming, action
                            );
                            // We've done what we came to do - the transformations don't
                            // overlap, so we do one at most.  (Without this, Rust knows that
                            // we still have a reference to 'data' for another iteration, and
                            // it won't let us change it.  So smart.)
                            break;
                        } else {
                            println!("'{}' is not set to {:?}, leaving alone", SETTING, outgoing);
                        }
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'; SharedContainerdConfigs only handles strings",
                        SETTING, data
                    );
                }
            }
        } else {
            println!("Found no setting '{}'", SETTING);
        }

        Ok(input)
    }
}

impl Migration for SharedContainerdConfigs {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        self.migrate(input, *TEMPLATE_CHANGES, "upgrade")
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        let transforms: Vec<(&str, &str)> =
            TEMPLATE_CHANGES.iter().map(|(a, b)| (*b, *a)).collect();
        self.migrate(input, &transforms, "downgrade")
    }
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
