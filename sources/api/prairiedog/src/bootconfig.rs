use crate::error;
use crate::error::Result;
use crate::initrd::generate_initrd;
use model::modeled_types::{BootConfigKey, BootConfigValue};
use model::BootSettings;
use snafu::{ensure, ResultExt};
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;
use tokio::io;

// Boot config related consts
const BOOTCONFIG_INITRD_PATH: &str = "/var/lib/bottlerocket/bootconfig.data";
const PROC_BOOTCONFIG: &str = "/proc/bootconfig";
const DEFAULT_BOOTCONFIG_STR: &str = r#"
    kernel = ""
    init = ""
"#;

fn append_boot_config_value_list(values: &[BootConfigValue], output: &mut String) {
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        // If the value itself has double quotes in it, then we wrap the value with single-quotes
        if v.contains('\"') {
            output.push_str(&format!("\'{}\'", v));
        } else {
            output.push_str(&format!("\"{}\"", v));
        }
    }
}

/// Serializes `BootSettings` out to a multi-line string representation of the boot config that can be
/// loaded by the kernel
fn serialize_boot_settings_to_boot_config(boot_settings: &BootSettings) -> Result<String> {
    // Preallocate string buffer to avoid a bunch of memory allocation calls when we append to
    // the string buffer
    let mut output = String::with_capacity(128);
    if let Some(kernel_param) = &boot_settings.kernel_parameters {
        for (key, values) in kernel_param.iter() {
            output.push_str(&format!("kernel.{} = ", key));
            append_boot_config_value_list(values, &mut output);
            output.push('\n')
        }
    }
    if let Some(init_param) = &boot_settings.init_parameters {
        for (key, values) in init_param.iter() {
            output.push_str(&format!("init.{} = ", key));
            append_boot_config_value_list(values, &mut output);
            output.push('\n')
        }
    }
    Ok(output)
}

/// Queries Bottlerocket boot settings and generates initrd image file with boot config as the only data
pub(crate) async fn generate_boot_config<P>(socket_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let bootconfig_bytes = match get_boot_config_settings(socket_path).await? {
        Some(boot_settings) => {
            info!("Generating initrd boot config from boot settings");
            trace!("Boot settings: {:?}", boot_settings);
            let bootconfig = serialize_boot_settings_to_boot_config(&boot_settings)?;
            trace!("Serializing boot config string: {}", bootconfig);
            bootconfig.into_bytes()
        }
        None => {
            // If we don't have any boot settings, write out an initrd with default boot config contents
            trace!("Serializing boot config string: {}", DEFAULT_BOOTCONFIG_STR);
            DEFAULT_BOOTCONFIG_STR.to_string().into_bytes()
        }
    };
    let initrd = generate_initrd(&bootconfig_bytes)?;
    trace!("Writing initrd image file: {:?}", initrd);
    tokio::fs::write(BOOTCONFIG_INITRD_PATH, &initrd)
        .await
        .context(error::WriteInitrdSnafu)?;
    Ok(())
}

/// Retrieves boot config related Bottlerocket settings. If they don't exist in the settings model,
/// we return `None` instead.
async fn get_boot_config_settings<P>(socket_path: P) -> Result<Option<BootSettings>>
where
    P: AsRef<Path>,
{
    let uri = "/settings";
    let settings: serde_json::Value =
        schnauzer::get_json(socket_path, uri, Some(("prefix", "boot")))
            .await
            .context(error::RetrieveSettingsSnafu)?;

    match settings.get("boot") {
        None => Ok(None),
        Some(boot_settings_val) => Ok(Some(
            serde_json::from_value(boot_settings_val.to_owned())
                .context(error::BootSettingsFromJsonValueSnafu)?,
        )),
    }
}

/// Reads `/proc/bootconfig`. Not having any boot config is ignored.
async fn read_proc_bootconfig() -> Result<Option<String>> {
    match tokio::fs::read_to_string(PROC_BOOTCONFIG).await {
        Ok(s) => Ok(Some(s)),
        Err(e) => {
            // If there's no `/proc/bootconfig`, then the user hasn't provisioned any kernel boot configuration.
            if e.kind() == io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e).context(error::ReadFileSnafu {
                    path: PROC_BOOTCONFIG,
                })
            }
        }
    }
}

/// Reads `/proc/bootconfig` and populates the Bottlerocket boot settings based on the existing boot config data
pub(crate) async fn generate_boot_settings() -> Result<()> {
    if let Some(proc_bootconfig) = read_proc_bootconfig().await? {
        debug!(
            "Generating kernel boot config settings from `{}`",
            PROC_BOOTCONFIG
        );
        println!("{}", boot_config_to_boot_settings_json(&proc_bootconfig)?);
    }
    Ok(())
}

/// Parses out a valid boot config value
fn parse_value(input: &str) -> Result<BootConfigValue> {
    let input = input.trim();
    let quoted = (input.starts_with('"') && input.ends_with('"'))
        || (input.starts_with('\'') && input.ends_with('\''));
    let chars_that_require_quotes = ['\'', '"', '\n', ',', ';', '#', '}'];
    let valid_value = input
        .chars()
        .all(|c| c.is_ascii() && (quoted || !chars_that_require_quotes.contains(&c)));
    ensure!(valid_value, error::InvalidBootConfigValueSnafu { input });
    // We want the value without the quotes
    let s = if quoted {
        &input[1..input.len() - 1]
    } else {
        input
    };
    s.try_into().context(error::ParseBootConfigValueSnafu)
}

/// Takes a string and parse it into a list of valid bootconfig values
fn parse_boot_config_values(input: &str) -> Result<Vec<BootConfigValue>> {
    // Sequences of elements can mix quoted and unquoted values
    // We also don't want to separate on a quoted comma
    let mut elements = Vec::new();
    let mut quote = None;
    let mut expect_delimiter = false;
    let mut start_index = 0;
    for (i, c) in input.trim().chars().enumerate() {
        if expect_delimiter && !c.is_whitespace() && c != ',' {
            return error::ExpectedArrayCommaSnafu { input }.fail();
        }
        if c == '\'' || c == '\"' {
            if let Some(q) = quote {
                // If the quote-types match, we're expecting a delimiter next
                if q == c {
                    quote = None;
                    expect_delimiter = true;
                }
            } else {
                quote = Some(c);
            }
        } else if c == ',' && quote.is_none() {
            // We've encountered the delimiter, and if it's outside quotes, we have a new element
            elements.push(parse_value(&input[start_index..i])?);
            start_index = i + 1;
            expect_delimiter = false;
        }
    }
    ensure!(quote == None, error::UnbalancedQuotesSnafu { input });
    // Push last element
    let last_ele = if &input[start_index..] == "," {
        // If it's just a comma, assume it's an empty value at the end
        ""
    } else {
        &input[start_index..]
    };
    elements.push(parse_value(last_ele)?);
    Ok(elements)
}

/// Takes a string representation of a bootconfig file and parse it into `BootSettings`
fn parse_boot_config_to_boot_settings(bootconfig: &str) -> Result<BootSettings> {
    let mut kernel_params: HashMap<BootConfigKey, Vec<BootConfigValue>> = HashMap::new();
    let mut init_params: HashMap<BootConfigKey, Vec<BootConfigValue>> = HashMap::new();
    for line in bootconfig.trim().lines() {
        let mut kv = line.trim().split('=').map(|kv| kv.trim());
        // Ensure the key is a valid boot config key
        let key: BootConfigKey = kv
            .next()
            .ok_or(error::Error::InvalidBootConfig)?
            .try_into()
            .context(error::ParseBootConfigKeySnafu)?;
        let values = parse_boot_config_values(kv.next().ok_or(error::Error::InvalidBootConfig)?)?;

        if key != "kernel" && key.starts_with("kernel") {
            kernel_params.insert(
                key["kernel.".len()..]
                    .try_into()
                    .context(error::ParseBootConfigKeySnafu)?,
                values,
            );
        } else if key != "init" && key.starts_with("init") {
            init_params.insert(
                key["init.".len()..]
                    .try_into()
                    .context(error::ParseBootConfigKeySnafu)?,
                values,
            );
        } else if key == "kernel" || key == "init" {
            let empty_value_list: Vec<BootConfigValue> =
                vec!["".try_into().context(error::ParseBootConfigValueSnafu)?];
            // `BootSettings` does not support `kernel` or `init` as parent keys to non-null values.
            if values != empty_value_list {
                return error::ParentBootConfigKeySnafu.fail();
            }
        } else {
            return error::UnsupportedBootConfigKeySnafu { key }.fail();
        }
    }

    Ok(BootSettings {
        reboot_to_reconcile: None,
        kernel_parameters: if kernel_params.is_empty() {
            None
        } else {
            Some(kernel_params)
        },
        init_parameters: if init_params.is_empty() {
            None
        } else {
            Some(init_params)
        },
    })
}

/// Given a boot config string, deserialize it to `model::BootSettings` and then serialize it back
/// out as a JSON string for sundog consumption
fn boot_config_to_boot_settings_json(bootconfig_str: &str) -> Result<String> {
    // We'll only send the setting if the existing boot config file fits our settings model
    let boot_settings = parse_boot_config_to_boot_settings(bootconfig_str)?;
    // sundog expects JSON-serialized output
    serde_json::to_string(&boot_settings).context(error::OutputJsonSnafu)
}

#[cfg(test)]
mod boot_settings_tests {
    use crate::bootconfig::{
        boot_config_to_boot_settings_json, serialize_boot_settings_to_boot_config,
        DEFAULT_BOOTCONFIG_STR,
    };
    use maplit::hashmap;
    use model::modeled_types::{BootConfigKey, BootConfigValue};
    use model::BootSettings;
    use serde_json::json;
    use serde_json::value::Value;
    use std::collections::HashMap;
    use std::convert::TryInto;

    /// Convert a plain hash map into BootSettings parameters.
    fn to_boot_settings_params(
        params: HashMap<&str, Vec<&str>>,
    ) -> Option<HashMap<BootConfigKey, Vec<BootConfigValue>>> {
        Some(
            params
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.try_into().unwrap(),
                        v.into_iter().map(|s| s.try_into().unwrap()).collect(),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn boot_settings_to_string() {
        let boot_settings = BootSettings {
            reboot_to_reconcile: None,
            kernel_parameters: to_boot_settings_params(hashmap! {
                "console" => vec!["ttyS1,115200n8", "tty0"],
            }),
            init_parameters: to_boot_settings_params(hashmap! {
                "systemd.log_level" => vec!["debug"],
                "splash" => vec![""],
                "weird" => vec!["'single'quotes'","\"double\"quotes\""],
            }),
        };
        let output = serialize_boot_settings_to_boot_config(&boot_settings).unwrap();
        // Sort the entries alphabetically to keep results consistent
        let mut lines = output
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        lines.sort();
        let output = lines.join("\n");
        assert_eq!(
            output,
            r#"
            init.splash = ""
            init.systemd.log_level = "debug"
            init.weird = "'single'quotes'", '"double"quotes"'
            kernel.console = "ttyS1,115200n8", "tty0"
            "#
            .trim()
            .lines()
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .join("\n")
        );
    }

    #[test]
    fn none_boot_settings_to_string() {
        let boot_settings = BootSettings {
            reboot_to_reconcile: None,
            kernel_parameters: None,
            init_parameters: None,
        };
        assert_eq!(
            serialize_boot_settings_to_boot_config(&boot_settings).unwrap(),
            r#""#
        );

        let init_none_boot_settings = BootSettings {
            reboot_to_reconcile: None,
            kernel_parameters: to_boot_settings_params(hashmap! {
                "console" => vec!["ttyS1,115200n8", "tty0"],
                "usbcore.quirks" => vec!["0781:5580:bk","0a5c:5834:gij"],
            }),
            init_parameters: None,
        };
        let output = serialize_boot_settings_to_boot_config(&init_none_boot_settings).unwrap();
        // Sort the entries alphabetically to keep results consistent
        let mut lines = output
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        lines.sort();
        let output = lines.join("\n");
        assert_eq!(
            output,
            r#"
            kernel.console = "ttyS1,115200n8", "tty0"
            kernel.usbcore.quirks = "0781:5580:bk", "0a5c:5834:gij"
            "#
            .trim()
            .lines()
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .join("\n")
        );
    }

    #[test]
    fn empty_map_boot_settings_to_string() {
        let boot_settings = BootSettings {
            reboot_to_reconcile: None,
            kernel_parameters: Some(hashmap! {}),
            init_parameters: None,
        };
        assert_eq!(
            serialize_boot_settings_to_boot_config(&boot_settings).unwrap(),
            r#""#
        );
    }

    static STANDARD_BOOTCONFIG: &str = r#"
        kernel.console = "ttyS1,115200n8", "tty0"
        init.systemd.log_level = "debug"
        init.splash = ""
        "#;

    #[test]
    fn standard_boot_config_to_boot_settings_json() {
        assert_eq!(
            json!({"kernel":{"console":["ttyS1,115200n8","tty0"]},"init":{"systemd.log_level":["debug"],"splash":[""]}}),
            serde_json::from_str::<Value>(
                &boot_config_to_boot_settings_json(STANDARD_BOOTCONFIG).unwrap()
            )
            .unwrap()
        );
    }

    static SPECIAL_BOOTCONFIG: &str = r#"
        kernel = ""
        kernel.console = "ttyS1,115200n8", "tty0"
        init = ""
        init.systemd.log_level = "debug"
        "#;

    #[test]
    fn special_boot_config_to_boot_settings_json() {
        assert_eq!(
            json!({"kernel":{"console":["ttyS1,115200n8","tty0"]},"init":{"systemd.log_level":["debug"]}}),
            serde_json::from_str::<Value>(
                &boot_config_to_boot_settings_json(SPECIAL_BOOTCONFIG).unwrap()
            )
            .unwrap()
        );
    }

    static UNSUPPORTED_BOOTCONFIG: &str = r#"
        do.androids.dream.of.electric.sheep = "?"
        kernel.console = "ttyS1,115200n8", "tty0"
        init.systemd.log_level = "debug"
        "#;

    #[test]
    fn unsupported_boot_config_to_boot_settings_json() {
        assert!(&boot_config_to_boot_settings_json(UNSUPPORTED_BOOTCONFIG).is_err());
    }

    static MISSING_COMMA: &str = r#"
        kernel = "?" "???"
        "#;

    #[test]
    fn missing_comma_boot_config_to_boot_settings_json() {
        assert!(&boot_config_to_boot_settings_json(MISSING_COMMA).is_err());
    }

    static BAD_UNQUOTED_VALUE: &str = r#"
        kernel = #bang
        "#;

    #[test]
    fn bad_unquoted_value_boot_config_to_boot_settings_json() {
        assert!(&boot_config_to_boot_settings_json(BAD_UNQUOTED_VALUE).is_err());
    }

    static KERNEL_INIT_PARENT_KEY: &str = r#"
        kernel = "foo"
        init = "bar"
        "#;

    #[test]
    fn kernel_init_parent_key_boot_config_to_boot_settings_json() {
        assert!(&boot_config_to_boot_settings_json(KERNEL_INIT_PARENT_KEY).is_err());
    }

    #[test]
    fn test_default_boot_config_to_boot_settings_json() {
        assert_eq!(
            // We expect null with a bootconfig with empty keys
            serde_json::from_str::<Value>(r#"{}"#).unwrap(),
            serde_json::from_str::<Value>(
                &boot_config_to_boot_settings_json(DEFAULT_BOOTCONFIG_STR).unwrap()
            )
            .unwrap()
        );
    }
}
