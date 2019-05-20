//! This module defines the accepted API methods and routes them to appropriate controller code.

use rouille::{Request, Response};
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use crate::datastore::{Committed, FilesystemDataStore};
use crate::model::Settings;
use crate::server::controller::*;
use crate::server::{Result, ServerError};

/// Helper to get a required parameter from the Request.
fn get_param(request: &Request, name: &str) -> Result<String> {
    let res = request
        .get_param(name)
        .ok_or_else(|| ServerError::MissingInput(name.to_string()));
    match res {
        Ok(ref s) if s.is_empty() => Err(ServerError::MissingInput(name.to_string())),
        x => x,
    }
}

/// Helper to get the body of a Request.  Returns Err if we couldn't read the body; returns
/// Ok(None) if we could read it and it was empty.  (If you require a body, call expect_body on
/// the Option.)
fn get_body(request: &Request) -> Result<Option<String>> {
    let mut body_str = String::new();
    let mut request_body = request
        .data()
        .ok_or_else(|| ServerError::MissingInput("request body".to_string()))?;
    request_body.read_to_string(&mut body_str)?;

    if body_str.is_empty() {
        return Ok(None);
    }

    Ok(Some(body_str))
}

/// Helper to make an error when a required body is empty.
fn expect_body(maybe_body: Option<String>) -> Result<String> {
    maybe_body.ok_or_else(|| ServerError::InvalidInput("Empty body".to_string()))
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
            if let Ok(keys_str) = get_param(&request, "keys") {
                let keys: HashSet<&str> = keys_str.split(',').collect();
                try_or!(500, get_settings_keys(&datastore, &keys, Committed::Live)
                             .map(|ref s| Response::json(s)))
            } else if let Ok(prefix_str) = get_param(&request, "prefix") {
                // Note: the prefix should not include "settings."
                try_or!(500, get_settings_prefix(&datastore, prefix_str, Committed::Live)
                             .map(|ref s| Response::json(s)))
            } else {
                try_or!(500, get_settings(&datastore, Committed::Live)
                             .map(|ref s| Response::json(s)))
            }
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
            try_or!(500, get_settings(&datastore, Committed::Pending)
                         .map(|ref s| Response::json(s)))
        },

        // Save settings changes to main data store and kick off appliers
        (POST) (/settings/commit) => {
            let changes = try_or!(500, commit(&mut datastore));
            try_or!(500, apply_changes(&changes).map(|_| Response::empty_204()))
        },

        // Get the value of a metadata key for a list of data keys
        (GET) (/metadata/affected-services) => {
            let data_keys_str = try_or!(400, get_param(&request, "keys"));
            let data_keys: HashSet<&str> = data_keys_str.split(',').collect();
            try_or!(500, get_metadata(&datastore, "affected-services", &data_keys)
                         .map(|ref s| Response::json(s)))
        },

        // Services
        (GET) (/services) => {
            if let Ok(names_str) = get_param(&request, "names") {
                let names: HashSet<&str> = names_str.split(',').collect();
                try_or!(500, get_services_names(&datastore, &names, Committed::Live)
                             .map(|ref s| Response::json(s)))
            } else {
                try_or!(500, get_services(&datastore)
                             .map(|ref s| Response::json(s)))
            }
        },

        // Configuration files
        (GET) (/configuration-files) => {
            if let Ok(names_str) = get_param(&request, "names") {
                let names: HashSet<&str> = names_str.split(',').collect();
                try_or!(500, get_configuration_files_names(&datastore, &names, Committed::Live)
                             .map(|ref s| Response::json(s)))
            } else {
                try_or!(500, get_configuration_files(&datastore)
                             .map(|ref s| Response::json(s)))
            }
        },

        _ => Response::empty_404()
    )
}
