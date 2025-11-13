use crate::{error, Migration, MigrationData, Result};
use regex::Regex;
use snafu::{OptionExt, ResultExt};

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
                println!("Removed {setting}, which was set to '{data}'");
            } else {
                println!("Found no {setting} to remove");
            }
        }
        Ok(input)
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
                println!("Removed {setting}, which was set to '{data}'");
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
pub struct PrefixSuffix {
    pub prefix: &'static str,
    pub suffix: &'static str,
}
pub struct AddPrefixSuffixMigration(pub Vec<PrefixSuffix>);

impl Migration for AddPrefixSuffixMigration {
    /// New versions must either have a default for the settings or generate them; we don't need to
    /// do anything.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!(
            "AddPrefixSuffixMigration({:?}) has no work to do on upgrade.",
            self.0
                .iter()
                .map(|ps| format!("{}*{}", ps.prefix, ps.suffix))
                .collect::<Vec<_>>()
        );
        Ok(input)
    }

    /// Older versions don't know about the settings; we remove them so that old versions don't see
    /// them and fail deserialization.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        let mut compiled_patterns = Vec::new();
        for pattern in &self.0 {
            let regex_pattern = format!(
                r"^{}\.(.+)\.{}$",
                regex::escape(pattern.prefix),
                regex::escape(pattern.suffix)
            );
            let regex =
                Regex::new(&regex_pattern).context(error::InvalidPrefixSuffixPatternSnafu {
                    prefix: pattern.prefix,
                    suffix: pattern.suffix,
                })?;
            compiled_patterns.push(regex);
        }

        let settings = input
            .data
            .keys()
            .filter(|k| compiled_patterns.iter().any(|regex| regex.is_match(k)))
            .cloned()
            .collect::<Vec<_>>();
        for setting in settings {
            if let Some(data) = input.data.remove(&setting) {
                println!("Removed {setting}, which was set to '{data}'");
            }
        }
        Ok(input)
    }
}

#[cfg(test)]
mod test_add_prefix_suffix_migration {
    use super::{AddPrefixSuffixMigration, PrefixSuffix};
    use crate::{Migration, MigrationData};
    use maplit::hashmap;
    use std::collections::HashMap;

    #[test]
    fn single_entry() {
        let data = MigrationData {
            data: hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "remove.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
                "remove.also.like.this".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        let result = AddPrefixSuffixMigration(vec![PrefixSuffix {
            prefix: "remove",
            suffix: "this",
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
            }
        );
    }

    #[test]
    fn compound_suffix() {
        let data = MigrationData {
            data: hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "remove.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
                "remove.not.this".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        let result = AddPrefixSuffixMigration(vec![PrefixSuffix {
            prefix: "remove",
            suffix: "like.this",
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
                "remove.not.this".into() => 0.into(),
            }
        );
    }

    #[test]
    fn multiple_entries() {
        let data = MigrationData {
            data: hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "remove.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
                "remove.also.this".into() => 0.into(),
                "delete.something.here".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        let result = AddPrefixSuffixMigration(vec![
            PrefixSuffix {
                prefix: "remove",
                suffix: "this",
            },
            PrefixSuffix {
                prefix: "delete",
                suffix: "here",
            },
        ])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
            }
        );
    }

    #[test]
    fn no_match() {
        let data = MigrationData {
            data: hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
                "other.setting.here".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        let result = AddPrefixSuffixMigration(vec![PrefixSuffix {
            prefix: "remove.",
            suffix: ".this",
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "keep.stuff.like.this".into() => 0.into(),
                "keep.this.too".into() => 0.into(),
                "other.setting.here".into() => 0.into(),
            }
        );
    }

    #[test]
    fn tight_matching() {
        let data = MigrationData {
            data: hashmap! {
                "settings.host-containers.admin.command".into() => 0.into(),
                "settings.host-containers.command".into() => 0.into(), // No middle segment
                "settings.host-containersadmincommand".into() => 0.into(), // No dots
                "keep.this".into() => 0.into(),
            },
            metadata: HashMap::new(),
        };
        let result = AddPrefixSuffixMigration(vec![PrefixSuffix {
            prefix: "settings.host-containers",
            suffix: "command",
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "settings.host-containers.command".into() => 0.into(),
                "settings.host-containersadmincommand".into() => 0.into(),
                "keep.this".into() => 0.into(),
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
                println!("Removed {setting}, which was set to '{data}'");
            } else {
                println!("Found no {setting} to remove");
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
                        self.new_val.clone_into(data);
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
                        self.old_val.clone_into(data);
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

/// We use this migration when we need to replace settings that contain lists of string values;
/// for example, when a release changes the list of configuration-files associated with a service.
// String is the only type we use today, and handling multiple value types is more complicated than
// we need at the moment.  Allowing &[serde_json::Value] seems nice, but it would allow arbitrary
// data transformations that the API model would then fail to load.
pub struct ListReplacement {
    pub setting: &'static str,
    pub old_vals: &'static [&'static str],
    pub new_vals: &'static [&'static str],
}

pub struct ReplaceListsMigration(pub Vec<ListReplacement>);

impl Migration for ReplaceListsMigration {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for replacement in &self.0 {
            if let Some(data) = input.data.get_mut(replacement.setting) {
                match data {
                    serde_json::Value::Array(data) => {
                        // We only handle string lists; convert each value to a str we can compare.
                        let list: Vec<&str> = data
                            .iter()
                            .map(|v| v.as_str())
                            .collect::<Option<Vec<&str>>>()
                            .with_context(|| error::ReplaceListContentsSnafu {
                                setting: replacement.setting,
                                data: data.clone(),
                            })?;

                        if list == replacement.old_vals {
                            // Convert back to the original type so we can store it.
                            *data = replacement.new_vals.iter().map(|s| (*s).into()).collect();
                            println!(
                                "Changed value of '{}' from {:?} to {:?} on upgrade",
                                replacement.setting, replacement.old_vals, replacement.new_vals
                            );
                        } else {
                            println!(
                                "'{}' is not set to {:?}, leaving alone",
                                replacement.setting, list
                            );
                        }
                    }
                    _ => {
                        println!(
                            "'{}' is set to non-list value '{}'; ReplaceListsMigration only handles lists",
                            replacement.setting, data
                        );
                    }
                }
            } else {
                println!("Found no '{}' to change on upgrade", replacement.setting);
            }
        }
        Ok(input)
    }

    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for replacement in &self.0 {
            if let Some(data) = input.data.get_mut(replacement.setting) {
                match data {
                    serde_json::Value::Array(data) => {
                        // We only handle string lists; convert each value to a str we can compare.
                        let list: Vec<&str> = data
                            .iter()
                            .map(|v| v.as_str())
                            .collect::<Option<Vec<&str>>>()
                            .with_context(|| error::ReplaceListContentsSnafu {
                                setting: replacement.setting,
                                data: data.clone(),
                            })?;

                        if list == replacement.new_vals {
                            // Convert back to the original type so we can store it.
                            *data = replacement.old_vals.iter().map(|s| (*s).into()).collect();
                            println!(
                                "Changed value of '{}' from {:?} to {:?} on downgrade",
                                replacement.setting, replacement.new_vals, replacement.old_vals
                            );
                        } else {
                            println!(
                                "'{}' is not set to {:?}, leaving alone",
                                replacement.setting, list
                            );
                        }
                    }
                    _ => {
                        println!(
                        "'{}' is set to non-list value '{}'; ReplaceListsMigration only handles lists",
                        replacement.setting, data
                    );
                    }
                }
            } else {
                println!("Found no '{}' to change on downgrade", replacement.setting);
            }
        }
        Ok(input)
    }
}

#[cfg(test)]
mod test_replace_list {
    use super::{ListReplacement, ReplaceListsMigration};
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
        let result = ReplaceListsMigration(vec![ListReplacement {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup"],
        }])
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
        let result = ReplaceListsMigration(vec![ListReplacement {
            setting: "hi",
            old_vals: &["sup"],
            new_vals: &["there"],
        }])
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
                "hi2".into() => vec!["hey", "listen"].into(),
                "ignored".into() => vec!["no", "change"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = ReplaceListsMigration(vec![
            ListReplacement {
                setting: "hi",
                old_vals: &["there", "you"],
                new_vals: &["sup", "hey"],
            },
            ListReplacement {
                setting: "hi2",
                old_vals: &["hey", "listen"],
                new_vals: &["look", "watch out"],
            },
        ])
        .forward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "hi".into() => vec!["sup", "hey"].into(),
                "hi2".into() => vec!["look", "watch out"].into(),
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
        let result = ReplaceListsMigration(vec![ListReplacement {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup", "hey"],
        }])
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
        let result = ReplaceListsMigration(vec![ListReplacement {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup", "hey"],
        }])
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
        ReplaceListsMigration(vec![ListReplacement {
            setting: "hi",
            old_vals: &["there"],
            new_vals: &["sup", "hey"],
        }])
        .forward(data)
        .unwrap_err();
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// When we add conditional migrations that can only run for specific variants, we need to run this
/// migration helper for cases where the migration does NOT apply so migrator will still create a valid
/// intermediary datastore that the host can transition to.
#[derive(Debug)]
pub struct NoOpMigration;

impl Migration for NoOpMigration {
    /// No work to do on forward migrations, copy the same datastore
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("NoOpMigration has no work to do on upgrade.",);
        Ok(input)
    }

    /// No work to do on backward migrations, copy the same datastore
    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("NoOpMigration has no work to do on downgrade.",);
        Ok(input)
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration when we add new values to a list setting that older versions don't
/// understand. On downgrade, we filter the list to only include values the old version accepts.
pub struct ListRestriction {
    pub setting: &'static str,
    pub allowed_vals: &'static [&'static str],
}

pub struct RestrictListsMigration(pub Vec<ListRestriction>);

impl Migration for RestrictListsMigration {
    /// New versions can handle all values; no work needed on upgrade.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!(
            "RestrictListsMigration({:?}) has no work to do on upgrade.",
            self.0.iter().map(|r| r.setting).collect::<Vec<_>>()
        );
        Ok(input)
    }

    /// Older versions only understand certain values; remove any values not in the allowed list.
    fn backward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        for restriction in &self.0 {
            if let Some(data) = input.data.get_mut(restriction.setting) {
                match data {
                    serde_json::Value::Array(data) => {
                        let list: Vec<&str> = data
                            .iter()
                            .map(|v| v.as_str())
                            .collect::<Option<Vec<&str>>>()
                            .with_context(|| error::ReplaceListContentsSnafu {
                                setting: restriction.setting,
                                data: data.clone(),
                            })?;

                        let filtered: Vec<&str> = list
                            .into_iter()
                            .filter(|val| restriction.allowed_vals.contains(val))
                            .collect();

                        if filtered.len() != data.len() {
                            let new_data: Vec<serde_json::Value> =
                                filtered.iter().map(|s| (*s).into()).collect();
                            println!(
                                "Filtered '{}' to allowed values {:?} on downgrade",
                                restriction.setting, filtered
                            );
                            *data = new_data;
                        } else {
                            println!(
                                "'{}' already contains only allowed values, leaving alone",
                                restriction.setting
                            );
                        }
                    }
                    _ => {
                        println!(
                            "'{}' is set to non-list value '{}'; RestrictListsMigration only handles lists",
                            restriction.setting, data
                        );
                    }
                }
            } else {
                println!("Found no '{}' to filter on downgrade", restriction.setting);
            }
        }
        Ok(input)
    }
}

#[cfg(test)]
mod test_restrict_lists {
    use super::{ListRestriction, RestrictListsMigration};
    use crate::{Migration, MigrationData};
    use maplit::hashmap;
    use std::collections::HashMap;

    #[test]
    fn filter_values() {
        let data = MigrationData {
            data: hashmap! {
                "setting".into() => vec!["old1", "old2", "new1", "new2"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = RestrictListsMigration(vec![ListRestriction {
            setting: "setting",
            allowed_vals: &["old1", "old2"],
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "setting".into() => vec!["old1", "old2"].into(),
            }
        );
    }

    #[test]
    fn no_filtering_needed() {
        let data = MigrationData {
            data: hashmap! {
                "setting".into() => vec!["old1", "old2"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = RestrictListsMigration(vec![ListRestriction {
            setting: "setting",
            allowed_vals: &["old1", "old2", "new1"],
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "setting".into() => vec!["old1", "old2"].into(),
            }
        );
    }

    #[test]
    fn forward_no_change() {
        let data = MigrationData {
            data: hashmap! {
                "setting".into() => vec!["old1", "new1", "new2"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = RestrictListsMigration(vec![ListRestriction {
            setting: "setting",
            allowed_vals: &["old1"],
        }])
        .forward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "setting".into() => vec!["old1", "new1", "new2"].into(),
            }
        );
    }

    #[test]
    fn multiple_restrictions() {
        let data = MigrationData {
            data: hashmap! {
                "setting1".into() => vec!["a", "b", "c"].into(),
                "setting2".into() => vec!["x", "y", "z"].into(),
            },
            metadata: HashMap::new(),
        };
        let result = RestrictListsMigration(vec![
            ListRestriction {
                setting: "setting1",
                allowed_vals: &["a", "b"],
            },
            ListRestriction {
                setting: "setting2",
                allowed_vals: &["x"],
            },
        ])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "setting1".into() => vec!["a", "b"].into(),
                "setting2".into() => vec!["x"].into(),
            }
        );
    }

    #[test]
    fn not_list() {
        let data = MigrationData {
            data: hashmap! {
                "setting".into() => "not a list".into(),
            },
            metadata: HashMap::new(),
        };
        let result = RestrictListsMigration(vec![ListRestriction {
            setting: "setting",
            allowed_vals: &["old1"],
        }])
        .backward(data)
        .unwrap();
        assert_eq!(
            result.data,
            hashmap! {
                "setting".into() => "not a list".into(),
            }
        );
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// We use this migration to remove a setting string if it matches the old value.
/// We will need this migration once to adapt the concept of Strength on settings.
pub struct RemoveMatchingString {
    pub setting: &'static str,
    pub old_val: &'static str,
}

impl Migration for RemoveMatchingString {
    fn forward(&mut self, mut input: MigrationData) -> Result<MigrationData> {
        if let Some(data) = input.data.get_mut(self.setting) {
            match data {
                serde_json::Value::String(data) => {
                    if data == self.old_val {
                        input.data.remove(self.setting);
                    } else {
                        println!(
                            "'{}' is not set to '{}', leaving alone",
                            self.setting, self.old_val
                        );
                    }
                }
                _ => {
                    println!(
                        "'{}' is set to non-string value '{}'; RemoveOldData expects a string setting value",
                        self.setting, data
                    );
                }
            }
        } else {
            println!("Found no '{}' to change on upgrade", self.setting);
        }
        Ok(input)
    }

    fn backward(&mut self, input: MigrationData) -> Result<MigrationData> {
        println!("RemoveOldData has no work to do on downgrade.",);
        Ok(input)
    }
}
