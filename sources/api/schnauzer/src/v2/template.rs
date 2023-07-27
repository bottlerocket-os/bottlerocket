//! This module contains functionality for parsing Bottlerocket configuration templates.
//!
//! We use `pest` to disambiguate the TOML frontmatter from the body of the template, then serde to
//! extract the contents of the frontmatter.
use pest::Parser;
use pest_derive::Parser;
use serde::Deserialize;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Parser, Debug, Clone)]
#[grammar = "v2/grammars/template.pest"]
#[grammar = "v2/grammars/toml.pest"]
pub struct TemplateParser;

/// A Bottlerocket configuration template.
///
/// Templates have:
/// * A frontmatter section containing metadata on requirements to render the template.
/// * A body containing the handlebars template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// The template frontmatter.
    ///
    /// Contains settings extensions and handlebars helpers required to render the template.
    pub frontmatter: TemplateFrontmatter,
    /// The template body, using the `handlebars` template language.
    pub body: String,
}

// Type aliases to clarify the intent of string data.
type ExtensionName = String;
type ExtensionVersion = String;
type HelperName = String;

/// Frontmatter defines the settings extensions and helpers needed to render a template.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct TemplateFrontmatter {
    #[serde(rename = "required-extensions")]
    required_extensions: Option<HashMap<ExtensionName, TemplateExtensionRequirements>>,
}

/// Template extension requirements can be specified in two ways, similar to Cargo.toml:
///   * extension = "version"
///   * extension = { version = "version", helpers = ["helper1", "helper2"] }
///
/// The first form is simpler but cannot express a dependency on any helpers.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
enum TemplateExtensionRequirements {
    Version(ExtensionVersion),
    VersionAndHelpers(DetailedTemplateExtensionRequirements),
}

/// Serialized structure of settings and handlebars helper requirements.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
struct DetailedTemplateExtensionRequirements {
    version: ExtensionVersion,
    helpers: Option<Vec<HelperName>>,
}

impl TemplateFrontmatter {
    /// Returns the name, version of each setting and the associated helpers required by this template.
    pub fn extension_requirements(&self) -> impl Iterator<Item = ExtensionRequirement> + '_ {
        self.required_extensions
            .as_ref()
            .map(|requirements| Box::new(requirements.iter()) as Box<dyn Iterator<Item = _>>)
            .unwrap_or_else(|| Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>)
            .map(|(extension_name, extension_requirements)| {
                ExtensionRequirement::from_template_requirements(
                    extension_name,
                    extension_requirements,
                )
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct ExtensionRequirement {
    pub name: ExtensionName,
    pub version: ExtensionVersion,
    pub helpers: Vec<HelperName>,
}

impl ExtensionRequirement {
    fn from_template_requirements(
        extension_name: &ExtensionName,
        requirements: &TemplateExtensionRequirements,
    ) -> Self {
        match requirements {
            TemplateExtensionRequirements::Version(version) => ExtensionRequirement {
                name: extension_name.clone(),
                version: version.clone(),
                helpers: vec![],
            },
            TemplateExtensionRequirements::VersionAndHelpers(extension_requirements) => {
                ExtensionRequirement {
                    name: extension_name.clone(),
                    version: extension_requirements.version.clone(),
                    helpers: extension_requirements.helpers.clone().unwrap_or_default(),
                }
            }
        }
    }
}

impl FromStr for Template {
    type Err = error::Error;

    fn from_str(input_str: &str) -> Result<Template> {
        // Potential improvement here would be to use `ariadne` crate to emit more helpful error messages
        let mut parsed_pairs = TemplateParser::parse(Rule::template, input_str)
            .context(error::GrammarParseSnafu)?
            .flatten();

        // If the template succesfully parses, we know there's always exactly one TOML document and one template body.
        let frontmatter_pair = parsed_pairs
            .clone()
            .find(|p| p.as_rule() == Rule::toml_document)
            .context(error::TemplateParserLogicSnafu {
                message: "Template parser did not find frontmatter document.",
            })?;

        let frontmatter_document: TemplateFrontmatter =
            toml::from_str(frontmatter_pair.as_str()).context(error::FrontmatterParseSnafu)?;

        let template_body = parsed_pairs
            .find(|p| p.as_rule() == Rule::body)
            // The pest grammar ensures that there is exactly one template body.
            .context(error::TemplateParserLogicSnafu {
                message: "Template parser did not find template body.",
            })?
            .as_str();

        Ok(Self {
            frontmatter: frontmatter_document,
            body: template_body.to_string(),
        })
    }
}

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Error when parsing template grammar: '{}'\n\nThis is usually due to errors in frontmatter TOML formatting.", source))]
        GrammarParse {
            source: pest::error::Error<super::Rule>,
        },

        #[snafu(display("Error when parsing template frontmatter: '{}'", source))]
        FrontmatterParse { source: toml::de::Error },

        #[snafu(display("Error in template parser: '{}'", message))]
        TemplateParserLogic { message: &'static str },
    }
}

pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
