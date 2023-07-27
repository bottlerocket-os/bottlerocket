/// The schnauzer library can be used to render file- or string-based templates that contain
/// settings references, e.g. "foo-{{ settings.bar }}", and contains common helper functions for
/// use inside the templates.

#[macro_use]
extern crate log;

pub mod helpers;
pub mod v2;

use handlebars::Handlebars;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::de::DeserializeOwned;
use snafu::ResultExt;
use std::path::Path;

// https://url.spec.whatwg.org/#query-percent-encode-set
const ENCODE_QUERY_CHARS: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');

pub mod error {
    use http::StatusCode;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Error {}ing to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Error {} when {}ing to {}: {}", code, method, uri, response_body))]
        APIResponse {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display(
            "Error deserializing response as JSON from {} to '{}': {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: serde_json::Error,
        },
    }
}
pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

/// Simple helper that extends the API client, abstracting the repeated request logic and
/// deserialization from JSON.
pub async fn get_json<T, P, S1, S2, S3>(
    socket_path: P,
    uri: S1,
    // Query parameter name, query parameter value
    query: Option<(S2, S3)>,
) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
    S1: AsRef<str>,
    S2: AsRef<str>,
    S3: AsRef<str>,
{
    let mut uri = uri.as_ref().to_string();
    // Add (escaped) query parameter, if given
    if let Some((query_param, query_arg)) = query {
        let query_raw = format!("{}={}", query_param.as_ref(), query_arg.as_ref());
        let query_escaped = utf8_percent_encode(&query_raw, ENCODE_QUERY_CHARS);
        uri = format!("{}?{}", uri, query_escaped);
    }

    let method = "GET";
    trace!("{}ing from {}", method, uri);
    let (code, response_body) = apiclient::raw_request(socket_path, &uri, method, None)
        .await
        .context(error::APIRequestSnafu { method, uri: &uri })?;

    if !code.is_success() {
        return error::APIResponseSnafu {
            method,
            uri,
            code,
            response_body,
        }
        .fail();
    }
    trace!("JSON response: {}", response_body);

    serde_json::from_str(&response_body).context(error::ResponseJsonSnafu { method, uri })
}

/// Requests all settings from the API so they can be used as the data source for a handlebars
/// templating call.
pub async fn get_settings<P>(socket_path: P) -> Result<model::Model>
where
    P: AsRef<Path>,
{
    debug!("Querying API for settings data");
    let settings: model::Model =
        get_json(&socket_path, "/", None as Option<(String, String)>).await?;
    trace!("Model values: {:?}", settings);

    Ok(settings)
}

/// Build a handlebars template registry with our common helper functions.
pub fn build_template_registry() -> Result<handlebars::Handlebars<'static>> {
    let mut template_registry = Handlebars::new();
    // Strict mode will panic if a key exists in the template
    // but isn't provided in the data given to the renderer
    template_registry.set_strict_mode(true);

    // Prefer snake case for helper names (we accidentally created a few with kabob case)
    template_registry.register_helper("base64_decode", Box::new(helpers::base64_decode));
    template_registry.register_helper("join_map", Box::new(helpers::join_map));
    template_registry.register_helper("join_node_taints", Box::new(helpers::join_node_taints));
    template_registry.register_helper("default", Box::new(helpers::default));
    template_registry.register_helper("ecr-prefix", Box::new(helpers::ecr_prefix));
    template_registry.register_helper("pause-prefix", Box::new(helpers::pause_prefix));
    template_registry.register_helper("tuf-prefix", Box::new(helpers::tuf_prefix));
    template_registry.register_helper("metadata-prefix", Box::new(helpers::metadata_prefix));
    template_registry.register_helper("host", Box::new(helpers::host));
    template_registry.register_helper("goarch", Box::new(helpers::goarch));
    template_registry.register_helper("join_array", Box::new(helpers::join_array));
    template_registry.register_helper("kube_reserve_cpu", Box::new(helpers::kube_reserve_cpu));
    template_registry.register_helper(
        "kube_reserve_memory",
        Box::new(helpers::kube_reserve_memory),
    );
    template_registry.register_helper("localhost_aliases", Box::new(helpers::localhost_aliases));
    template_registry.register_helper("etc_hosts_entries", Box::new(helpers::etc_hosts_entries));
    template_registry.register_helper("any_enabled", Box::new(helpers::any_enabled));
    template_registry.register_helper("oci_defaults", Box::new(helpers::oci_defaults));

    Ok(template_registry)
}

#[cfg(test)]
mod test {
    use handlebars::Handlebars;
    use serde_json::json;

    #[test]
    fn render_whitespace() {
        let registry = Handlebars::new();
        // Similar to a proxy configuration file whose rendering behavior changed in handlebars 4.
        let tmpl = r###"
{{#if p}}
VAR1={{p}}
VAR2={{p}}
{{/if}}
LIST_UPPER={{#each a}}{{this}},{{/each}}x,y{{#if b}},{{b}}{{/if}}{{#if c}},.{{c}}{{/if}}
list_lower={{#each a}}{{this}},{{/each}}x,y{{#if b}},{{b}}{{/if}}{{#if c}},.{{c}}{{/if}}
        "###;
        let data = json!({"a": ["a1", "a2"], "b": "b1", "c": "c1", "p": "hi"});
        let expected = r###"
VAR1=hi
VAR2=hi
LIST_UPPER=a1,a2,x,y,b1,.c1
list_lower=a1,a2,x,y,b1,.c1
        "###;

        let result = registry.render_template(tmpl, &data).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn render_newline() {
        let registry = Handlebars::new();
        // Another simple check for whitespace behavior changes in handlebars 4.
        let tmpl = r###"{{#if a}}x{{/if}}
y"###;
        let data = json!({ "a": true});
        let expected = "x
y";

        let result = registry.render_template(tmpl, &data).unwrap();
        assert_eq!(result, expected);
    }
}
