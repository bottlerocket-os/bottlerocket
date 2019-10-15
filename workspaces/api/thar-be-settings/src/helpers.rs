// This module contains helpers for rendering templates. These helpers can
// be registerd with the Handlebars library to assist in manipulating
// text at render time.

use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use serde_json::value::Value;
use snafu::{OptionExt, ResultExt};

/// Potential errors during helper execution
mod error {
    use handlebars::RenderError;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum TemplateHelperError {
        #[snafu(display(
            "Incorrect number of params provided to helper '{}' in template '{}' - {} expected, {} received",
            helper,
            template,
            expected,
            received,
        ))]
        IncorrectNumberOfParams {
            expected: u8,
            received: usize,
            helper: String,
            template: String,
        },

        #[snafu(display("Internal error: {}", msg))]
        Internal { msg: String },

        // handlebars::JsonValue is a serde_json::Value, which implements
        // the 'Display' trait and should provide valuable context
        #[snafu(display(
            "Invalid base64 template value, expected {}, got '{}' in template {}",
            expected,
            value,
            template
        ))]
        InvalidTemplateValue {
            expected: &'static str,
            value: handlebars::JsonValue,
            template: String,
        },

        #[snafu(display(
            "Missing data and fail-if-missing was set; see given line/col in template '{}'",
            template,
        ))]
        MissingTemplateData { template: String },

        #[snafu(display(
            "Unable to base64 decode string '{}' in template '{}': '{}'",
            base64_string,
            template,
            source
        ))]
        Base64Decode {
            base64_string: String,
            template: String,
            source: base64::DecodeError,
        },

        #[snafu(display(
            "Invalid (non-utf8) output from base64 string '{}' in template '{}': '{}'",
            base64_string,
            template,
            source
        ))]
        InvalidUTF8 {
            base64_string: String,
            template: String,
            source: std::str::Utf8Error,
        },

        #[snafu(display("Unable to write template '{}': '{}'", template, source))]
        TemplateWrite {
            template: String,
            source: std::io::Error,
        },
    }

    // Handlebars helpers are required to return a RenderError.
    // Implement "From" for TemplateHelperError.
    impl From<TemplateHelperError> for RenderError {
        fn from(e: TemplateHelperError) -> RenderError {
            RenderError::with(e)
        }
    }
}

/// `base64_decode` decodes base64 encoded text at template render time.
/// It takes a single variable as a parameter: {{base64_decode var}}
pub fn base64_decode(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // To give context to our errors, get the template name.
    // In the context of thar-be-settings, all of our templates have
    // registered names, which means that the get_root_template_name()
    // call should return an intelligible and valid name.
    // To protect us in the unlikely event a template was registered
    // without a name, we return the text "dynamic template"
    trace!("Starting base64_decode helper");
    let template_name = renderctx
        .get_root_template_name()
        .map(|i| i.to_string())
        .unwrap_or_else(|| "dynamic template".to_string());
    trace!("Template name: {}", &template_name);

    // Check number of parameters, must be exactly one
    trace!("Number of params: {}", helper.params().len());

    if helper.params().len() != 1 {
        return Err(RenderError::from(
            error::TemplateHelperError::IncorrectNumberOfParams {
                expected: 1,
                received: helper.params().len(),
                helper: helper.name().to_string(),
                template: template_name,
            },
        ));
    }

    // Get the resolved key out of the template (param(0)). value() returns
    // a serde_json::Value
    let base64_value = helper
        .param(0)
        .map(|v| v.value())
        .context(error::Internal {
            msg: "Found no params after confirming there is one param",
        })?;
    trace!("Base64 value from template: {}", base64_value);

    // Create an &str from the serde_json::Value
    let base64_str = base64_value.as_str().context(error::InvalidTemplateValue {
        expected: "string",
        value: base64_value.to_owned(),
        template: template_name.to_owned(),
    })?;
    trace!("Base64 string from template: {}", base64_str);

    // Base64 decode the &str
    let decoded_bytes = base64::decode(&base64_str).context(error::Base64Decode {
        base64_string: base64_str.to_string(),
        template: template_name.to_owned(),
    })?;

    // Create a valid utf8 str
    let decoded = std::str::from_utf8(&decoded_bytes).context(error::InvalidUTF8 {
        base64_string: base64_str.to_string(),
        template: template_name.to_owned(),
    })?;
    trace!("Decoded base64: {}", decoded);

    // Write the string out to the template
    out.write(decoded).context(error::TemplateWrite {
        template: template_name.to_owned(),
    })?;
    Ok(())
}

/// `join_map` lets you join together strings in a map with given characters, for example when
/// you're writing values out to a configuration file.
///
/// The map is expected to be a single level deep, with string keys and string values.
///
/// The first parameter is the character to use to join keys to values; the second parameter is the
/// character to use to join pairs; the third parameter is the name of the map.  The third
/// parameter is a literal string that describes the behavior you want if the map is missing from
/// settings; "fail-if-missing" to fail the template, or "no-fail-if-missing" to continue but write
/// out nothing for this invocation of the helper.
///
/// Example:
///    {{ join_map "=" "," "fail-if-missing" map }}
///    ...where `map` is: {"hi": "there", "whats": "up"}
///    ...will produce: "hi=there,whats=up"
pub fn join_map(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting join_map helper");
    let template_name = renderctx
        .get_root_template_name()
        .map(|i| i.to_string())
        .unwrap_or_else(|| "dynamic template".to_string());
    trace!("Template name: {}", &template_name);

    trace!("Number of params: {}", helper.params().len());
    if helper.params().len() != 4 {
        return Err(RenderError::from(
            error::TemplateHelperError::IncorrectNumberOfParams {
                expected: 4,
                received: helper.params().len(),
                helper: helper.name().to_string(),
                template: template_name,
            },
        ));
    }

    // Pull out the parameters and confirm their types
    let join_key_val = helper
        .param(0)
        .map(|v| v.value())
        .context(error::Internal {
            msg: "Missing param after confirming there are enough",
        })?;
    let join_key = join_key_val
        .as_str()
        .with_context(|| error::InvalidTemplateValue {
            expected: "string",
            value: join_key_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Character used to join keys to values: {}", join_key);

    let join_pairs_val = helper
        .param(1)
        .map(|v| v.value())
        .context(error::Internal {
            msg: "Missing param after confirming there are enough",
        })?;
    let join_pairs = join_pairs_val
        .as_str()
        .with_context(|| error::InvalidTemplateValue {
            expected: "string",
            value: join_pairs_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Character used to join pairs: {}", join_pairs);

    let fail_behavior_val = helper
        .param(2)
        .map(|v| v.value())
        .context(error::Internal {
            msg: "Missing param after confirming there are enough",
        })?;
    let fail_behavior_str =
        fail_behavior_val
            .as_str()
            .with_context(|| error::InvalidTemplateValue {
                expected: "string",
                value: join_pairs_val.to_owned(),
                template: template_name.to_owned(),
            })?;
    let fail_if_missing = match fail_behavior_str {
        "fail-if-missing" => true,
        "no-fail-if-missing" => false,
        _ => {
            return Err(RenderError::from(
                error::TemplateHelperError::InvalidTemplateValue {
                    expected: "fail-if-missing or no-fail-if-missing",
                    value: fail_behavior_val.to_owned(),
                    template: template_name.to_owned(),
                },
            ))
        }
    };
    trace!(
        "Will we fail if missing the specified map: {}",
        fail_if_missing
    );

    let map_value = helper
        .param(3)
        .map(|v| v.value())
        .context(error::Internal {
            msg: "Missing param after confirming there are enough",
        })?;
    // If the requested setting is not set, we check the user's requested fail-if-missing behavior
    // to determine whether to fail hard or just write nothing quietly.
    if !map_value.is_object() {
        if fail_if_missing {
            return Err(RenderError::from(
                error::TemplateHelperError::MissingTemplateData {
                    template: template_name.to_owned(),
                },
            ));
        } else {
            return Ok(());
        }
    }
    let map = map_value.as_object().context(error::Internal {
        msg: "Already confirmed map is_object but as_object failed",
    })?;
    trace!("Map to join: {:?}", map);

    // Join the key/value pairs with requested string
    let mut pairs = Vec::new();
    for (key, val_value) in map.into_iter() {
        // We don't want the JSON form of scalars, we want the Display form of the Rust type inside.
        let val = match val_value {
            // these ones Display as their simple scalar selves
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_string(),
            // Null not supported; probably don't want blanks in config files, and we don't have a
            // use for this yet; consider carefully if/when we do
            Value::Null => {
                return Err(RenderError::from(
                    error::TemplateHelperError::InvalidTemplateValue {
                        expected: "non-null",
                        value: val_value.to_owned(),
                        template: template_name.to_owned(),
                    },
                ))
            }
            // composite types unsupported
            Value::Array(_) | Value::Object(_) => {
                return Err(RenderError::from(
                    error::TemplateHelperError::InvalidTemplateValue {
                        expected: "scalar",
                        value: val_value.to_owned(),
                        template: template_name.to_owned(),
                    },
                ))
            }
        };

        // Do the actual key/value join.
        pairs.push(format!("{}{}{}", key, join_key, val));
    }

    // Join all pairs with the given string.
    let joined = pairs.join(join_pairs);
    trace!("Joined output: {}", joined);

    // Write the string out to the template
    out.write(&joined).context(error::TemplateWrite {
        template: template_name.to_owned(),
    })?;
    Ok(())
}

#[cfg(test)]
mod test_base64_decode {
    use super::*;
    use handlebars::TemplateRenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, TemplateRenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("base64_decode", Box::new(base64_decode));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn renders_decoded_base64() {
        let result =
            setup_and_render_template("{{base64_decode var}}", &json!({"var": "SGk="})).unwrap();
        assert_eq!(result, "Hi")
    }

    #[test]
    fn does_not_render_invalid_base64() {
        assert!(setup_and_render_template("{{base64_decode var}}", &json!({"var": "hi"})).is_err())
    }

    #[test]
    fn does_not_render_invalid_utf8() {
        // "wygk" is the invalid UTF8 string "\xc3\x28" base64 encoded
        assert!(
            setup_and_render_template("{{base64_decode var}}", &json!({"var": "wygK"})).is_err()
        )
    }

    #[test]
    fn base64_helper_with_missing_param() {
        assert!(setup_and_render_template("{{base64_decode}}", &json!({"var": "foo"})).is_err());
    }

    #[test]
    fn base64_helper_with_extra_param() {
        assert!(setup_and_render_template(
            "{{base64_decode var1 var2}}",
            &json!({"var1": "Zm9v", "var2": "YmFy"})
        )
        .is_err());
    }
}

#[cfg(test)]
mod test_join_map {
    use super::*;
    use handlebars::TemplateRenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, TemplateRenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("join_map", Box::new(join_map));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn single_pair() {
        let result = setup_and_render_template(
            "{{join_map \"=\" \",\" \"fail-if-missing\" map}}",
            &json!({"map": {"hi": "there"}}),
        )
        .unwrap();
        assert_eq!(result, "hi=there")
    }

    #[test]
    fn basic() {
        let result = setup_and_render_template(
            "{{join_map \"=\" \",\" \"fail-if-missing\" map}}",
            &json!({"map": {"hi": "there", "whats": "up"}}),
        )
        .unwrap();
        assert_eq!(result, "hi=there,whats=up")
    }

    #[test]
    fn number() {
        let result = setup_and_render_template(
            "{{join_map \"=\" \",\" \"fail-if-missing\" map}}",
            &json!({"map": {"hi": 42}}),
        )
        .unwrap();
        assert_eq!(result, "hi=42")
    }

    #[test]
    fn boolean() {
        let result = setup_and_render_template(
            "{{join_map \"=\" \",\" \"fail-if-missing\" map}}",
            &json!({"map": {"hi": true}}),
        )
        .unwrap();
        assert_eq!(result, "hi=true")
    }

    #[test]
    fn invalid_nested_map() {
        setup_and_render_template(
            "{{join_map \"=\" \",\" \"fail-if-missing\" map}}",
            &json!({"map": {"hi": {"too": "deep"}}}),
        )
        .unwrap_err();
    }

    #[test]
    fn fail_if_missing() {
        setup_and_render_template(
            "{{join_map \"=\" \",\" \"fail-if-missing\" map}}",
            &json!({}),
        )
        // Requested failure if map was missing, should fail
        .unwrap_err();
    }

    #[test]
    fn no_fail_if_missing() {
        let result = setup_and_render_template(
            "{{join_map \"=\" \",\" \"no-fail-if-missing\" map}}",
            &json!({}),
        )
        .unwrap();
        // Requested no failure even if map was missing, should get no output
        assert_eq!(result, "")
    }

    #[test]
    fn invalid_fail_if_missing() {
        setup_and_render_template(
            "{{join_map \"=\" \",\" \"sup\" map}}",
            &json!({}),
        )
        // Invalid failure mode 'sup'
        .unwrap_err();
    }
}
