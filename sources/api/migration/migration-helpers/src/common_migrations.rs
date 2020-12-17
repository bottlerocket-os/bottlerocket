use crate::{error, Metadata, Migration, MigrationData, Result};
use serde::Serialize;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;

/// We use this migration when we add settings and want to make sure they're removed before we go
/// back to old versions that don't understand them.
pub struct AddSettingsMigration<'a>(pub &'a [&'static str]);

impl Migration for AddSettingsMigration<'_> {
    /// New versions must either have a default for the settings or generate them; we don't need to
    /// do anything.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!(
            "AddSettingsMigration({:?}) has no work to do on upgrade.",
            self.0
        );
        Ok(input)
    }

    /// Older versions don't know about the settings; we remove them so that old versions don't see
    /// them and fail deserialization.  (The settings must be defaulted or generated in new versions,
    /// and safe to remove.)
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for setting in self.0 {
            if let Some(data) = input.data.remove(*setting) {
                println!("Removed {}, which was set to '{}'", setting, data);
            } else {
                println!("Found no {} to remove", setting);
            }
        }
        Ok(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Similar to the above, this migration is for when we add a single setting.
/// We are retaining this migration helper in case there are migrations already using it.
#[deprecated(note = "Please use `AddSettingsMigration` instead")]
pub struct AddSettingMigration(pub &'static str);

#[allow(deprecated)]
impl Migration for AddSettingMigration {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        AddSettingsMigration(&[self.0]).forward(input)
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        AddSettingsMigration(&[self.0]).backward(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we add a cluster of settings under known prefixes and want to make
/// sure they're removed before we go back to old versions that don't understand them.  Normally
/// you'd use AddSettingsMigration since you know the key names, but this is useful for
/// user-defined keys, for example in a map like settings.kernel.sysctl or
/// settings.host-containers.
pub struct AddPrefixesMigration(pub Vec<&'static str>);

impl Migration for AddPrefixesMigration {
    /// New versions must either have a default for the settings or generate them; we don't need to
    /// do anything.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!(
            "AddPrefixesMigration({:?}) has no work to do on upgrade.",
            self.0
        );
        Ok(input)
    }

    /// Older versions don't know about the settings; we remove them so that old versions don't see
    /// them and fail deserialization.  (The settings must be defaulted or generated in new versions,
    /// and safe to remove.)
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let settings = input
            .data
            .keys()
            .filter(|k| self.0.iter().any(|prefix| k.starts_with(prefix)))
            .cloned()
            .collect::<Vec<_>>();
        for setting in settings {
            if let Some(data) = input.data.remove(&setting) {
                println!("Removed {}, which was set to '{}'", setting, data);
            }
        }
        Ok(input)
    }
}

#[cfg(test)]
mod test_add_prefixes_migration {
    use super::AddPrefixesMigration;
    use crate::{Migration, MigrationData};
    use maplit::hashmap;
    use std::collections::HashMap;

    #[test]
    fn single() {
        let data = MigrationData {
            data: hashmap! {
                "keep.me.a".into() => 0.into(),
                "remove.me.b".into() => 0.into(),
                "keep.this.c".into() => 0.into(),
                "remove.me.d.e".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        // Run backward, e.g. downgrade, to test that the right keys are removed
        let result = AddPrefixesMigration(vec!["remove.me"])
            .backward(data)
            .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.me.a".into() => 0.into(),
                "keep.this.c".into() => 0.into(),
            }
        );
    }

    #[test]
    fn multiple() {
        let data = MigrationData {
            data: hashmap! {
                "keep.me.a".into() => 0.into(),
                "remove.me.b".into() => 0.into(),
                "keep.this.c".into() => 0.into(),
                "remove.this.d.e".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        // Run backward, e.g. downgrade, to test that the right keys are removed
        let result = AddPrefixesMigration(vec!["remove.me", "remove.this"])
            .backward(data)
            .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.me.a".into() => 0.into(),
                "keep.this.c".into() => 0.into(),
            }
        );
    }

    #[test]
    fn no_match() {
        let data = MigrationData {
            data: hashmap! {
                "keep.me.a".into() => 0.into(),
                "remove.me.b".into() => 0.into(),
                "keep.this.c".into() => 0.into(),
                "remove.this.d.e".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        // Run backward, e.g. downgrade, to test that the right keys are removed
        let result = AddPrefixesMigration(vec!["not.found", "nor.this"])
            .backward(data)
            .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.me.a".into() => 0.into(),
                "remove.me.b".into() => 0.into(),
                "keep.this.c".into() => 0.into(),
                "remove.this.d.e".into() => 0.into(),
            }
        );
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we remove settings from the model, so the new version doesn't see
/// them and error.
pub struct RemoveSettingsMigration<'a>(pub &'a [&'static str]);

impl Migration for RemoveSettingsMigration<'_> {
    /// Newer versions don't know about the settings; we remove them so that new versions don't see
    /// them and fail deserialization.  (The settings must be defaulted or generated in old versions,
    /// and safe to remove.)
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for setting in self.0 {
            if let Some(data) = input.data.remove(*setting) {
                println!("Removed {}, which was set to '{}'", setting, data);
            } else {
                println!("Found no {} to remove", setting);
            }
        }
        Ok(input)
    }

    /// Old versions must either have a default for the settings or generate it; we don't need to
    /// do anything.
    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!(
            "RemoveSettingsMigration({:?}) has no work to do on downgrade.",
            self.0
        );
        Ok(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Similar to the above, this migration is for when we need to remove a single setting.
/// We are retaining this migration helper in case there are migrations already using it.
#[deprecated(note = "Please use `RemoveSettingsMigration` instead")]
pub struct RemoveSettingMigration(pub &'static str);

#[allow(deprecated)]
impl Migration for RemoveSettingMigration {
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        RemoveSettingsMigration(&[self.0]).forward(input)
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        RemoveSettingsMigration(&[self.0]).backward(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we replace a setting's old string value with a new string value.
pub struct ReplaceStringMigration {
    pub setting: &'static str,
    pub old_val: &'static str,
    pub new_val: &'static str,
}

impl Migration for ReplaceStringMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(self.setting) {
            match data {
                serde_json::Value::String(data) => {
                    if data == self.old_val {
                        *data = self.new_val.to_owned();
                        println!(
                            "Changed value of '{}' from '{}' to '{}' on upgrade",
                            self.setting, self.old_val, self.new_val
                        );
                    } else {
                        println!(
                            "'{}' is not set to '{}', leaving alone",
                            self.setting, self.old_val
                        );
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'; ReplaceStringMigration only handles strings",
                        self.setting, data
                    );
                }
            }
        } else {
            println!("Found no '{}' to change on upgrade", self.setting);
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(self.setting) {
            match data {
                serde_json::Value::String(data) => {
                    if data == self.new_val {
                        *data = self.old_val.to_owned();
                        println!(
                            "Changed value of '{}' from '{}' to '{}' on downgrade",
                            self.setting, self.new_val, self.old_val
                        );
                    } else {
                        println!(
                            "'{}' is not set to '{}', leaving alone",
                            self.setting, self.new_val
                        );
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'; ReplaceStringMigration only handles strings",
                        self.setting, data
                    );
                }
            }
        } else {
            println!("Found no '{}' to change on downgrade", self.setting);
        }
        Ok(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we replace a setting that contains a list of string values.
// String is the only type we use today, and handling multiple value types is more complicated than
// we need at the moment.  Allowing &[serde_json::Value] seems nice, but it would allow arbitrary
// data transformations that the API model would then fail to load.
pub struct ReplaceListMigration {
    pub setting: &'static str,
    pub old_vals: &'static [&'static str],
    pub new_vals: &'static [&'static str],
}

impl Migration for ReplaceListMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(self.setting) {
            match data {
                serde_json::Value::Array(data) => {
                    // We only handle string lists; convert each value to a str we can compare.
                    let list: Vec<&str> = data
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Option<Vec<&str>>>()
                        .with_context(|| error::ReplaceListContents {
                            setting: self.setting,
                            data: data.clone(),
                        })?;

                    if list == self.old_vals {
                        // Convert back to the original type so we can store it.
                        *data = self.new_vals.iter().map(|s| (*s).into()).collect();
                        println!(
                            "Changed value of '{}' from {:?} to {:?} on upgrade",
                            self.setting, self.old_vals, self.new_vals
                        );
                    } else {
                        println!("'{}' is not set to {:?}, leaving alone", self.setting, list);
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-list value '{}'; ReplaceListMigration only handles lists",
                        self.setting, data
                    );
                }
            }
        } else {
            println!("Found no '{}' to change on upgrade", self.setting);
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(self.setting) {
            match data {
                serde_json::Value::Array(data) => {
                    // We only handle string lists; convert each value to a str we can compare.
                    let list: Vec<&str> = data
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Option<Vec<&str>>>()
                        .with_context(|| error::ReplaceListContents {
                            setting: self.setting,
                            data: data.clone(),
                        })?;

                    if list == self.new_vals {
                        // Convert back to the original type so we can store it.
                        *data = self.old_vals.iter().map(|s| (*s).into()).collect();
                        println!(
                            "Changed value of '{}' from {:?} to {:?} on downgrade",
                            self.setting, self.new_vals, self.old_vals
                        );
                    } else {
                        println!("'{}' is not set to {:?}, leaving alone", self.setting, list);
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-list value '{}'; ReplaceListMigration only handles lists",
                        self.setting, data
                    );
                }
            }
        } else {
            println!("Found no '{}' to change on downgrade", self.setting);
        }
        Ok(input)
    }
}

#[cfg(test)]
mod test_replace_list {
    use super::ReplaceListMigration;
    use crate::{Migration, MigrationData};
    use maplit::hashmap;
    use std::collections::HashMap;

    #[test]
    fn single() {
        let data = MigrationData {
            data: hashmap! {
                "hi".into() => vec!["there"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = ReplaceListMigration {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup"],
        }
        .forward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "hi".into() => vec!["sup"].into(),
            }
        );
    }

    #[test]
    fn backward() {
        let data = MigrationData {
            data: hashmap! {
                "hi".into() => vec!["there"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = ReplaceListMigration {
            setting: "hi",
            old_vals: &["sup"],
            new_vals: &["there"],
        }
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "hi".into() => vec!["sup"].into(),
            }
        );
    }

    #[test]
    fn multiple() {
        let data = MigrationData {
            data: hashmap! {
                "hi".into() => vec!["there", "you"].into(),
                "ignored".into() => vec!["no", "change"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = ReplaceListMigration {
            setting: "hi",
            old_vals: &["there", "you"],
            new_vals: &["sup", "hey"],
        }
        .forward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "hi".into() => vec!["sup", "hey"].into(),
                "ignored".into() => vec!["no", "change"].into(),
            }
        );
    }

    #[test]
    fn no_match() {
        let data = MigrationData {
            data: hashmap! {
                "hi".into() => vec!["no", "change"].into(),
                "hi2".into() => vec!["no", "change"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = ReplaceListMigration {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup", "hey"],
        }
        .forward(data)
        .unwrap();
        // No change
        assert_eq!(
            result.data,
            hashmap! {
                "hi".into() => vec!["no", "change"].into(),
                "hi2".into() => vec!["no", "change"].into(),
            }
        );
    }

    #[test]
    fn not_list() {
        let data = MigrationData {
            data: hashmap! {
                "hi".into() => "just a string, not a list".into(),
            },
            metadata: HashMap::new(),
        };
        let result = ReplaceListMigration {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup", "hey"],
        }
        .forward(data)
        .unwrap();
        // No change
        assert_eq!(
            result.data,
            hashmap! {
                "hi".into() => "just a string, not a list".into(),
            }
        );
    }

    #[test]
    fn not_string() {
        let data = MigrationData {
            data: hashmap! {
                "hi".into() => vec![0].into(),
            },
            metadata: HashMap::new(),
        };
        ReplaceListMigration {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup", "hey"],
        }
        .forward(data)
        .unwrap_err();
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we replace an existing template for generating some setting.
pub struct ReplaceTemplateMigration {
    pub setting: &'static str,
    pub old_template: &'static str,
    pub new_template: &'static str,
}

impl ReplaceTemplateMigration {
    /// Helper to retrieve a setting's template
    fn get_setting_template(&self, input: &MigrationData) -> Option<String> {
        if let Some(metadata) = input.metadata.get(self.setting) {
            if let Some(template) = metadata.get("template") {
                if let Some(template) = template.as_str() {
                    return Some(template.to_owned());
                } else {
                    eprintln!(
                        "'{}' has non-string template value '{}'",
                        self.setting, template
                    )
                }
            } else {
                eprintln!("'{}' has no 'template' key in metadata", self.setting);
            }
        } else {
            eprintln!("'{}' has no metadata", self.setting);
        }
        None
    }

    /// This helper function takes `MigrationData.data`, which is a mapping of dotted keys ->
    /// scalar values, and converts it into the hierarchical representation needed by handlebars templates.
    fn structure_migration_data(
        &self,
        input: &HashMap<String, serde_json::Value>,
    ) -> Result<impl Serialize> {
        let mut datastore: HashMap<datastore::Key, String> = HashMap::new();
        for (k, v) in input.iter() {
            // The prefixes we want to make available; these each have to be deserialized below.
            if k.starts_with("settings.") || k.starts_with("os.") {
                datastore.insert(
                    datastore::Key::new(datastore::KeyType::Data, k).context(error::NewKey)?,
                    // We want the serialized form here, to work with the datastore deserialization code.
                    // to_string on a Value gives the serialized form.
                    v.to_string(),
                );
            }
        }
        // Note this is a workaround because we don't have a top level model structure that encompasses 'settings'.
        // We need to use `from_map_with_prefix` because we don't have a struct; it strips away the
        // "settings" layer, which we then add back on with a wrapping HashMap.
        let settings_data: HashMap<String, serde_json::Value> =
            datastore::deserialization::from_map_with_prefix(
                Some("settings".to_string()),
                &datastore,
            )
            .context(error::DeserializeDatastore)?;
        // Same for "os.*"
        let os_data: HashMap<String, serde_json::Value> =
            datastore::deserialization::from_map_with_prefix(Some("os".to_string()), &datastore)
                .context(error::DeserializeDatastore)?;

        let mut structured_data = HashMap::new();
        structured_data.insert("settings", settings_data);
        structured_data.insert("os", os_data);

        Ok(structured_data)
    }

    /// This handles the common behavior of the forward and backward migrations.
    /// We get the setting's template and generate the old value to be sure the user hasn't changed
    /// it, then generate the new value for our output.
    fn update_template_and_data(
        &self,
        outgoing_setting_data: &str,
        outgoing_template: &str,
        incoming_template: &str,
        input: &mut MigrationData,
    ) -> Result<()> {
        if let Some(template) = &self.get_setting_template(&input) {
            if template == outgoing_template {
                println!(
                    "Changing template of '{}' from '{}' to '{}'",
                    self.setting, outgoing_template, incoming_template
                );
                // Update the setting's template
                let metadata = input
                    .metadata
                    .entry(self.setting.to_string())
                    .or_insert(Metadata::new());
                metadata.insert(
                    "template".to_string(),
                    serde_json::Value::String(incoming_template.to_string()),
                );
                let registry =
                    schnauzer::build_template_registry().context(error::BuildTemplateRegistry)?;
                // Structure the input migration data into its hierarchical representation needed by render_template
                let input_data = self.structure_migration_data(&input.data)?;
                // Generate settings data using the setting's outgoing template  so we can confirm
                // it matches our expected value; if not, the user has changed it and we should stop.
                let generated_old_data = registry
                    .render_template(template, &input_data)
                    .context(error::RenderTemplate { template })?;
                if generated_old_data == *outgoing_setting_data {
                    // Generate settings data using the setting's incoming template
                    let generated_new_data = registry
                        .render_template(incoming_template, &input_data)
                        .context(error::RenderTemplate { template })?;
                    println!(
                        "Changing value of '{}' from '{}' to '{}'",
                        self.setting, outgoing_setting_data, generated_new_data
                    );
                    // Update settings value with new generated value
                    input.data.insert(
                        self.setting.to_string(),
                        serde_json::Value::String(generated_new_data),
                    );
                } else {
                    println!(
                        "'{}' is not set to '{}', leaving alone",
                        self.setting, generated_old_data
                    );
                }
            } else {
                println!(
                    "Template for '{}' is not set to '{}', leaving alone",
                    self.setting, outgoing_template
                );
            }
        }

        Ok(())
    }
}

impl Migration for ReplaceTemplateMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(input_value) = input.data.get(self.setting) {
            let data = input_value
                .as_str()
                .context(error::NonStringSettingDataType {
                    setting: self.setting,
                })?;
            println!(
                "Updating template and value of '{}' on upgrade",
                self.setting
            );
            self.update_template_and_data(
                // Clone the input string; we need to give the function mutable access to
                // the structure that contains the string, so we can't pass a reference into the structure.
                &data.to_owned(),
                self.old_template,
                self.new_template,
                &mut input,
            )?;
        } else {
            println!("Found no '{}' to change on upgrade", self.setting);
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(input_value) = input.data.get(self.setting) {
            let data = input_value
                .as_str()
                .context(error::NonStringSettingDataType {
                    setting: self.setting,
                })?;
            println!(
                "Updating template and value of '{}' on downgrade",
                self.setting
            );
            self.update_template_and_data(
                // Clone the input string; we need to give the function mutable access to
                // the structure that contains the string, so we can't pass a reference into the structure.
                &data.to_owned(),
                self.new_template,
                self.old_template,
                &mut input,
            )?;
        } else {
            println!("Found no '{}' to change on downgrade", self.setting);
        }
        Ok(input)
    }
}
