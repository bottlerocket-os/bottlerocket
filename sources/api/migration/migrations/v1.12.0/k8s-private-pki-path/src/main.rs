use migration_helpers::{migrate, Migration, MigrationData, Result};
use std::process;

const SETTING: &str = "configuration-files.kubelet-server-key.path";
const OLD_VALUE: &str = "/etc/kubernetes/pki/kubelet-server.key";
const NEW_VALUE: &str = "/etc/kubernetes/pki/private/kubelet-server.key";

/// We moved the render output location for the kubelet PKI private key to be in a restricted
/// subdirectory. We need to update this output path in the stored configuration so updated nodes
/// pick up the change.
fn run() -> Result<()> {
    migrate(KubeletServerKey {})
}

pub struct KubeletServerKey {}

impl KubeletServerKey {
    fn migrate(&mut self, mut input: MigrationData, action: &'static str) -> Result<MigrationData> {
        let old_value;
        let new_value;
        if action == "upgrade" {
            old_value = OLD_VALUE;
            new_value = NEW_VALUE;
        } else {
            // Downgrade: everything old is new again
            old_value = NEW_VALUE;
            new_value = OLD_VALUE;
        }

        if let Some(data) = input.data.get_mut(SETTING) {
            match data {
                serde_json::Value::String(current_value) => {
                    if current_value == old_value {
                        *data = new_value.into();
                        println!(
                            "Changed '{}' from {:?} to {:?} on {}",
                            SETTING, old_value, new_value, action
                        );
                    } else {
                        println!(
                            "'{}' is already set to {:?}, leaving alone",
                            SETTING, new_value
                        );
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'; KubeletServerKey only handles strings",
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

impl Migration for KubeletServerKey {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        self.migrate(input, "upgrade")
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        self.migrate(input, "downgrade")
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
