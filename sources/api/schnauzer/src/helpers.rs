// This module contains helpers for rendering templates. These helpers can
// be registered with the Handlebars library to assist in manipulating
// text at render time.

use dns_lookup::lookup_host;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, Output, RenderContext, RenderError,
};
use lazy_static::lazy_static;
use model::modeled_types::{OciDefaultsCapability, OciDefaultsResourceLimitType};
use model::OciDefaultsResourceLimit;
use serde::Deserialize;
use serde_json::value::Value;
use serde_plain::derive_fromstr_from_deserialize;
use snafu::{OptionExt, ResultExt};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use url::Url;

lazy_static! {
    /// A map to tell us which registry to pull ECR images from for a given region.
    static ref ECR_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("af-south-1", "917644944286");
        m.insert("ap-east-1", "375569722642");
        m.insert("ap-northeast-1", "328549459982");
        m.insert("ap-northeast-2", "328549459982");
        m.insert("ap-northeast-3", "328549459982");
        m.insert("ap-south-1", "328549459982");
        m.insert("ap-south-2", "764716012617");
        m.insert("ap-southeast-1", "328549459982");
        m.insert("ap-southeast-2", "328549459982");
        m.insert("ap-southeast-3", "386774335080");
        m.insert("ap-southeast-4", "731751899352");
        m.insert("ca-central-1", "328549459982");
        m.insert("ca-west-1", "253897149516");
        m.insert("cn-north-1", "183470599484");
        m.insert("cn-northwest-1", "183901325759");
        m.insert("eu-central-1", "328549459982");
        m.insert("eu-central-2", "861738308508");
        m.insert("eu-north-1", "328549459982");
        m.insert("eu-south-1", "586180183710");
        m.insert("eu-south-2", "620625777247");
        m.insert("eu-west-1", "328549459982");
        m.insert("eu-west-2", "328549459982");
        m.insert("eu-west-3", "328549459982");
        m.insert("il-central-1", "288123944683");
        m.insert("me-central-1", "553577323255");
        m.insert("me-south-1", "509306038620");
        m.insert("sa-east-1", "328549459982");
        m.insert("us-east-1", "328549459982");
        m.insert("us-east-2", "328549459982");
        m.insert("us-gov-east-1", "388230364387");
        m.insert("us-gov-west-1", "347163068887");
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

lazy_static! {
    /// A map to tell us which registry to pull pause container images from for a given region.
    static ref PAUSE_CONTAINER_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("af-south-1", "877085696533");
        m.insert("ap-east-1", "800184023465");
        m.insert("ap-northeast-1", "602401143452");
        m.insert("ap-northeast-2", "602401143452");
        m.insert("ap-northeast-3", "602401143452");
        m.insert("ap-south-1", "602401143452");
        m.insert("ap-south-2", "900889452093");
        m.insert("ap-southeast-1", "602401143452");
        m.insert("ap-southeast-2", "602401143452");
        m.insert("ap-southeast-3", "296578399912");
        m.insert("ap-southeast-4", "491585149902");
        m.insert("ca-central-1", "602401143452");
        m.insert("ca-west-1", "761377655185");
        m.insert("cn-north-1", "918309763551");
        m.insert("cn-northwest-1", "961992271922");
        m.insert("eu-central-1", "602401143452");
        m.insert("eu-central-2", "900612956339");
        m.insert("eu-north-1", "602401143452");
        m.insert("eu-south-1", "590381155156");
        m.insert("eu-south-2", "455263428931");
        m.insert("eu-west-1", "602401143452");
        m.insert("eu-west-2", "602401143452");
        m.insert("eu-west-3", "602401143452");
        m.insert("il-central-1", "066635153087");
        m.insert("me-central-1", "759879836304");
        m.insert("me-south-1", "558608220178");
        m.insert("sa-east-1", "602401143452");
        m.insert("us-east-1", "602401143452");
        m.insert("us-east-2", "602401143452");
        m.insert("us-gov-east-1", "151742754352");
        m.insert("us-gov-west-1", "013241004608");
        m.insert("us-west-1", "602401143452");
        m.insert("us-west-2", "602401143452");
        m
    };
}

/// But if there is a region that does not exist in our map (for example a new
/// region is created or being tested), then we will fall back to this.
const PAUSE_FALLBACK_REGISTRY: &str = "602401143452";
const PAUSE_FALLBACK_REGION: &str = "us-east-1";

lazy_static! {
    /// A map to tell us which endpoint to pull updates from for a given region.
    static ref TUF_ENDPOINT_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("cn-north-1", "bottlerocket-updates-cn-north-1.s3.dualstack");
        m.insert("cn-northwest-1", "bottlerocket-updates-cn-northwest-1.s3.dualstack");
        m
    };
}

const TUF_PUBLIC_REPOSITORY: &str = "https://updates.bottlerocket.aws";

lazy_static! {
    /// A map to tell us the partition for a given non-standard region.
    static ref ALT_PARTITION_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("cn-north-1", "aws-cn");
        m.insert("cn-northwest-1", "aws-cn");
        m.insert("us-gov-east-1", "aws-us-gov");
        m.insert("us-gov-west-1", "aws-us-gov");
        m
    };
}

/// The partition for standard AWS regions.
const STANDARD_PARTITION: &str = "aws";

/// The amount of CPU to reserve
/// We are using these CPU ranges from GKE
/// (https://cloud.google.com/kubernetes-engine/docs/concepts/cluster-architecture#node_allocatable):
/// 6% of the first core
/// 1% of the next core (up to 2 cores)
/// 0.5% of the next 2 cores (up to 4 cores)
/// 0.25% of any cores above 4 cores
const KUBE_RESERVE_1_CORE: f32 = 60.0;
const KUBE_RESERVE_2_CORES: f32 = KUBE_RESERVE_1_CORE + 10.0;
const KUBE_RESERVE_3_CORES: f32 = KUBE_RESERVE_2_CORES + 5.0;
const KUBE_RESERVE_4_CORES: f32 = KUBE_RESERVE_3_CORES + 5.0;
const KUBE_RESERVE_ADDITIONAL: f32 = 2.5;

const IPV4_LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const IPV6_LOCALHOST: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));

/// Potential errors during helper execution
mod error {
    use handlebars::RenderError;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum TemplateHelperError {
        #[snafu(display("Expected an AWS region, got '{}' in template {}", value, template))]
        AwsRegion {
            value: handlebars::JsonValue,
            template: String,
        },

        #[snafu(display(
            "Expected ecr helper to be called with either 'registry' or 'region', got '{}'",
            value,
        ))]
        EcrParam { value: String },

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

        #[snafu(display("Internal error: Missing param after confirming that it existed."))]
        ParamUnwrap {},

        #[snafu(display("Invalid OCI spec section: {}", source))]
        InvalidOciSpecSection { source: serde_plain::Error },

        // handlebars::JsonValue is a serde_json::Value, which implements
        // the 'Display' trait and should provide valuable context
        #[snafu(display(
            "Invalid template value, expected {}, got '{}' in template {}",
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
            "Unable to parse template value, expected {}, got '{}' in template {}: '{}'",
            expected,
            value,
            template,
            source,
        ))]
        UnparseableTemplateValue {
            source: serde_json::Error,
            expected: &'static str,
            value: handlebars::JsonValue,
            template: String,
        },

        #[snafu(display(
            "The join_array helper expected type '{}' while processing '{}' for template '{}'",
            expected_type,
            value,
            template
        ))]
        JoinStringsWrongType {
            expected_type: &'static str,
            value: handlebars::JsonValue,
            template: String,
        },

        #[snafu(display("Missing param {} for helper '{}'", index, helper_name))]
        MissingParam { index: usize, helper_name: String },

        #[snafu(display(
            "Missing parameter path for param {} for helper '{}'",
            index,
            helper_name
        ))]
        MissingParamPath { index: usize, helper_name: String },

        #[snafu(display(
            "Missing data and fail-if-missing was set; see given line/col in template '{}'",
            template,
        ))]
        MissingTemplateData { template: String },

        #[snafu(display("Unable to decode base64 in template '{}': '{}'", template, source))]
        Base64Decode {
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

        #[snafu(display("Unknown architecture '{}' given to goarch helper", given))]
        UnknownArch { given: String },

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

        #[snafu(display("Failed to convert {} {} to {}", what, number, target))]
        ConvertNumber {
            what: String,
            number: String,
            target: String,
        },

        #[snafu(display("Failed to convert usize {} to u16: {}", number, source))]
        ConvertUsizeToU16 {
            number: usize,
            source: std::num::TryFromIntError,
        },

        #[snafu(display("Invalid output type '{}', expected 'docker' or 'containerd'", runtime))]
        InvalidOutputType {
            source: serde_plain::Error,
            runtime: String,
        },
    }

    // Handlebars helpers are required to return a RenderError.
    // Implement "From" for TemplateHelperError.
    impl From<TemplateHelperError> for RenderError {
        fn from(e: TemplateHelperError) -> RenderError {
            RenderError::from_error("TemplateHelperError", e)
        }
    }
}

use error::TemplateHelperError;

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
        .context(error::ParamUnwrapSnafu {})?;
    trace!("Base64 value from template: {}", base64_value);

    // Create an &str from the serde_json::Value
    let base64_str = base64_value
        .as_str()
        .context(error::InvalidTemplateValueSnafu {
            expected: "string",
            value: base64_value.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Base64 string from template: {}", base64_str);

    // Base64 decode the &str
    let decoded_bytes = base64::decode(base64_str).context(error::Base64DecodeSnafu {
        template: template_name.to_owned(),
    })?;

    // Create a valid utf8 str
    let decoded = std::str::from_utf8(&decoded_bytes).context(error::InvalidUTF8Snafu {
        base64_string: base64_str.to_string(),
        template: template_name.to_owned(),
    })?;
    trace!("Decoded base64: {}", decoded);

    // Write the string out to the template
    out.write(decoded).context(error::TemplateWriteSnafu {
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
        .with_context(|| error::InvalidTemplateValueSnafu {
            expected: "string",
            value: join_key_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Character used to join keys to values: {}", join_key);

    let join_pairs_val = get_param(helper, 1)?;
    let join_pairs = join_pairs_val
        .as_str()
        .with_context(|| error::InvalidTemplateValueSnafu {
            expected: "string",
            value: join_pairs_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Character used to join pairs: {}", join_pairs);

    let fail_behavior_val = get_param(helper, 2)?;
    let fail_behavior_str =
        fail_behavior_val
            .as_str()
            .with_context(|| error::InvalidTemplateValueSnafu {
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
    let map = map_value.as_object().context(error::InternalSnafu {
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
    out.write(&joined).context(error::TemplateWriteSnafu {
        template: template_name.to_owned(),
    })?;
    Ok(())
}
/// `join_node_taints` is a specialized version of `join_map` that joins the kubernetes node taints
/// setting in the correct format `kubelet` expects for its `--register-with-taints` option.
///
/// Example:
///    {{ join_node_taints settings.kubernetes.node-taints }}
///    ...where `settings.kubernetes.node-taints` is: {"key1": ["value1:NoSchedule","value1:NoExecute"], "key2": ["value2:NoSchedule"]}
///    ...will produce: "key1=value1:NoSchedule,key1=value1:NoExecute,key2=value2:NoSchedule"
pub fn join_node_taints(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting join_node_taints helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 1)?;

    let node_taints_value = get_param(helper, 0)?;
    // It's ok if there are no node-taints, output nothing
    if !node_taints_value.is_object() {
        return Ok(());
    }

    let node_taints = node_taints_value
        .as_object()
        .context(error::InternalSnafu {
            msg: "Already confirmed map is_object but as_object failed",
        })?;
    trace!("node taints to join: {:?}", node_taints);

    // Join the key/value pairs for node taints
    let mut pairs = Vec::new();
    for (key, val_value) in node_taints.into_iter() {
        match val_value {
            Value::Array(values) => {
                for taint_value in values {
                    if let Some(taint_str) = taint_value.as_str() {
                        pairs.push(format!("{}={}", key, taint_str));
                    } else {
                        return Err(RenderError::from(
                            error::TemplateHelperError::InvalidTemplateValue {
                                expected: "string",
                                value: taint_value.to_owned(),
                                template: template_name.to_owned(),
                            },
                        ));
                    }
                }
            }
            Value::Null => {
                return Err(RenderError::from(
                    error::TemplateHelperError::InvalidTemplateValue {
                        expected: "non-null",
                        value: val_value.to_owned(),
                        template: template_name.to_owned(),
                    },
                ))
            }
            // all other types unsupported
            _ => {
                return Err(RenderError::from(
                    error::TemplateHelperError::InvalidTemplateValue {
                        expected: "sequence",
                        value: val_value.to_owned(),
                        template: template_name.to_owned(),
                    },
                ))
            }
        };
    }

    // Join all pairs with the given string.
    let joined = pairs.join(",");
    trace!("Joined output: {}", joined);

    // Write the string out to the template
    out.write(&joined).context(error::TemplateWriteSnafu {
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
        Value::Null => default,
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
    out.write(&value).context(error::TemplateWriteSnafu {
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
    let aws_region = aws_region.as_str().with_context(|| error::AwsRegionSnafu {
        value: aws_region.to_owned(),
        template: template_name,
    })?;

    // construct the registry fqdn
    let ecr_registry = ecr_registry(aws_region);

    // write it to the template
    out.write(&ecr_registry)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// The `pause-prefix` helper is used to map an AWS region to the correct pause
/// container registry.
///
/// Because the repo URL includes the the registry number, we created this helper
/// to lookup the correct registry number for a given region.
///
/// This helper takes the AWS region as its only parameter, and returns the
/// fully qualified domain name to the correct registry.
///
/// # Fallback
///
/// If we do not have the region in our map, a fallback region and registry number
/// are returned.  This would allow a version of Bottlerocket to run in a new region
/// before this map has been updated.
///
/// # Example
///
/// In this example the registry number for the region will be returned.
/// `{{ pause-prefix settings.aws.region }}`
///
/// This would result in something like:
/// `602401143452.dkr.ecr.eu-central-1.amazonaws.com`
pub fn pause_prefix(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting pause prefix helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 1)?;

    // get the region parameter, which is probably given by the template value
    // settings.aws.region. regardless, we expect it to be a string.
    let aws_region = get_param(helper, 0)?;
    let aws_region = aws_region.as_str().with_context(|| error::AwsRegionSnafu {
        value: aws_region.to_owned(),
        template: template_name,
    })?;

    // construct the registry fqdn
    let pause_registry = pause_registry(aws_region);

    // write it to the template
    out.write(&pause_registry)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// The `tuf-prefix` helper is used to map an AWS region to the correct TUF
/// repository.
///
/// This helper takes the AWS region as its only parameter, and returns the
/// fully qualified domain name to the correct TUF repository.
///
/// # Fallback
///
/// A map of region to TUF repository endpoint is maintained herein. But if we
/// do not have the region in our map, a fallback repository is returned. This
/// would allow a version of Bottlerocket to run in a new region before this map
/// has been updated.
///
/// # Example
///
/// In this example the repository endpoint for the region will be returned.
/// `{{ tuf-prefix settings.aws.region }}`
///
/// This would result in something like:
/// `https://bottlerocket-updates-us-west-2.s3.dualstack.us-west-2.amazonaws.com/latest`
pub fn tuf_prefix(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting tuf helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 1)?;

    // get the region parameter, which is probably given by the template value
    // settings.aws.region. regardless, we expect it to be a string.
    let aws_region = get_param(helper, 0)?;
    let aws_region = aws_region.as_str().with_context(|| error::AwsRegionSnafu {
        value: aws_region.to_owned(),
        template: template_name,
    })?;

    // construct the registry fqdn
    let tuf_repository = tuf_repository(aws_region);

    // write it to the template
    out.write(&tuf_repository)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// The `metadata-prefix` helper is used to map an AWS region to the correct
/// metadata location inside of the TUF repository.
///
/// This helper takes the AWS region as its only parameter, and returns the
/// prefix of the metadata.
///
pub fn metadata_prefix(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting tuf helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 1)?;

    // get the region parameter, which is probably given by the template value
    // settings.aws.region. regardless, we expect it to be a string.
    let aws_region = get_param(helper, 0)?;
    let aws_region = aws_region.as_str().with_context(|| error::AwsRegionSnafu {
        value: aws_region.to_owned(),
        template: template_name,
    })?;

    // construct the prefix
    let metadata_location = if TUF_ENDPOINT_MAP.contains_key(aws_region) {
        "/metadata"
    } else {
        ""
    };

    // write it to the template
    out.write(metadata_location)
        .with_context(|_| error::TemplateWriteSnafu {
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
        .with_context(|| error::InvalidTemplateValueSnafu {
            expected: "string",
            value: url_val.to_owned(),
            template: template_name.to_owned(),
        })?;
    let url = Url::parse(url_str).context(error::UrlParseSnafu {
        url_str,
        template: template_name,
    })?;
    let url_host = url.host_str().context(error::UrlHostSnafu { url_str })?;

    // write it to the template
    out.write(url_host)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// `goarch` takes one parameter, the name of a machine architecture, and converts it to the "Go"
/// form, so named because its use in golang popularized it elsewhere.
///
/// The canonical architecture names in Bottlerocket are things like "x86_64" and "aarch64"; goarch
/// converts these to "amd64" and "arm64", etc.
pub fn goarch(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting goarch helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 1)?;

    // Retrieve the given arch string
    let arch_val = get_param(helper, 0)?;
    let arch_str = arch_val
        .as_str()
        .with_context(|| error::InvalidTemplateValueSnafu {
            expected: "string",
            value: arch_val.to_owned(),
            template: template_name.to_owned(),
        })?;

    // Transform the arch string
    let goarch = match arch_str {
        "x86_64" | "amd64" => "amd64",
        "aarch64" | "arm64" => "arm64",
        _ => {
            return Err(RenderError::from(error::TemplateHelperError::UnknownArch {
                given: arch_str.to_string(),
            }))
        }
    };

    // write it to the template
    out.write(goarch)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// `join_array` is used to join an array of scalar strings into an array of
/// quoted, delimited strings. The delimiter must be specified.
///
/// # Example
///
/// Consider an array of values: `[ "a", "b", "c" ]` stored in a setting such as
/// `settings.somewhere.foo-list`. In our template we can write:
/// `{{ join_array ", " settings.somewhere.foo-list }}`
///
/// This will render `"a", "b", "c"`.
pub fn join_array(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting join_array helper");
    let template_name = template_name(renderctx);
    check_param_count(helper, template_name, 2)?;

    // get the delimiter
    let delimiter_param = get_param(helper, 0)?;
    let delimiter = delimiter_param
        .as_str()
        .with_context(|| error::JoinStringsWrongTypeSnafu {
            expected_type: "string",
            value: delimiter_param.to_owned(),
            template: template_name,
        })?;

    // get the array
    let array_param = get_param(helper, 1)?;
    let array = array_param
        .as_array()
        .with_context(|| error::JoinStringsWrongTypeSnafu {
            expected_type: "array",
            value: array_param.to_owned(),
            template: template_name,
        })?;

    let mut result = String::new();
    for (i, value) in array.iter().enumerate() {
        if i > 0 {
            result.push_str(delimiter);
        }
        result.push_str(
            format!(
                "\"{}\"",
                value.as_str().context(error::JoinStringsWrongTypeSnafu {
                    expected_type: "string",
                    value: array.to_owned(),
                    template: template_name,
                })?
            )
            .as_str(),
        );
    }

    // write it to the template
    out.write(&result)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// kube_reserve_memory and kube_reserve_cpu are taken from EKS' calculations.
/// https://github.com/awslabs/amazon-eks-ami/blob/db28da15d2b696bc08ac3aacc9675694f4a69933/files/bootstrap.sh

/// Calculates the amount of memory to reserve for kubeReserved in mebibytes.
/// Formula: memory_to_reserve = max_num_pods * 11 + 255 is taken from
/// https://github.com/awslabs/amazon-eks-ami/pull/419#issuecomment-609985305
pub fn kube_reserve_memory(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting kube_reserve_memory helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 2)?;

    let max_num_pods_val = get_param(helper, 0)?;
    let max_num_pods = match max_num_pods_val {
        Value::Number(n) => n,

        _ => {
            return Err(RenderError::from(
                error::TemplateHelperError::InvalidTemplateValue {
                    expected: "number",
                    value: max_num_pods_val.to_owned(),
                    template: template_name.to_owned(),
                },
            ))
        }
    };
    let max_num_pods = max_num_pods
        .as_u64()
        .with_context(|| error::ConvertNumberSnafu {
            what: "number of pods",
            number: max_num_pods.to_string(),
            target: "u64",
        })?;

    // Calculates the amount of memory to reserve
    let memory_to_reserve_value = get_param(helper, 1)?;
    let memory_to_reserve = match memory_to_reserve_value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        // If no value is set, use the given default.
        Value::Null => {
            format!("{}Mi", (max_num_pods * 11 + 255))
        }
        // composite types unsupported
        _ => {
            return Err(RenderError::from(
                error::TemplateHelperError::InvalidTemplateValue {
                    expected: "scalar",
                    value: memory_to_reserve_value.to_owned(),
                    template: template_name.to_owned(),
                },
            ))
        }
    };

    // write it to the template
    out.write(&memory_to_reserve)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// Get the amount of CPU to reserve for kubeReserved in millicores
pub fn kube_reserve_cpu(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("Starting kube_reserve_cpu helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 1)?;

    // Calculates the amount of CPU to reserve
    let num_cores = num_cpus::get();
    let cpu_to_reserve_value = get_param(helper, 0)?;
    let cpu_to_reserve = match cpu_to_reserve_value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        // If no value is set, use the given default.
        Value::Null => kube_cpu_helper(num_cores)?,
        // composite types unsupported
        _ => {
            return Err(RenderError::from(
                error::TemplateHelperError::InvalidTemplateValue {
                    expected: "scalar",
                    value: cpu_to_reserve_value.to_owned(),
                    template: template_name.to_owned(),
                },
            ))
        }
    };

    // write it to the template
    out.write(&cpu_to_reserve)
        .with_context(|_| error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// Completes `localhost` alias lines in /etc/hosts by returning a series of space-delimited host aliases.
///
/// This helper reconciles `settings.network.hostname` and `settings.network.hosts` references to loopback.
/// * `hostname`: Attempts to resolve the current configured hostname in DNS. If unsuccessful, the return
///   includes an alias for the hostname to be included for the given IP version.
/// * `hosts`: For any static `/etc/hosts` mappings which refer to loopback, this includes aliases in the
///   same order specified in `settings.network.hosts`. These settings take the lowest precedence for
///   loopback aliases.
pub fn localhost_aliases(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // To give context to our errors, get the template name, if available.
    trace!("Starting localhost_aliases helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    // Check number of parameters, must be exactly three (IP version, hostname, hosts overrides)
    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 3)?;

    // Get the resolved keys out of the template. value() returns a serde_json::Value
    let ip_version_value = helper
        .param(0)
        .map(|v| v.value())
        .context(error::ParamUnwrapSnafu {})?;
    trace!("IP version value from template: {}", ip_version_value);

    let hostname_value = helper
        .param(1)
        .map(|v| v.value())
        .context(error::ParamUnwrapSnafu {})?;
    trace!("Hostname value from template: {}", hostname_value);

    let hosts_value = helper
        .param(2)
        .map(|v| v.value())
        .context(error::ParamUnwrapSnafu {})?;
    trace!("Hosts value from template: {}", hosts_value);

    // Extract our variables from their serde_json::Value objects
    let ip_version = ip_version_value
        .as_str()
        .context(error::InvalidTemplateValueSnafu {
            expected: "string",
            value: ip_version_value.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("IP version string from template: {}", ip_version);

    let localhost_comparator = match ip_version {
        "ipv4" => IPV4_LOCALHOST,
        "ipv6" => IPV6_LOCALHOST,
        _ => {
            return Err(error::TemplateHelperError::InvalidTemplateValue {
                expected: r#"one of ("ipv4", "ipv6")"#,
                value: ip_version_value.to_owned(),
                template: template_name.to_owned(),
            }
            .into());
        }
    };

    let hostname = hostname_value
        .as_str()
        .context(error::InvalidTemplateValueSnafu {
            expected: "string",
            value: hostname_value.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Hostname string from template: {}", hostname);

    let mut results: Vec<String> = vec![];

    let hosts: Option<model::modeled_types::EtcHostsEntries> = (!hosts_value.is_null())
        .then(|| {
            serde_json::from_value(hosts_value.clone()).context(
                error::UnparseableTemplateValueSnafu {
                    expected: "EtcHostsEntries",
                    value: hosts_value.to_owned(),
                    template: template_name.to_owned(),
                },
            )
        })
        .transpose()?;
    trace!("Hosts from template: {:?}", hosts);

    // If our hostname isn't resolveable, add it to the alias list.
    if !hostname.is_empty() && !hostname_resolveable(hostname, hosts.as_ref()) {
        results.push(hostname.to_owned());
    }

    // If hosts are specified and any overrides exist for loopback, add them.
    if let Some(hosts) = hosts {
        // If any static mappings in `settings.network.hosts` are for localhost, add them as well.
        if let Some((_, aliases)) = hosts
            .iter_merged()
            .find(|(ip_address, _)| *ip_address == localhost_comparator)
        {
            // Downcast our hostnames into Strings and append to the results
            let mut hostname_aliases: Vec<String> = aliases
                .into_iter()
                .map(|a| a.as_ref().to_string())
                .collect();
            results.append(&mut hostname_aliases);
        }
    }

    // Write out our localhost aliases.
    let localhost_aliases = results.join(" ");
    out.write(&localhost_aliases)
        .context(error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// This helper writes out /etc/hosts lines based on `network.settings.hosts`.
///
/// The map of <IpAddr => Vec<HostAlias>> is written as newline-delimited text lines.
/// Any entries which reference localhost are ignored, as these are intended to be merged
/// with the existing localhost entries via `localhost_aliases`.
pub fn etc_hosts_entries(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // To give context to our errors, get the template name, if available.
    trace!("Starting etc_hosts_entries helper");
    let template_name = template_name(renderctx);
    trace!("Template name: {}", &template_name);

    // Check number of parameters, must be exactly one (hosts overrides)
    trace!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 1)?;

    // Get the resolved keys out of the template. value() returns a serde_json::Value
    let hosts_value = helper
        .param(0)
        .map(|v| v.value())
        .context(error::ParamUnwrapSnafu {})?;
    trace!("Hosts value from template: {}", hosts_value);

    if hosts_value.is_null() {
        // If hosts aren't set, just exit.
        return Ok(());
    }
    // Otherwise we need to generate /etc/hosts lines, ignoring loopback.
    let mut result_lines: Vec<String> = Vec::new();

    let hosts: model::modeled_types::EtcHostsEntries = serde_json::from_value(hosts_value.clone())
        .context(error::UnparseableTemplateValueSnafu {
            expected: "EtcHostsEntries",
            value: hosts_value.to_owned(),
            template: template_name.to_owned(),
        })?;
    trace!("Hosts from template: {:?}", hosts);

    hosts
        .iter_merged()
        .filter(|(ip_address, _)| {
            // Localhost aliases are handled by the `localhost_aliases` helper, so we disregard them here.
            *ip_address != IPV4_LOCALHOST && *ip_address != IPV6_LOCALHOST
        })
        .for_each(|(ip_address, aliases)| {
            // Downcast hostnames to Strings and render the /etc/hosts line.
            let alias_strs: Vec<String> = aliases.iter().map(|a| a.as_ref().into()).collect();

            result_lines.push(format!("{} {}", ip_address, alias_strs.join(" ")));
        });

    out.write(&result_lines.join("\n"))
        .context(error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

// This helper checks if any objects have '"enabled": true' in their properties.
//
// any_enabled takes one argument that is expected to be an array of objects,
// then checks if any of these objects have an "enabled" property set to true.
// If the argument is not an array of objects, does not have an "enabled" property,
// or does not include any objects with this property set to true, it evaluates
// to be falsey.
handlebars_helper!(any_enabled: |arg: Value| {
    #[derive(Deserialize)]
    struct EnablableObject {
        enabled: bool,
    }

    let mut result = false;
    match arg {
        Value::Array(items) => {
            for item in &items {
                let res = serde_json::from_value::<EnablableObject>(item.clone());
                if let Ok(eo) = res {
                    if eo.enabled {
                        result = true;
                        break;
                    }
                }
            }
        }
        Value::Object(items) => {
            for (_, item) in &items {
                let res = serde_json::from_value::<EnablableObject>(item.clone());
                if let Ok(eo) = res {
                    if eo.enabled {
                        result = true;
                        break;
                    }
                }
            }
        }
        _ => ()
    }
    result
});

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OciSpecSection {
    Capabilities,
    ResourceLimits,
}

derive_fromstr_from_deserialize!(OciSpecSection);

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum Runtime {
    Docker,
    Containerd,
}

derive_fromstr_from_deserialize!(Runtime);

impl Runtime {
    fn get_capabilities(&self, caps: String) -> String {
        match self {
            Self::Docker => Docker::get_capabilities(caps),
            Self::Containerd => Containerd::get_capabilities(caps),
        }
    }

    fn get_resource_limits(
        &self,
        rlimit_type: &OciDefaultsResourceLimitType,
        values: &OciDefaultsResourceLimit,
    ) -> String {
        match self {
            Self::Docker => Docker::get_resource_limits(rlimit_type, values),
            Self::Containerd => Containerd::get_resource_limits(rlimit_type, values),
        }
    }
}

struct Docker;
struct Containerd;

impl Docker {
    /// Formats capabilities for Docker
    fn get_capabilities(caps: String) -> String {
        format!(concat!(r#"["#, "{capabilities}", "]",), capabilities = caps)
    }

    /// Formats resource limits for Docker
    fn get_resource_limits(
        rlimit_type: &OciDefaultsResourceLimitType,
        values: &OciDefaultsResourceLimit,
    ) -> String {
        format!(
            r#" "{}":{{ "Name": "{}", "Hard": {}, "Soft": {} }}"#,
            rlimit_type
                .to_linux_string()
                .replace("RLIMIT_", "")
                .to_lowercase(),
            rlimit_type
                .to_linux_string()
                .replace("RLIMIT_", "")
                .to_lowercase(),
            values.hard_limit,
            values.soft_limit,
        )
    }
}

impl Containerd {
    /// Formats capabilities for Containerd
    fn get_capabilities(caps: String) -> String {
        format!(
            concat!(
                r#""bounding": ["#,
                "{capabilities_bounding}",
                "],\n",
                r#""effective": ["#,
                "{capabilities_effective}",
                "],\n",
                r#""permitted": ["#,
                "{capabilities_permitted}",
                "]\n",
            ),
            capabilities_bounding = caps,
            capabilities_effective = caps,
            capabilities_permitted = caps,
        )
    }

    /// Formats resource limits for Containerd
    fn get_resource_limits(
        rlimit_type: &OciDefaultsResourceLimitType,
        values: &OciDefaultsResourceLimit,
    ) -> String {
        format!(
            r#"{{ "type": "{}", "hard": {}, "soft": {} }}"#,
            rlimit_type.to_linux_string(),
            Self::get_limit(values.hard_limit),
            Self::get_limit(values.soft_limit),
        )
    }

    /// Converts I64 values to u64 for Containerd
    fn get_limit(limit: i64) -> u64 {
        match limit {
            -1 => u64::MAX,
            _ => limit as u64,
        }
    }
}

/// This helper writes out the default OCI runtime spec.
///
/// The calling pattern is `{{ oci_defaults settings.oci-defaults.resource-limits }}`,
/// where `settings.oci-defaults.resource-limits` is a map of OCI defaults
/// settings and their values, as defined in the model.
/// See the following file for more details:
/// * `sources/models/src/modeled_types/oci_defaults.rs`
///
/// This helper function extracts the setting name from the settings map
/// parameter. Specific setting names are expected, or else this helper
/// will return an error. The specific setting names are defined in the
/// following file:
/// * `sources/models/src/lib.rs`
pub fn oci_defaults(
    helper: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    renderctx: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // To give context to our errors, get the template name (e.g. what file we are rendering), if available.
    debug!("Starting oci_defaults helper");
    let template_name = template_name(renderctx);
    debug!("Template name: {}", &template_name);

    // Check number of parameters, must be exactly two (OCI spec section to render and settings values for the section)
    debug!("Number of params: {}", helper.params().len());
    check_param_count(helper, template_name, 2)?;
    debug!("params: {:?}", helper.params());

    debug!("Getting the requested output type to render");
    let runtime_val = get_param(helper, 0)?;
    let runtime_str = runtime_val
        .as_str()
        .with_context(|| error::InvalidTemplateValueSnafu {
            expected: "string",
            value: runtime_val.to_owned(),
            template: template_name.to_owned(),
        })?;

    let runtime = Runtime::from_str(runtime_str).context(error::InvalidOutputTypeSnafu {
        runtime: runtime_str.to_owned(),
    })?;

    debug!("Getting the requested OCI spec section to render");
    let oci_defaults_values = get_param(helper, 1)?;
    // We want the settings path so we know which OCI spec section we are rendering.
    // e.g. settings.oci-defaults.resource-limits
    let settings_path = get_param_key_name(helper, 1)?;
    // Extract the last part of the settings path, which is the OCI spec section we want to render.
    let oci_spec_section = settings_path
        .split('.')
        .last()
        .expect("did not find (got None for) an oci_spec_section");

    // Render the requested OCI spec section
    let section =
        OciSpecSection::from_str(oci_spec_section).context(error::InvalidOciSpecSectionSnafu)?;
    let result_lines = match section {
        OciSpecSection::Capabilities => {
            let capabilities = oci_spec_capabilities(oci_defaults_values)?;
            runtime.get_capabilities(capabilities)
        }
        OciSpecSection::ResourceLimits => {
            let rlimits = oci_spec_resource_limits(oci_defaults_values)?;
            rlimits
                .iter()
                .map(|(rlimit_type, values)| runtime.get_resource_limits(rlimit_type, values))
                .collect::<Vec<String>>()
                .join(",\n")
        }
    };

    debug!("{}_section: \n{}", oci_spec_section, result_lines);

    // Write out the final values to the configuration file
    out.write(result_lines.as_str())
        .context(error::TemplateWriteSnafu {
            template: template_name.to_owned(),
        })?;

    Ok(())
}

/// This helper writes out the capabilities section of the default
/// OCI runtime spec that `containerd` will use.
///
/// This function is called by the `oci_defaults` helper function,
/// specifically when matching over the `oci_spec_section` parameter
/// to determine which section of the OCI spec to render.
///
/// This helper function generates the linux capabilities section of
/// the OCI runtime spec from the provided `value` parameter, which is
/// the settings data from the datastore (`settings.oci-defaults.capabilities`).
fn oci_spec_capabilities(value: &Value) -> Result<String, RenderError> {
    let oci_default_capabilities: HashMap<OciDefaultsCapability, bool> =
        serde_json::from_value(value.clone())?;

    // Output the capabilities that are enabled
    let mut capabilities_lines: Vec<String> = oci_default_capabilities
        .iter()
        .filter(|(_, &capability_enabled)| capability_enabled)
        .map(|(&capability, _)| format!("\"{}\"", capability.to_linux_string()))
        .collect();

    // Sort for consistency for human-readers of the OCI spec defaults file.
    capabilities_lines.sort();
    let capabilities_lines_joined = capabilities_lines.join(",\n");

    Ok(capabilities_lines_joined)
}

/// This helper writes out the resource limits section of the default
/// OCI runtime spec that `containerd` will use.
///
/// This function is called by the `oci_defaults` helper function,
/// specifically when matching over the `oci_spec_section` parameter
/// to determine which section of the OCI spec to render.
///
/// This helper function generates the resource limits section of
/// the OCI runtime spec from the provided `value` parameter, which is
/// the settings data from the datastore (`settings.oci-defaults.resource-limits`).
fn oci_spec_resource_limits(
    value: &Value,
) -> Result<HashMap<OciDefaultsResourceLimitType, OciDefaultsResourceLimit>, RenderError> {
    Ok(serde_json::from_value(value.clone())?)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
// helpers to the helpers

/// Gets the key name (path) of the parameter at `index`. Returns an error if the param cannot be extracted.
fn get_param_key_name<'a>(
    helper: &'a Helper<'_, '_>,
    index: usize,
) -> Result<&'a String, RenderError> {
    Ok(helper
        .params()
        .get(index)
        .context(error::MissingParamSnafu {
            index,
            helper_name: helper.name(),
        })?
        .relative_path()
        .context(error::MissingParamPathSnafu {
            index,
            helper_name: helper.name(),
        })?)
}

/// Gets the value at `idx` and unwraps it. Returns an error if the param cannot be unwrapped.
fn get_param<'a>(helper: &'a Helper<'_, '_>, idx: usize) -> Result<&'a Value, RenderError> {
    Ok(helper
        .param(idx)
        .map(|v| v.value())
        .context(error::MissingParamSnafu {
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
    let partition = match ALT_PARTITION_MAP.borrow().get(region) {
        None => STANDARD_PARTITION,
        Some(partition) => *partition,
    };
    match partition {
        "aws-cn" => format!("{}.dkr.ecr.{}.amazonaws.com.cn", registry_id, region),
        _ => format!("{}.dkr.ecr.{}.amazonaws.com", registry_id, region),
    }
}

/// Constructs the fully qualified domain name for the pause container (pod infra
/// container) for the given region. Returns a default if the region is not mapped.
fn pause_registry<S: AsRef<str>>(region: S) -> String {
    // lookup the registry ID or fallback to the default region and id
    let (region, registry_id) = match PAUSE_CONTAINER_MAP.borrow().get(region.as_ref()) {
        None => (PAUSE_FALLBACK_REGION, PAUSE_FALLBACK_REGISTRY),
        Some(registry_id) => (region.as_ref(), *registry_id),
    };
    let partition = match ALT_PARTITION_MAP.borrow().get(region) {
        None => STANDARD_PARTITION,
        Some(partition) => *partition,
    };
    match partition {
        "aws-cn" => format!("{}.dkr.ecr.{}.amazonaws.com.cn", registry_id, region),
        _ => format!("{}.dkr.ecr.{}.amazonaws.com", registry_id, region),
    }
}

/// Constructs the fully qualified domain name for the TUF repository for the
/// given region. Returns a default if the region is not mapped.
fn tuf_repository<S: AsRef<str>>(region: S) -> String {
    // lookup the repository endpoint or fallback to the public url
    let (region, endpoint) = match TUF_ENDPOINT_MAP.borrow().get(region.as_ref()) {
        None => return TUF_PUBLIC_REPOSITORY.to_string(),
        Some(endpoint) => (region.as_ref(), *endpoint),
    };
    let partition = match ALT_PARTITION_MAP.borrow().get(region) {
        None => STANDARD_PARTITION,
        Some(partition) => *partition,
    };
    match partition {
        "aws-cn" => format!("https://{}.{}.amazonaws.com.cn/latest", endpoint, region),
        _ => format!("https://{}.{}.amazonaws.com/latest", endpoint, region),
    }
}

/// Calculates and returns the amount of CPU to reserve
fn kube_cpu_helper(num_cores: usize) -> Result<String, TemplateHelperError> {
    let num_cores =
        u16::try_from(num_cores).context(error::ConvertUsizeToU16Snafu { number: num_cores })?;
    let millicores_unit = "m";
    let cpu_to_reserve = match num_cores {
        0 => 0.0,
        1 => KUBE_RESERVE_1_CORE,
        2 => KUBE_RESERVE_2_CORES,
        3 => KUBE_RESERVE_3_CORES,
        4 => KUBE_RESERVE_4_CORES,
        _ => {
            let num_cores = f32::from(num_cores);
            KUBE_RESERVE_4_CORES + ((num_cores - 4.0) * KUBE_RESERVE_ADDITIONAL)
        }
    };
    Ok(format!("{}{}", cpu_to_reserve.floor(), millicores_unit))
}

/// Returns whether or not a hostname resolves to a non-loopback IP address.
///
/// If `configured_hosts` is set, the hostname will be considered resolvable if it is listed as an alias for any given IP address.
fn hostname_resolveable(
    hostname: &str,
    configured_hosts: Option<&model::modeled_types::EtcHostsEntries>,
) -> bool {
    // If the hostname is in our configured hosts, then it *will* be resolvable when /etc/hosts is rendered.
    // Note that DNS search paths in /etc/resolv.conf are not relevant here, as they are not checked when searching /etc/hosts.
    if let Some(etc_hosts_entries) = configured_hosts {
        for (_, alias_list) in etc_hosts_entries.iter_merged() {
            if alias_list.iter().any(|alias| alias == hostname) {
                return true;
            }
        }
    }

    // Attempt to resolve the hostname
    match lookup_host(hostname) {
        Ok(ip_list) => {
            // If the list of IPs is empty or resolves to localhost, consider the hostname
            // unresolvable
            let resolves_to_localhost = ip_list
                .iter()
                .any(|ip| ip == &IPV4_LOCALHOST || ip == &IPV6_LOCALHOST);

            !(ip_list.is_empty() || resolves_to_localhost)
        }
        Err(e) => {
            trace!("DNS hostname lookup failed: {},", e);
            false
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[cfg(test)]
mod test_base64_decode {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
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
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
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
mod test_join_node_taints {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("join_node_taints", Box::new(join_node_taints));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn basic() {
        let result = setup_and_render_template(
            "{{ join_node_taints map }}",
            &json!({"map":{"key1": ["value1:NoSchedule"], "key2": ["value2:NoSchedule"]}}),
        )
        .unwrap();
        assert_eq!(result, "key1=value1:NoSchedule,key2=value2:NoSchedule")
    }

    #[test]
    fn none() {
        let result = setup_and_render_template("{{ join_node_taints map }}", &json!({})).unwrap();
        assert_eq!(result, "")
    }

    #[test]
    fn empty_map() {
        let result =
            setup_and_render_template("{{ join_node_taints map }}", &json!({"map":{}})).unwrap();
        assert_eq!(result, "")
    }

    #[test]
    fn more_than_one() {
        let result = setup_and_render_template(
            "{{ join_node_taints map }}",
            &json!({"map":{"key1": ["value1:NoSchedule","value1:NoExecute"], "key2": ["value2:NoSchedule"]}}),
        )
        .unwrap();
        assert_eq!(
            result,
            "key1=value1:NoSchedule,key1=value1:NoExecute,key2=value2:NoSchedule"
        )
    }
}

#[cfg(test)]
mod test_default {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
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
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("ecr-prefix", Box::new(ecr_prefix));

        registry.render_template(tmpl, data)
    }

    const ECR_REGISTRY_TESTS: &[(&str, &str)] = &[
        (
            "eu-central-1",
            "328549459982.dkr.ecr.eu-central-1.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
        (
            "af-south-1",
            "917644944286.dkr.ecr.af-south-1.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
        // Test fallback url
        (
            "xy-ztown-1",
            "328549459982.dkr.ecr.us-east-1.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
        (
            "cn-north-1",
            "183470599484.dkr.ecr.cn-north-1.amazonaws.com.cn/bottlerocket-admin:v0.5.1",
        ),
        (
            "ap-south-2",
            "764716012617.dkr.ecr.ap-south-2.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
        (
            "ap-southeast-4",
            "731751899352.dkr.ecr.ap-southeast-4.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
        (
            "eu-central-2",
            "861738308508.dkr.ecr.eu-central-2.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
        (
            "eu-south-2",
            "620625777247.dkr.ecr.eu-south-2.amazonaws.com/bottlerocket-admin:v0.5.1",
        ),
    ];

    const ADMIN_CONTAINER_TEMPLATE: &str =
        "{{ ecr-prefix settings.aws.region }}/bottlerocket-admin:v0.5.1";

    #[test]
    fn registry_urls() {
        for (region_name, expected_url) in ECR_REGISTRY_TESTS {
            let result = setup_and_render_template(
                ADMIN_CONTAINER_TEMPLATE,
                &json!({"settings": {"aws": {"region": *region_name}}}),
            )
            .unwrap();
            assert_eq!(result, *expected_url);
        }
    }
}

#[cfg(test)]
mod test_pause_registry {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("pause-prefix", Box::new(pause_prefix));

        registry.render_template(tmpl, data)
    }

    const CONTAINER_TEMPLATE: &str = "{{ pause-prefix settings.aws.region }}/container:tag";

    const PAUSE_REGISTRY_TESTS: &[(&str, &str)] = &[
        (
            "eu-central-1",
            "602401143452.dkr.ecr.eu-central-1.amazonaws.com/container:tag",
        ),
        (
            "af-south-1",
            "877085696533.dkr.ecr.af-south-1.amazonaws.com/container:tag",
        ),
        (
            "xy-ztown-1",
            "602401143452.dkr.ecr.us-east-1.amazonaws.com/container:tag",
        ),
        (
            "cn-north-1",
            "918309763551.dkr.ecr.cn-north-1.amazonaws.com.cn/container:tag",
        ),
        (
            "ap-southeast-4",
            "491585149902.dkr.ecr.ap-southeast-4.amazonaws.com/container:tag",
        ),
    ];

    #[test]
    fn pause_container_registry() {
        for (region, expected) in PAUSE_REGISTRY_TESTS {
            let result = setup_and_render_template(
                CONTAINER_TEMPLATE,
                &json!({"settings": {"aws": {"region": *region}}}),
            )
            .unwrap();
            assert_eq!(result, *expected);
        }
    }
}

#[cfg(test)]
mod test_tuf_repository {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut repository = Handlebars::new();
        repository.register_helper("tuf-prefix", Box::new(tuf_prefix));
        repository.register_helper("metadata-prefix", Box::new(metadata_prefix));

        repository.render_template(tmpl, data)
    }

    const METADATA_TEMPLATE: &str =
        "{{ tuf-prefix settings.aws.region }}{{ metadata-prefix settings.aws.region }}/2020-07-07/";

    const EXPECTED_URL_AF_SOUTH_1: &str = "https://updates.bottlerocket.aws/2020-07-07/";

    const EXPECTED_URL_XY_ZTOWN_1: &str = "https://updates.bottlerocket.aws/2020-07-07/";

    const EXPECTED_URL_CN_NORTH_1: &str =
        "https://bottlerocket-updates-cn-north-1.s3.dualstack.cn-north-1.amazonaws.com.cn/latest/metadata/2020-07-07/";

    #[test]
    fn url_af_south_1() {
        let result = setup_and_render_template(
            METADATA_TEMPLATE,
            &json!({"settings": {"aws": {"region": "af-south-1"}}}),
        )
        .unwrap();
        assert_eq!(result, EXPECTED_URL_AF_SOUTH_1);
    }

    #[test]
    fn url_fallback() {
        let result = setup_and_render_template(
            METADATA_TEMPLATE,
            &json!({"settings": {"aws": {"region": "xy-ztown-1"}}}),
        )
        .unwrap();
        assert_eq!(result, EXPECTED_URL_XY_ZTOWN_1);
    }

    #[test]
    fn url_cn_north_1() {
        let result = setup_and_render_template(
            METADATA_TEMPLATE,
            &json!({"settings": {"aws": {"region": "cn-north-1"}}}),
        )
        .unwrap();
        assert_eq!(result, EXPECTED_URL_CN_NORTH_1);
    }
}

#[cfg(test)]
mod test_host {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
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

#[cfg(test)]
mod test_goarch {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("goarch", Box::new(goarch));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn good_arches() {
        for (arch, expected) in &[
            ("x86_64", "amd64"),
            ("amd64", "amd64"),
            ("aarch64", "arm64"),
            ("arm64", "arm64"),
        ] {
            let result =
                setup_and_render_template("{{ goarch os.arch }}", &json!({"os": {"arch": arch}}))
                    .unwrap();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn bad_arches() {
        for bad_arch in &["", "amdarm", "x86", "aarch32"] {
            setup_and_render_template("{{ goarch os.arch }}", &json!({ "os": {"arch": bad_arch }}))
                .unwrap_err();
        }
    }
}

#[cfg(test)]
mod test_join_array {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("join_array", Box::new(join_array));

        registry.render_template(tmpl, data)
    }

    const TEMPLATE: &str = r#"{{join_array ", " settings.foo-list}}"#;

    #[test]
    fn join_array_empty() {
        let result =
            setup_and_render_template(TEMPLATE, &json!({"settings": {"foo-list": []}})).unwrap();
        let expected = "";
        assert_eq!(result, expected);
    }

    #[test]
    fn join_array_one_item() {
        let result =
            setup_and_render_template(TEMPLATE, &json!({"settings": {"foo-list": ["a"]}})).unwrap();
        let expected = r#""a""#;
        assert_eq!(result, expected);
    }

    #[test]
    fn join_array_two_items() {
        let result =
            setup_and_render_template(TEMPLATE, &json!({"settings": {"foo-list": ["a", "b"]}}))
                .unwrap();
        let expected = r#""a", "b""#;
        assert_eq!(result, expected);
    }

    #[test]
    fn join_array_two_delimiter() {
        let template = r#"{{join_array "~ " settings.foo-list}}"#;
        let result = setup_and_render_template(
            template,
            &json!({"settings": {"foo-list": ["a", "b", "c"]}}),
        )
        .unwrap();
        let expected = r#""a"~ "b"~ "c""#;
        assert_eq!(result, expected);
    }

    #[test]
    fn join_array_empty_item() {
        let result =
            setup_and_render_template(TEMPLATE, &json!({"settings": {"foo-list": ["a", "", "c"]}}))
                .unwrap();
        let expected = r#""a", "", "c""#;
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod test_kube_reserve_memory {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("kube_reserve_memory", Box::new(kube_reserve_memory));

        registry.render_template(tmpl, data)
    }

    const TEMPLATE: &str = r#""{{kube_reserve_memory  max-pods kube-reserved-memory}}""#;

    #[test]
    fn have_settings_1024_mi() {
        let result = setup_and_render_template(
            TEMPLATE,
            &json!({"max-pods": 29, "kube-reserved-memory": "1024Mi"}),
        )
        .unwrap();
        assert_eq!(result, "\"1024Mi\"");
    }

    #[test]
    fn no_settings_max_pods_0() {
        let result =
            setup_and_render_template(TEMPLATE, &json!({"max-pods": 0, "no-settings": "hi"}))
                .unwrap();
        assert_eq!(result, "\"255Mi\"");
    }

    #[test]
    fn no_settings_max_pods_29() {
        let result =
            setup_and_render_template(TEMPLATE, &json!({"max-pods": 29, "no-settings": "hi"}))
                .unwrap();
        assert_eq!(result, "\"574Mi\"");
    }

    #[test]
    fn max_pods_not_number() {
        setup_and_render_template(
            TEMPLATE,
            &json!({"settings": {"kubernetes": {"max-pods": "ten"}}}),
        )
        .unwrap_err();
    }
}

#[cfg(test)]
mod test_kube_reserve_cpu {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("kube_reserve_cpu", Box::new(kube_reserve_cpu));

        registry.render_template(tmpl, data)
    }

    const TEMPLATE: &str = r#"{{kube_reserve_cpu settings.kubernetes.kube-reserved.cpu}}"#;

    #[test]
    fn kube_reserve_cpu_ok() {
        assert!(setup_and_render_template(TEMPLATE, &json!({"not-the-setting": "hi"})).is_ok());
    }

    #[test]
    fn kube_reserve_cpu_30_m() {
        let result = setup_and_render_template(
            TEMPLATE,
            &json!({"settings": {"kubernetes": {"kube-reserved": {"cpu": "30m"}}}}),
        )
        .unwrap();
        assert_eq!(result, "30m");
    }
}

#[cfg(test)]
mod test_kube_cpu_helper {
    use crate::helpers::kube_cpu_helper;
    use std::collections::HashMap;

    #[test]
    fn kube_cpu_helper_ok() {
        let mut cpu_reserved: HashMap<usize, &str> = HashMap::new();
        cpu_reserved.insert(0, "0m");
        cpu_reserved.insert(1, "60m");
        cpu_reserved.insert(2, "70m");
        cpu_reserved.insert(3, "75m");
        cpu_reserved.insert(4, "80m");
        cpu_reserved.insert(5, "82m");
        cpu_reserved.insert(6, "85m");
        cpu_reserved.insert(47, "187m");
        cpu_reserved.insert(48, "190m");

        for (num_cpus, expected_millicores) in cpu_reserved.into_iter() {
            assert_eq!(kube_cpu_helper(num_cpus).unwrap(), expected_millicores);
        }
    }
}

#[cfg(test)]
mod test_etc_hosts_helpers {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("localhost_aliases", Box::new(localhost_aliases));
        registry.register_helper("etc_hosts_entries", Box::new(etc_hosts_entries));

        registry.render_template(tmpl, data)
    }

    #[test]
    fn test_hostname_resolvable_respects_etc_hosts() {
        assert!(hostname_resolveable(
            "unresolveable.irrelevanthostname.tld",
            Some(
                &serde_json::from_str::<model::modeled_types::EtcHostsEntries>(
                    r#"[["10.0.0.1", ["unresolveable.irrelevanthostname.tld"]]]"#
                )
                .unwrap()
            )
        ));
    }

    #[test]
    fn resolves_to_localhost_renders_entries() {
        // Given a configured hostname that does not resolve in DNS,
        // When /etc/hosts is rendered,
        // Then an additional alias shall be rendered pointing the configured hostname to localhost.
        let result = setup_and_render_template(
            r#"{{localhost_aliases "ipv4" hostname hosts}}"#,
            &json!({"hostname": "localhost"}),
        )
        .unwrap();
        assert_eq!(result, "localhost")
    }

    #[test]
    fn hostname_resolves_to_static_mapping() {
        // Given a configured hostname that does not resolve in DNS
        // and an /etc/hosts configuration that contains that hostname as an alias to an IP address,
        // When /etc/hosts is rendered,
        // Then an additional alias *shall not* be rendered pointing the hostname to localhost.
        let result = setup_and_render_template(
            r#"{{localhost_aliases "ipv4" hostname hosts}}"#,
            &json!({"hostname": "noresolve.bottlerocket.aws", "hosts": [["10.0.0.1", ["irrelevant", "noresolve.bottlerocket.aws"]]]}),
        )
        .unwrap();
        assert_eq!(result, "")
    }

    #[test]
    fn resolvable_hostname_renders_nothing() {
        let result = setup_and_render_template(
            r#"{{localhost_aliases "ipv6" hostname hosts}}"#,
            &json!({"hostname": "amazon.com", "hosts": []}),
        )
        .unwrap();
        assert_eq!(result, "")
    }

    #[test]
    fn static_localhost_mappings_render() {
        let result = setup_and_render_template(
            r#"127.0.0.1 localhost {{localhost_aliases "ipv4" hostname hosts}}"#,
            &json!({"hostname": "", "hosts": [["127.0.0.1", ["test.example.com", "test"]]]}),
        )
        .unwrap();
        assert_eq!(result, "127.0.0.1 localhost test.example.com test")
    }

    #[test]
    fn static_localhost_mappings_low_precedence() {
        let result = setup_and_render_template(
            r#"::1 localhost {{localhost_aliases "ipv6" hostname hosts}}"#,
            &json!({"hostname": "unresolvable.bottlerocket.aws", "hosts": [["::1", ["test.example.com", "test"]]]}),
        )
        .unwrap();
        assert_eq!(
            result,
            "::1 localhost unresolvable.bottlerocket.aws test.example.com test"
        )
    }

    #[test]
    fn hosts_unset_works() {
        let result = setup_and_render_template(
            r#"{{localhost_aliases "ipv4" hostname hosts}}"#,
            &json!({"hostname": "localhost"}),
        )
        .unwrap();
        assert_eq!(result, "localhost")
    }

    #[test]
    fn etc_hosts_entries_works() {
        let result = setup_and_render_template(
            r#"{{etc_hosts_entries hosts}}"#,
            &json!({"hosts": [["10.0.0.1", ["test.example.com", "test"]], ["10.0.0.2", ["test.example.com"]]]}),
        )
        .unwrap();
        assert_eq!(
            result,
            "10.0.0.1 test.example.com test\n10.0.0.2 test.example.com"
        )
    }

    #[test]
    fn etc_hosts_entries_ignores_localhost() {
        let result = setup_and_render_template(
            r#"{{etc_hosts_entries hosts}}"#,
            &json!({"hosts": [["10.0.0.1", ["test.example.com", "test"]], ["127.0.0.1", ["test.example.com"]], ["::1", ["test.example.com"]]]}),
        )
        .unwrap();
        assert_eq!(result, "10.0.0.1 test.example.com test")
    }

    #[test]
    fn etc_hosts_works_with_empty_hosts() {
        let result =
            setup_and_render_template(r#"{{etc_hosts_entries hosts}}"#, &json!({})).unwrap();
        assert_eq!(result, "")
    }
}

#[cfg(test)]
mod test_any_enabled {
    use super::*;
    use handlebars::RenderError;
    use serde::Serialize;
    use serde_json::json;
    use std::collections::BTreeMap;

    // A thin wrapper around the handlebars render_template method that includes
    // setup and registration of helpers
    fn setup_and_render_template<T>(tmpl: &str, data: T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut registry = Handlebars::new();
        registry.register_helper("any_enabled", Box::new(any_enabled));

        let mut input_data = BTreeMap::new();
        input_data.insert("input", data);

        registry.render_template(tmpl, &input_data)
    }

    const TEMPLATE: &str = r#"{{#if (any_enabled input)}}enabled{{else}}disabled{{/if}}"#;

    #[test]
    fn test_any_enabled_with_enabled_elements() {
        let result =
            setup_and_render_template(TEMPLATE, &json!([{"foo": [], "enabled": true}])).unwrap();
        let expected = "enabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_enabled_elements_from_map() {
        let result = setup_and_render_template(
            TEMPLATE,
            &json!({"foo": {"enabled": false}, "bar": {"enabled": true}}),
        )
        .unwrap();
        let expected = "enabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_without_enabled_elements() {
        let result = setup_and_render_template(TEMPLATE, &json!([{"enabled": false}])).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_without_enabled_elements_from_map() {
        let result = setup_and_render_template(
            TEMPLATE,
            &json!({"foo": {"enabled": false}, "bar": {"enabled": false}}),
        )
        .unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_empty_elements() {
        let result = setup_and_render_template(TEMPLATE, &json!([])).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_empty_map() {
        let result = setup_and_render_template(TEMPLATE, &json!({})).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_bool_flag_as_string() {
        // Helper is only expected to work with boolean values
        let result = setup_and_render_template(TEMPLATE, &json!([{"enabled": "true"}])).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_different_type_array() {
        // Validates no errors if a different kind of struct is passed in
        let result = setup_and_render_template(TEMPLATE, &json!([{"name": "fred"}])).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_different_type_map() {
        // Validates no errors if a different kind of struct is passed in
        let result =
            setup_and_render_template(TEMPLATE, &json!({"test": {"name": "fred"}})).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_any_enabled_with_different_type() {
        // Validates no errors when a completely different JSON struct is passed
        let result = setup_and_render_template(TEMPLATE, &json!({"state": "enabled"})).unwrap();
        let expected = "disabled";
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod test_oci_spec {
    use super::{Containerd, Docker};
    use crate::helpers::*;
    use serde_json::json;
    use OciDefaultsResourceLimitType::*;

    #[test]
    fn oci_spec_capabilities_test() {
        let json = json!({
            "kill": true,
            "lease": false,
            "mac-admin": true,
            "mknod": true
        });
        let capabilities = oci_spec_capabilities(&json).unwrap();
        let rendered = Containerd::get_capabilities(capabilities);
        assert_eq!(
            rendered,
            r#""bounding": ["CAP_KILL",
"CAP_MAC_ADMIN",
"CAP_MKNOD"],
"effective": ["CAP_KILL",
"CAP_MAC_ADMIN",
"CAP_MKNOD"],
"permitted": ["CAP_KILL",
"CAP_MAC_ADMIN",
"CAP_MKNOD"]
"#
        );
    }

    fn check_all_rlimits(
        (cap, bottlerocket, hard_limit, soft_limit): (OciDefaultsResourceLimitType, &str, i64, i64),
    ) {
        let json = json!({bottlerocket: {"hard-limit": hard_limit, "soft-limit": soft_limit}});
        let rlimits = oci_spec_resource_limits(&json).unwrap();
        let rendered = Containerd::get_resource_limits(&cap, rlimits.get(&cap).unwrap());
        let result = format!(
            r#"{{ "type": "{}", "hard": {}, "soft": {} }}"#,
            cap.to_linux_string(),
            hard_limit,
            soft_limit,
        );
        assert_eq!(rendered, result);
    }

    #[test]
    fn oci_spec_resource_limits_test() {
        let rlimits = [
            (MaxAddressSpace, "max-address-space", 2, 1),
            (MaxCoreFileSize, "max-core-file-size", 4, 3),
            (MaxCpuTime, "max-cpu-time", 5, 4),
            (MaxDataSize, "max-data-size", 6, 5),
            (MaxFileLocks, "max-file-locks", 7, 6),
            (MaxFileSize, "max-file-size", 8, 7),
            (MaxLockedMemory, "max-locked-memory", 9, 8),
            (MaxMsgqueueSize, "max-msgqueue-size", 10, 9),
            (MaxNicePriority, "max-nice-priority", 11, 10),
            (MaxOpenFiles, "max-open-files", 12, 11),
            (MaxPendingSignals, "max-pending-signals", 17, 15),
            (MaxProcesses, "max-processes", 13, 12),
            (MaxRealtimePriority, "max-realtime-priority", 18, 23),
            (MaxRealtimeTimeout, "max-realtime-timeout", 14, 13),
            (MaxResidentSet, "max-resident-set", 15, 14),
            (MaxStackSize, "max-stack-size", 16, 15),
        ];

        for rlimit in rlimits {
            check_all_rlimits(rlimit);
        }
    }

    #[test]
    fn oci_spec_max_locked_memory_as_unlimited_resource_limit_test() {
        let json = json!({"max-locked-memory": {"hard-limit": "unlimited", "soft-limit": 18}});
        let rlimits = oci_spec_resource_limits(&json).unwrap();
        let rendered = Containerd::get_resource_limits(
            &MaxLockedMemory,
            rlimits.get(&MaxLockedMemory).unwrap(),
        );

        assert_eq!(
            rendered,
            r#"{ "type": "RLIMIT_MEMLOCK", "hard": 18446744073709551615, "soft": 18 }"#
        );
    }

    #[test]
    fn oci_spec_max_locked_memory_as_minus_one_resource_limit_test() {
        let json = json!({"max-locked-memory": {"hard-limit": -1, "soft-limit": 18}});
        let rlimits = oci_spec_resource_limits(&json).unwrap();
        let rendered = Containerd::get_resource_limits(
            &MaxLockedMemory,
            rlimits.get(&MaxLockedMemory).unwrap(),
        );
        assert_eq!(
            rendered,
            r#"{ "type": "RLIMIT_MEMLOCK", "hard": 18446744073709551615, "soft": 18 }"#
        );
    }

    #[test]
    fn oci_spec_capabilities_docker_test() {
        let json = json!({
            "kill": true,
            "lease": false,
            "mac-admin": true,
            "mknod": true
        });
        let capabilities = oci_spec_capabilities(&json).unwrap();
        let rendered = Docker::get_capabilities(capabilities);
        assert_eq!(
            rendered,
            r#"["CAP_KILL",
"CAP_MAC_ADMIN",
"CAP_MKNOD"]"#
        );
    }

    #[test]
    fn oci_spec_resource_limits_test_docker() {
        let json = json!({"max-open-files": {"hard-limit": 1, "soft-limit": 2}});
        let rlimits = oci_spec_resource_limits(&json).unwrap();
        let rendered =
            Docker::get_resource_limits(&MaxOpenFiles, rlimits.get(&MaxOpenFiles).unwrap());
        assert_eq!(
            rendered,
            r#" "nofile":{ "Name": "nofile", "Hard": 1, "Soft": 2 }"#
        );
    }

    #[test]
    fn oci_spec_max_locked_memory_as_unlimited_docker_resource_limit_test() {
        let json = json!({"max-locked-memory": {"hard-limit": "unlimited", "soft-limit": 18}});
        let rlimits = oci_spec_resource_limits(&json).unwrap();
        let rendered =
            Docker::get_resource_limits(&MaxLockedMemory, rlimits.get(&MaxLockedMemory).unwrap());

        assert_eq!(
            rendered,
            r#" "memlock":{ "Name": "memlock", "Hard": -1, "Soft": 18 }"#
        );
    }
}
