// This module contains helpers for rendering templates. These helpers can
// be registerd with the Handlebars library to assist in manipulating
// text at render time.

use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use lazy_static::lazy_static;
use serde_json::value::Value;
use snafu::{OptionExt, ResultExt};
use std::borrow::Borrow;
use std::collections::HashMap;
use url::Url;

lazy_static! {
    /// A map to tell us which registry to pull ECR images from for a given region.
    static ref ECR_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("af-south-1", "917644944286");
        m.insert("ap-east-1", "375569722642");
        m.insert("ap-northeast-1", "328549459982");
        m.insert("ap-northeast-2", "328549459982");
        m.insert("ap-south-1", "328549459982");
        m.insert("ap-southeast-1", "328549459982");
        m.insert("ap-southeast-2", "328549459982");
        m.insert("ca-central-1", "328549459982");
        m.insert("eu-central-1", "328549459982");
        m.insert("eu-north-1", "328549459982");
        m.insert("eu-south-1", "586180183710");
        m.insert("eu-west-1", "328549459982");
        m.insert("eu-west-2", "328549459982");
        m.insert("eu-west-3", "328549459982");
        m.insert("me-south-1", "509306038620");
        m.insert("sa-east-1", "328549459982");
        m.insert("us-east-1", "328549459982");
        m.insert("us-east-2", "328549459982");
        m.insert("us-west-1", "328549459982");
        m.insert("us-west-2", "328549459982");
        m
    };
}

/// But if there is a region that does not exist in our map (for example a new
/// region is created or being tested), then we will fallback to pulling ECR
/// containers from here.
const ECR_FALLBACK_REGION: &str = "us-east-1";
const ECR_FALLBACK_REGISTRY: &str = "328549459982";

/// Potential errors during helper execution
mod error {
    use handlebars::RenderError;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum TemplateHelperError {
        #[snafu(display(
            "Expected ecr helper to be called with either 'registry' or 'region', got '{}'",
            value,
        ))]
        EcrParam { value: String },

        #[snafu(display("Expected an AWS region, got '{}' in template {}", value, template))]
        EcrRegion {
            value: handlebars::JsonValue,
            template: String,
        },

        #[snafu(display(
            "Incorrect number of params provided to helper '{}' in template '{}' - {} expected, {} received",
            helper,
            template,
            expected,
            received,
        ))]
        IncorrectNumberOfParams {
            expected: usize,
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

        #[snafu(display("Missing param {} for helper '{}'", index, helper_name))]
        MissingParam { index: usize, helper_name: String },

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

        #[snafu(display(
            "Expected an absolute URL, got '{}' in template '{}': '{}'",
            url_str,
            template,
            source
        ))]
        UrlParse {
            url_str: String,
            template: String,
            source: url::ParseError,
        },

        #[snafu(display("URL '{}' is missing host component", url_str))]
        UrlHost { url_str: String },
    }

    // Handlebars helpers are required to return a RenderError.
    // Implement "From" for TemplateHelperError.
    impl From<TemplateHelperError> for RenderError {
        fn from(e: TemplateHelperError) -> RenderError {
            RenderError::from_error("TemplateHelperError", e)
        }
    }
}

/// `base64_decode` decodes base64 encoded text at template render time.
/// It takes a single variable as a parameter: {{base64_decode var}}
pub fn base64_decode(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // To give context to our errors, get the template name, if available.
    trace!("Starting base64_decode helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    // Check number of parameters, must be exactly one
    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 1)?;

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
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting join_map helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 4)?;

    // Pull out the parameters and confirm their types
    let join_key_val = get_param(helper, 0)?;
    let join_key = join_key_val
        .as_str()
        .with_context(|| error::InvalidTemplateValue {
            expected: "string",
            value: join_key_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Character used to join keys to values: {}", join_key);

    let join_pairs_val = get_param(helper, 1)?;
    let join_pairs = join_pairs_val
        .as_str()
        .with_context(|| error::InvalidTemplateValue {
            expected: "string",
            value: join_pairs_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Character used to join pairs: {}", join_pairs);

    let fail_behavior_val = get_param(helper, 2)?;
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

    let map_value = get_param(helper, 3)?;
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

/// `default` lets you specify the default value for a key in a template in case that key isn't
/// set.  The first argument is the default (scalar) value; the second argument is the key (with
/// scalar value) to check and insert if it is set.
pub fn default(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting default helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 2)?;

    // Pull out the parameters and confirm their types
    let default_val = get_param(helper, 0)?;
    let default = match default_val {
        // these ones Display as their simple scalar selves
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        // Null isn't allowed - we're here to give a default!
        // And composite types are unsupported.
        Value::Null | Value::Array(_) | Value::Object(_) => {
            return Err(RenderError::from(
                error::TemplateHelperError::InvalidTemplateValue {
                    expected: "non-null scalar",
                    value: default_val.to_owned(),
                    template: template_name.to_owned(),
                },
            ))
        }
    };
    trace!("Default value if key is not set: {}", default);

    let requested_value = get_param(helper, 1)?;
    let value = match requested_value {
        // these ones Display as their simple scalar selves
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        // If no value is set, use the given default.
        Value::Null => default.to_string(),
        // composite types unsupported
        Value::Array(_) | Value::Object(_) => {
            return Err(RenderError::from(
                error::TemplateHelperError::InvalidTemplateValue {
                    expected: "scalar",
                    value: requested_value.to_owned(),
                    template: template_name.to_owned(),
                },
            ))
        }
    };

    // Write the string out to the template
    out.write(&value).context(error::TemplateWrite {
        template: template_name.to_owned(),
    })?;
    Ok(())
}

/// The `ecr-prefix` helper is used to map an AWS region to the correct ECR
/// registry.
///
/// Initially we held all of our ECR repos in a single registry, but with some
/// regions this was no longer possible. Because the ECR repo URL includes the
/// the registry number, we created this helper to lookup the correct registry
/// number for a given region.
///
/// This helper takes the AWS region as its only parameter, and returns the
/// fully qualified domain name to the correct ECR registry.
///
/// # Fallback
///
/// A map of region to ECR registry ID is maintained herein. But if we do not
/// have the region in our map, a fallback region and registry number are
/// returned. This would allow a version of Bottlerocket to run in a new region
/// before this map has been updated.
///
/// # Example
///
/// In this example the registry number for the region will be returned.
/// `{{ ecr-prefix settings.aws.region }}`
///
/// This would result in something like:
/// `328549459982.dkr.ecr.eu-central-1.amazonaws.com`
pub fn ecr_prefix(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting ecr helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 1)?;

    // get the region parameter, which is probably given by the template value
    // settings.aws.region. regardless, we expect it to be a string.
    let aws_region = get_param(helper, 0)?;
    let aws_region = aws_region.as_str().with_context(|| error::EcrRegion {
        value: aws_region.to_owned(),
        template: template_name,
    })?;

    // construct the registry fqdn
    let ecr_registry = ecr_registry(aws_region);

    // write it to the template
    out.write(&ecr_registry)
        .with_context(|| error::TemplateWrite {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// `host` takes an absolute URL and trims it down and returns its host.
pub fn host(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting host helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 1)?;

    let url_val = get_param(helper, 0)?;
    let url_str = url_val
        .as_str()
        .with_context(|| error::InvalidTemplateValue {
            expected: "string",
            value: url_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    let url = Url::parse(url_str).context(error::UrlParse {
        url_str,
        template: template_name,
    })?;
    let url_host = url.host_str().context(error::UrlHost { url_str })?;

    // write it to the template
    out.write(&url_host).with_context(|| error::TemplateWrite {
        template: template_name.to_owned(),
    })?;

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
// helpers to the helpers

/// Gets the value at `idx` and unwraps it. Returns an error if the param cannot be unwrapped.
fn get_param<'a>(helper: &'a Helper<'_, '_>, idx: usize) -> Result<&'a Value, RenderError> {
    Ok(helper
        .param(idx)
        .map(|v| v.value())
        .context(error::MissingParam {
            index: idx,
            helper_name: helper.name(),
        })?)
}

/// Get the template name if there is one, otherwise return "dynamic template"
fn template_name<'a>(renderctx: &'a RenderContext<'_, '_>) -> &'a str {
    match renderctx.get_root_template_name() {
        Some(s) => s.as_str(),
        None => "dynamic template",
    }
}

/// Creates a an `IncorrectNumberofParams` error if the number of `helper`
/// params does not equal `expected`. Template name is only used in constructing
/// the error message.
fn check_param_count<S: AsRef<str>>(
    helper: &Helper<'_, '_>,
    template_name: S,
    expected: usize,
) -> Result<(), RenderError> {
    if helper.params().len() != expected {
        return Err(RenderError::from(
            error::TemplateHelperError::IncorrectNumberOfParams {
                expected,
                received: helper.params().len(),
                helper: helper.name().to_string(),
                template: template_name.as_ref().into(),
            },
        ));
    }
    Ok(())
}

/// Constructs the fully qualified domain name for the ECR registry for the
/// given region. Returns a default ECR registry if the region is not mapped.
fn ecr_registry<S: AsRef<str>>(region: S) -> String {
    // lookup the ecr registry ID or fallback to the default region and id
    let (region, registry_id) = match ECR_MAP.borrow().get(region.as_ref()) {
        None => (ECR_FALLBACK_REGION, ECR_FALLBACK_REGISTRY),
        Some(registry_id) => (region.as_ref(), *registry_id),
    };
    format!("{}.dkr.ecr.{}.amazonaws.com", registry_id, region)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

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
        setup_and_render_template("{{join_map \"=\" \",\" \"sup\" map}}", &json!({}))
            // Invalid failure mode 'sup'
            .unwrap_err();
    }
}

#[cfg(test)]
mod test_default {
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
        registry.register_helper("default", Box::new(default));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn have_setting() {
        let result =
            setup_and_render_template("{{default \"42\" setting}}", &json!({"setting": "hi"}))
                .unwrap();
        assert_eq!(result, "hi")
    }

    #[test]
    fn dont_have_setting() {
        let result = setup_and_render_template(
            "{{default \"42\" setting}}",
            &json!({"not-the-setting": "hi"}),
        )
        .unwrap();
        assert_eq!(result, "42")
    }

    #[test]
    fn have_setting_bool() {
        let result =
            setup_and_render_template("{{default \"42\" setting}}", &json!({"setting": true}))
                .unwrap();
        assert_eq!(result, "true")
    }

    #[test]
    fn dont_have_setting_bool() {
        let result = setup_and_render_template(
            "{{default \"42\" setting}}",
            &json!({"not-the-setting": true}),
        )
        .unwrap();
        assert_eq!(result, "42")
    }

    #[test]
    fn have_setting_number() {
        let result =
            setup_and_render_template("{{default \"42\" setting}}", &json!({"setting": 42.42}))
                .unwrap();
        assert_eq!(result, "42.42")
    }

    #[test]
    fn dont_have_setting_number() {
        let result = setup_and_render_template(
            "{{default \"42\" setting}}",
            &json!({"not-the-setting": 42.42}),
        )
        .unwrap();
        assert_eq!(result, "42")
    }

    #[test]
    fn number_default() {
        let result =
            setup_and_render_template("{{default 42 setting}}", &json!({"not-the-setting": 42.42}))
                .unwrap();
        assert_eq!(result, "42")
    }

    #[test]
    fn bool_default() {
        let result = setup_and_render_template(
            "{{default true setting}}",
            &json!({"not-the-setting": 42.42}),
        )
        .unwrap();
        assert_eq!(result, "true")
    }
}

#[cfg(test)]
mod test_ecr_registry {
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
        registry.register_helper("ecr-prefix", Box::new(ecr_prefix));

        registry.render_template(tmpl, data)
    }

    const ADMIN_CONTAINER_TEMPLATE: &str =
        "{{ ecr-prefix settings.aws.region }}/bottlerocket-admin:v0.5.1";

    const EXPECTED_URL_EU_CENTRAL_1: &str =
        "328549459982.dkr.ecr.eu-central-1.amazonaws.com/bottlerocket-admin:v0.5.1";

    const EXPECTED_URL_AF_SOUTH_1: &str =
        "917644944286.dkr.ecr.af-south-1.amazonaws.com/bottlerocket-admin:v0.5.1";

    const EXPECTED_URL_XY_ZTOWN_1: &str =
        "328549459982.dkr.ecr.us-east-1.amazonaws.com/bottlerocket-admin:v0.5.1";

    #[test]
    fn url_eu_central_1() {
        let result = setup_and_render_template(
            ADMIN_CONTAINER_TEMPLATE,
            &json!({"settings": {"aws": {"region": "eu-central-1"}}}),
        )
        .unwrap();
        assert_eq!(result, EXPECTED_URL_EU_CENTRAL_1);
    }

    #[test]
    fn url_af_south_1() {
        let result = setup_and_render_template(
            ADMIN_CONTAINER_TEMPLATE,
            &json!({"settings": {"aws": {"region": "af-south-1"}}}),
        )
        .unwrap();
        assert_eq!(result, EXPECTED_URL_AF_SOUTH_1);
    }

    #[test]
    fn url_fallback() {
        let result = setup_and_render_template(
            ADMIN_CONTAINER_TEMPLATE,
            &json!({"settings": {"aws": {"region": "xy-ztown-1"}}}),
        )
        .unwrap();
        assert_eq!(result, EXPECTED_URL_XY_ZTOWN_1);
    }
}

#[cfg(test)]
mod test_host {
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
        registry.register_helper("host", Box::new(host));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn not_absolute_url() {
        assert!(setup_and_render_template(
            "{{ host url_setting }}",
            &json!({"url_setting": "example.com"}),
        )
        .is_err());
    }

    #[test]
    fn https() {
        let result = setup_and_render_template(
            "{{ host url_setting }}",
            &json!({"url_setting": "https://example.example.com/example/example"}),
        )
        .unwrap();
        assert_eq!(result, "example.example.com");
    }

    #[test]
    fn http() {
        let result = setup_and_render_template(
            "{{ host url_setting }}",
            &json!({"url_setting": "http://example.com"}),
        )
        .unwrap();
        assert_eq!(result, "example.com");
    }

    #[test]
    fn unknown_scheme() {
        let result = setup_and_render_template(
            "{{ host url_setting }}",
            &json!({"url_setting": "foo://example.com"}),
        )
        .unwrap();
        assert_eq!(result, "example.com");
    }
}
