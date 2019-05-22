use std::collections::HashSet;

use crate::Result;

use handlebars::template;
use handlebars::Handlebars;

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
        template_registry.register_template_file(&name, &metadata.template_path)?;
    }

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
                handlebars::template::TemplateElement::Expression(name) => {
                    if let handlebars::template::Parameter::Name(key) = name {
                        trace!("Found key: {}", &key);
                        keys.insert(key.to_string());
                    }
                }

                handlebars::template::TemplateElement::HelperBlock(block) => {
                    if let Some(ref tmpl) = block.template {
                        for key in tmpl.get_all_template_keys()?.into_iter() {
                            trace!("Found key: {}", &key);
                            keys.insert(key);
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

    #[test]
    // Ensure that we get all the keys out of a single template
    fn get_template_keys_from_single_template() {
        let template_name = "test_tmpl";
        let template_string = "This is a cool {{template}}. Here is a conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}";
        let expected_keys = hashset! {"template".to_string(), "bridge-ip".to_string() };

        // Register the template so the registry creates a Template object
        let mut registry = Handlebars::new();
        registry
            .register_template_string(template_name, template_string)
            .unwrap();

        // Get the template from the registry
        let template = registry.get_template(template_name).unwrap();

        assert!(template.get_all_template_keys().is_ok());
        assert_eq!(template.get_all_template_keys().unwrap(), expected_keys)
    }

    #[test]
    // Ensure that we get all the keys out of a the entire registry
    fn get_template_keys_from_registry() {
        let name1 = "test_tmpl1";
        let tmpl1 = "This is a cool {{template}}. Here is a conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}";

        let name2 = "test_tmpl2";
        let tmpl2 = "This is a cool {{frob}}. Here is a conditional: {{#if frobnicate }}{{frobnicate}}{{/if}}";

        let expected_keys = hashset! {"template".to_string(), "bridge-ip".to_string(), "frob".to_string(), "frobnicate".to_string() };

        // Register the templates so the registry creates Template objects
        let mut registry = Handlebars::new();
        registry.register_template_string(name1, tmpl1).unwrap();
        registry.register_template_string(name2, tmpl2).unwrap();

        assert!(registry.get_all_template_keys().is_ok());
        assert_eq!(registry.get_all_template_keys().unwrap(), expected_keys)
    }
}
