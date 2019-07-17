use handlebars::template;
use handlebars::template::{Parameter, TemplateElement};
use handlebars::Handlebars;
use snafu::ResultExt;
use std::collections::HashSet;

use crate::{error, helpers, Result};

use apiserver::model;

/// Build the template registry using the ConfigFile structs
/// and let handlebars parse the templates
pub fn build_template_registry(
    files: &model::ConfigurationFiles,
) -> Result<handlebars::Handlebars> {
    let mut template_registry = Handlebars::new();
    // Strict mode will panic if a key exists in the template
    // but isn't provided in the data given to the renderer
    template_registry.set_strict_mode(true);

    debug!("Building template registry of configuration files");
    for (name, metadata) in files {
        debug!(
            "Registering {} at path '{}'",
            &name, &metadata.template_path
        );
        template_registry
            .register_template_file(&name, &metadata.template_path)
            .context(error::TemplateRegister {
                name: name.as_str(),
                path: &metadata.template_path,
            })?;
    }

    // TODO if we start writing lots of helpers, registering them
    // should probably exist in a "setup" function of its own
    // that we can call from here. For now, KISS.
    template_registry.register_helper("base64_decode", Box::new(helpers::base64_decode));

    Ok(template_registry)
}

/// This trait allows us to get a list of template keys (Expressions in handlebars
/// parlance) out of a template
pub trait TemplateKeys {
    /// Return a HashSet of template keys from a template
    fn get_all_template_keys(&self) -> Result<HashSet<String>>;
}

/// Extends the template::Template type from the Handlebars library to extract
/// all keys from a single template
impl TemplateKeys for template::Template {
    /// Retrieve all keys from a single template
    fn get_all_template_keys(&self) -> Result<HashSet<String>> {
        let mut keys: HashSet<String> = HashSet::new();

        for element in &self.elements {
            // Currently we only match on Expressions and HelperBlocks (conditionals)
            // and ignore everything else. Our templates are simple so far and this
            // match should capture all the template keys.
            match element {
                TemplateElement::Expression(helper_template) => {
                    // Blocks with helpers have the same data structure as
                    // those that don't.  However, blocks with helpers have a
                    // non-empty params vec. Assume this vec contains the keys.
                    if !helper_template.params.is_empty() {
                        for param in &helper_template.params {
                            if let Parameter::Name(key) = param {
                                trace!("Found key: {}", &key);
                                keys.insert(key.to_string());
                            }
                        }
                    } else if let Parameter::Name(key) = &helper_template.name {
                        trace!("Found key: {}", &key);
                        keys.insert(key.to_string());
                    }
                }

                TemplateElement::HelperBlock(block) => {
                    if let Some(ref tmpl) = block.template {
                        for key in tmpl.get_all_template_keys()?.into_iter() {
                            trace!("Found key: {}", &key);
                            keys.insert(key);
                        }
                    }

                    // Params are keys inside conditional expressions.
                    for param in &block.params {
                        if let Parameter::Name(key) = param {
                            trace!("Found key in a conditional: {}", &key);
                            keys.insert(key.to_string());
                        }
                    }
                }

                // Not an expression
                _ => {}
            }
        }
        Ok(keys)
    }
}

/// Extends the Handlebars type (the template Registry) from the Handlebars library
/// to extract all keys from all templates currently registered
impl TemplateKeys for Handlebars {
    /// Retrieve all keys from all templates in the registry
    fn get_all_template_keys(&self) -> Result<HashSet<String>> {
        debug!("Querying registry for templates");
        let templates = self.get_templates();
        trace!("Templates in registry: {:?}", &templates);

        // For each of the templates in the repository, get all the
        // keys and add them to the HashSet to be returned
        let mut keys = HashSet::new();
        for (template_name, template) in templates {
            debug!("Parsing template: {}", &template_name);
            for key in template.get_all_template_keys()?.into_iter() {
                keys.insert(key);
            }
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashset;

    fn assert_keys_in_template(template: &str, expected_keys: HashSet<String>) {
        let mut registry = Handlebars::new();
        registry.register_template_string("x", template).unwrap();

        // Get the template from the registry, then get the template's keys
        let template = registry.get_template("x").unwrap();
        let actual_keys = template.get_all_template_keys().unwrap();
        assert_eq!(actual_keys, expected_keys)
    }

    fn assert_keys_in_registry(templates: &[&str], expected_keys: HashSet<String>) {
        let mut registry = Handlebars::new();
        // Don't care about template name, just use an integer
        for (i, template) in templates.iter().enumerate() {
            registry
                .register_template_string(&i.to_string(), template)
                .unwrap();
        }

        // Get the keys from the registry directly
        let actual_keys = registry.get_all_template_keys().unwrap();
        assert_eq!(actual_keys, expected_keys)
    }

    #[test]
    // Ensure that we get all the keys out of a single template
    fn get_template_keys_from_single_template() {
        assert_keys_in_template(
            "This is a cool {{template}}. Here is a conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}",
            hashset! {"template".to_string(), "bridge-ip".to_string() },
        );
    }

    #[test]
    // Ensure that we get all the keys out of a the entire registry
    fn get_template_keys_from_registry() {
        let tmpl1 = "This is a cool {{template}}. Here is a conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}";
        let tmpl2 = "This is a cool {{frob}}. Here is a conditional: {{#if frobnicate }}{{frobnicate}}{{/if}}";

        assert_keys_in_registry(
            &[tmpl1, tmpl2],
            hashset! {"template".to_string(), "bridge-ip".to_string(), "frob".to_string(), "frobnicate".to_string() },
        );
    }

    #[test]
    // This template has a different key in the conditional expression, ensure we catch that
    fn get_keys_from_conditional() {
        let tmpl3 =
            "This is a cool {{frob}}. Here is a conditional: {{#if thar }}{{frobnicate}}{{/if}}";

        assert_keys_in_registry(
            &[tmpl3],
            hashset! {"frob".to_string(), "thar".to_string(), "frobnicate".to_string() },
        );
    }

    #[test]
    fn get_keys_with_boolean_in_conditional() {
        assert_keys_in_registry(
            &["This is a cool {{template}}. Here is a conditional: {{#if true }}{{bridge-ip}}{{/if}}"],
            hashset! {"template".to_string(), "bridge-ip".to_string() },
        );
    }

    #[test]
    fn get_keys_with_nested_conditional() {
        assert_keys_in_registry(
            &["This is a cool {{template}}. Here is a conditional: {{#if true }}{{bridge-ip}}{{#if thar}}{{baz}}{{/if}}{{/if}}"],
            hashset! {"template".to_string(), "bridge-ip".to_string(), "thar".to_string(), "baz".to_string() },
        );
    }

    #[test]
    fn empty_template_returns_empty_hashset() {
        assert_keys_in_registry(&[""], hashset! {});
    }

    #[test]
    fn template_with_helper_returns_correct_keys() {
        let mut registry = Handlebars::new();
        registry.register_helper("base64_decode", Box::new(helpers::base64_decode));

        // Register a template with the base64 helper
        let tmpl1 = "This is a cool {{base64_decode template}}. Here is a conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}";
        let tmpl2 = "This is a cool {{frob}}. Here is a conditional: {{#if frobnicate }}{{frobnicate}}{{/if}}";
        registry.register_template_string("tmpl1", tmpl1).unwrap();
        registry.register_template_string("tmpl2", tmpl2).unwrap();

        let expected_keys = hashset! {"template".to_string(), "bridge-ip".to_string(), "frob".to_string(), "frobnicate".to_string() };
        let actual_keys = registry.get_all_template_keys().unwrap();

        assert_eq!(actual_keys, expected_keys)
    }
}
