//! This module defines the accepted API methods and routes them to appropriate controller code.

use rouille::{Request, Response};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use super::controller::*;
use crate::datastore::{Committed, FilesystemDataStore};
use crate::model::Settings;

mod error {
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Missing required input '{}'", input))]
        MissingInput { input: String },

        #[snafu(display("Input '{}' cannot be empty", input))]
        EmptyInput { input: String },

        #[snafu(display("Error reading from request: {}", source))]
        RequestRead { source: io::Error },
    }
}
type Result<T> = std::result::Result<T, error::Error>;

/// Helper to get a required input from the Request.
fn get_param(request: &Request, name: &str) -> Result<String> {
    let res = request
        .get_param(name)
        .context(error::MissingInput { input: name })?;

    ensure!(!res.is_empty(), error::EmptyInput { input: name });
    Ok(res)
}

/// Helper to get the body of a Request.  Returns Err if we couldn't read the body; returns
/// Ok(None) if we could read it and it was empty.  (If you require a body, call expect_body on
/// the Option.)
fn get_body(request: &Request) -> Result<Option<String>> {
    let mut body_str = String::new();
    let mut request_body = request.data().context(error::MissingInput {
        input: "request body",
    })?;
    request_body
        .read_to_string(&mut body_str)
        .context(error::RequestRead)?;

    if body_str.is_empty() {
        return Ok(None);
    }

    Ok(Some(body_str))
}

/// Helper to make an error when a required body is empty.
fn expect_body(maybe_body: Option<String>) -> Result<String> {
    maybe_body.context(error::MissingInput {
        input: "request body",
    })
}

/// This is the primary interface of the module, intended to be spawned by rouille when it
/// receives a request.  It creates a datastore handle and uses that to interface with the
/// controller.
#[allow(clippy::cyclomatic_complexity)]
pub fn handle_request<P: AsRef<Path>>(request: &Request, datastore_path: P) -> Response {
    debug!(
        "Starting handle_request for {} {} from {}",
        request.method(),
        request.url(),
        request.remote_addr()
    );

    let mut datastore = FilesystemDataStore::new(datastore_path);
    router!(request,

        // Bulk settings
        (GET) (/settings) => {
            let try_settings = if let Ok(keys_str) = get_param(&request, "keys") {
                let keys: HashSet<&str> = keys_str.split(',').collect();
                get_settings_keys(&datastore, &keys, Committed::Live)
            } else if let Ok(prefix_str) = get_param(&request, "prefix") {
                // Note: the prefix should not include "settings."
                get_settings_prefix(&datastore, prefix_str, Committed::Live)
            } else {
                get_settings(&datastore, Committed::Live)
            };
            try_or!(500, try_settings.map(|ref s| Response::json(s)))
        },
        (PATCH) (/settings) => {
            let maybe_body = try_or!(500, get_body(&request));
            let body = try_or!(400, expect_body(maybe_body));
            let input: Settings = try_or!(400, settings_input(&body));

            try_or!(500, set_settings(&mut datastore, &input)
                         .map(|_| Response::empty_204()))
        },

        // Special subsets of settings
        (GET) (/settings/pending) => {
            try_or!(500, get_pending_settings(&datastore)
                         .map(|ref s| Response::json(s)))
        },

        // Save settings changes to main data store and kick off appliers
        (POST) (/settings/commit) => {
            let changes = try_or!(500, commit(&mut datastore));
            try_or!(500, apply_changes(&changes).map(|_| Response::empty_204()))
        },

        // Get the affected services for a list of data keys
        (GET) (/metadata/affected-services) => {
            let data_keys_str = try_or!(400, get_param(&request, "keys"));
            let data_keys: HashSet<&str> = data_keys_str.split(',').collect();
            try_or!(500, get_metadata_for_data_keys(&datastore, "affected-services", &data_keys)
                         .map(|ref s| Response::json(s)))
        },
        // Get all settings that have setting-generator metadata
        (GET) (/metadata/setting-generators) => {
            try_or!(500, get_metadata_for_all_data_keys(&datastore, "setting-generator")
                         .map(|ref s| Response::json(s)))
        },


        // Services
        (GET) (/services) => {
            let try_services = if let Ok(names_str) = get_param(&request, "names") {
                let names: HashSet<&str> = names_str.split(',').collect();
                get_services_names(&datastore, &names, Committed::Live)
            } else {
                get_services(&datastore)
            };
            try_or!(500, try_services.map(|ref s| Response::json(s)))
        },

        // Configuration files
        (GET) (/configuration-files) => {
            let try_conf = if let Ok(names_str) = get_param(&request, "names") {
                let names: HashSet<&str> = names_str.split(',').collect();
                get_configuration_files_names(&datastore, &names, Committed::Live)
            } else {
                get_configuration_files(&datastore)
            };
            try_or!(500, try_conf.map(|ref s| Response::json(s)))
        },

        _ => Response::empty_404()
    )
}
