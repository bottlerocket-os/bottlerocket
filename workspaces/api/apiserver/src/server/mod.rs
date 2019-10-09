//! The server module owns the API surface.  It interfaces with the datastore through the
//! server::controller module.

mod controller;
mod error;
pub use error::Error;

use actix_web::{error::ResponseError, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use snafu::{OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync;

use crate::datastore::{Committed, FilesystemDataStore, Key, Value};
use crate::model::{ConfigurationFiles, Services, Settings};
use error::Result;

use nix::unistd::{chown, Gid};
use std::fs::set_permissions;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// sd_notify helper
#[cfg(feature = "sd_notify")]
fn notify_unix_socket_ready() -> Result<()> {
    use snafu::ensure;
    use systemd::daemon;
    let daemon_notify_success = daemon::notify(
        true,
        [
            (daemon::STATE_READY, "1"),
            (daemon::STATE_STATUS, "Thar API Server: socket ready"),
        ]
        .into_iter(),
    )
    .context(error::SystemdNotify)?;
    ensure!(daemon_notify_success, error::SystemdNotifyStatus);
    Ok(())
}

// Router

/// This is the primary interface of the module.  It defines the server and application that actix
/// spawns for requests.  It creates a shared datastore handle that can be used by handler methods
/// to interface with the controller.
pub fn serve<P1, P2>(
    socket_path: P1,
    datastore_path: P2,
    threads: usize,
    socket_gid: Option<Gid>,
) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let shared_datastore = web::Data::new(SharedDataStore {
        ds: sync::RwLock::new(FilesystemDataStore::new(datastore_path)),
    });

    let http_server = HttpServer::new(move || {
        App::new()
            .register_data(shared_datastore.clone())
            .service(
                web::scope("/settings")
                    .route("", web::get().to(get_settings))
                    .route("", web::patch().to(patch_settings))
                    .route("/pending", web::get().to(get_pending_settings))
                    .route("/pending", web::delete().to(delete_pending_settings))
                    .route("/commit", web::post().to(commit_settings))
                    .route("/apply", web::post().to(apply_settings))
                    .route(
                        "/commit_and_apply",
                        web::post().to(commit_and_apply_settings),
                    ),
            )
            .service(
                web::scope("/metadata")
                    .route("/affected-services", web::get().to(get_affected_services))
                    .route("/setting-generators", web::get().to(get_setting_generators)),
            )
            .service(web::scope("/services").route("", web::get().to(get_services)))
            .service(
                web::scope("/configuration-files")
                    .route("", web::get().to(get_configuration_files)),
            )
    })
    .workers(threads)
    .bind_uds(socket_path.as_ref())
    .context(error::BindSocket {
        path: socket_path.as_ref(),
    })?;

    // If the socket needs to be chowned to a group to grant further access, that can be passed
    // as a paramter.
    if let Some(gid) = socket_gid {
        chown(socket_path.as_ref(), None, Some(gid)).context(error::SetGroup { gid })?;
    }

    let mode = 0o0660;
    let perms = Permissions::from_mode(mode);
    set_permissions(socket_path.as_ref(), perms).context(error::SetPermissions { mode })?;

    // Notify system manager the UNIX socket has been initialized, so other service units can proceed
    #[cfg(feature = "sd_notify")]
    {
        notify_unix_socket_ready()?;
    }

    http_server.run().context(error::ServerStart)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// Handler methods called by the router

// actix-web doesn't support Query for enums, so we use a HashMap and check for the expected keys
// ourselves.
/// Return the live settings from the data store; if 'keys' or 'prefix' are specified in query
/// parameters, return the subset of matching settings.
fn get_settings(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<Settings> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    if let Some(keys_str) = query.get("keys") {
        let keys = comma_separated("keys", keys_str)?;
        controller::get_settings_keys(&*datastore, &keys, Committed::Live)
    } else if let Some(prefix_str) = query.get("prefix") {
        if prefix_str.is_empty() {
            return error::EmptyInput { input: "prefix" }.fail();
        }
        // Note: the prefix should not include "settings."
        controller::get_settings_prefix(&*datastore, prefix_str, Committed::Live)
    } else {
        controller::get_settings(&*datastore, Committed::Live)
    }
}

/// Apply the requested settings to the pending data store
fn patch_settings(
    settings: web::Json<Settings>,
    data: web::Data<SharedDataStore>,
) -> Result<HttpResponse> {
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;
    controller::set_settings(&mut *datastore, &settings)?;
    Ok(HttpResponse::NoContent().finish()) // 204
}

/// Return any settings that have been received but not committed
fn get_pending_settings(data: web::Data<SharedDataStore>) -> Result<Settings> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
    controller::get_pending_settings(&*datastore)
}

/// Delete any settings that have been received but not committed
fn delete_pending_settings(data: web::Data<SharedDataStore>) -> Result<ChangedKeysResponse> {
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;
    let deleted = controller::delete_pending_settings(&mut *datastore)?;
    Ok(ChangedKeysResponse(deleted))
}

/// Save settings changes to the main data store and kick off appliers.
fn commit_settings(data: web::Data<SharedDataStore>) -> Result<ChangedKeysResponse> {
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;

    let changes = controller::commit(&mut *datastore)?;

    if changes.is_empty() {
        return error::CommitWithNoPending.fail();
    }

    Ok(ChangedKeysResponse(changes))
}

/// Save settings changes to the main data store and kick off appliers.
fn apply_settings(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse> {
    if let Some(keys_str) = query.get("keys") {
        let keys = comma_separated("keys", keys_str)?;
        controller::apply_changes(Some(&keys))?;
    } else {
        controller::apply_changes(None as Option<&HashSet<&str>>)?;
    }

    Ok(HttpResponse::NoContent().json(()))
}

/// Save settings changes to the main data store and kick off appliers.
fn commit_and_apply_settings(data: web::Data<SharedDataStore>) -> Result<ChangedKeysResponse> {
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;

    let changes = controller::commit(&mut *datastore)?;

    if changes.is_empty() {
        return error::CommitWithNoPending.fail();
    }

    controller::apply_changes(Some(&changes))?;

    Ok(ChangedKeysResponse(changes))
}

/// Get the affected services for a list of data keys
fn get_affected_services(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<MetadataResponse> {
    if let Some(keys_str) = query.get("keys") {
        let data_keys = comma_separated("keys", keys_str)?;
        let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
        let resp =
            controller::get_metadata_for_data_keys(&*datastore, "affected-services", &data_keys)?;

        Ok(MetadataResponse(resp))
    } else {
        return error::MissingInput { input: "keys" }.fail();
    }
}

/// Get all settings that have setting-generator metadata
fn get_setting_generators(data: web::Data<SharedDataStore>) -> Result<MetadataResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
    let resp = controller::get_metadata_for_all_data_keys(&*datastore, "setting-generator")?;
    Ok(MetadataResponse(resp))
}

/// Get all services, or if 'names' is specified, services with those names
fn get_services(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ServicesResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    let resp = if let Some(names_str) = query.get("names") {
        let names = comma_separated("names", names_str)?;
        controller::get_services_names(&*datastore, &names, Committed::Live)
    } else {
        controller::get_services(&*datastore)
    }?;

    Ok(ServicesResponse(resp))
}

/// Get all configuration files, or if 'names' is specified, configuration files with those names
fn get_configuration_files(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ConfigurationFilesResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    let resp = if let Some(names_str) = query.get("names") {
        let names = comma_separated("names", names_str)?;
        controller::get_configuration_files_names(&*datastore, &names, Committed::Live)
    } else {
        controller::get_configuration_files(&*datastore)
    }?;

    Ok(ConfigurationFilesResponse(resp))
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// Helpers for handler methods called by the router

fn comma_separated<'a>(key_name: &'static str, input: &'a str) -> Result<HashSet<&'a str>> {
    if input.is_empty() {
        return error::EmptyInput { input: key_name }.fail();
    }
    Ok(input.split(',').collect())
}

// Can also override `render_response` if we want to change headers, content type, etc.
impl ResponseError for error::Error {
    /// Maps our error types to the HTTP error code they should return.
    fn error_response(&self) -> HttpResponse {
        use error::Error::*;
        match self {
            // 400 Bad Request
            MissingInput { .. } => HttpResponse::BadRequest(),
            EmptyInput { .. } => HttpResponse::BadRequest(),
            NewKey { .. } => HttpResponse::BadRequest(),

            // 404 Not Found
            MissingData { .. } => HttpResponse::NotFound(),
            ListKeys { .. } => HttpResponse::NotFound(),

            // 422 Unprocessable Entity
            CommitWithNoPending => HttpResponse::UnprocessableEntity(),

            // 500 Internal Server Error
            DataStoreLock => HttpResponse::InternalServerError(),
            ResponseSerialization { .. } => HttpResponse::InternalServerError(),
            BindSocket { .. } => HttpResponse::InternalServerError(),
            ServerStart { .. } => HttpResponse::InternalServerError(),
            ListedKeyNotPresent { .. } => HttpResponse::InternalServerError(),
            DataStore { .. } => HttpResponse::InternalServerError(),
            Deserialization { .. } => HttpResponse::InternalServerError(),
            DataStoreSerialization { .. } => HttpResponse::InternalServerError(),
            CommandSerialization { .. } => HttpResponse::InternalServerError(),
            InvalidMetadata { .. } => HttpResponse::InternalServerError(),
            ConfigApplierStart { .. } => HttpResponse::InternalServerError(),
            ConfigApplierStdin {} => HttpResponse::InternalServerError(),
            ConfigApplierWrite { .. } => HttpResponse::InternalServerError(),
            SystemdNotify { .. } => HttpResponse::InternalServerError(),
            SystemdNotifyStatus {} => HttpResponse::InternalServerError(),
            SetPermissions { .. } => HttpResponse::InternalServerError(),
            SetGroup { .. } => HttpResponse::InternalServerError(),
        }
        .finish()
    }
}

struct SharedDataStore {
    ds: sync::RwLock<FilesystemDataStore>,
}

/// Helper macro for implementing the actix-web Responder trait for a type.
/// $for: the type for which we implement Responder.
/// $self: just pass "self"  (macro hygiene requires this)
/// $serialize_expr: the thing to serialize for a response; this is just "self" again if $for
///    implements Serialize, or is "self.0" for a newtype over something implementing Serialize
macro_rules! impl_responder_for {
    ($for:ident, $self:ident, $serialize_expr:expr) => (
        impl Responder for $for {
            type Error = error::Error;
            type Future = Result<HttpResponse>;

            fn respond_to($self, _req: &HttpRequest) -> Self::Future {
                let body = serde_json::to_string(&$serialize_expr).context(error::ResponseSerialization)?;
                Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body))
            }
        }
    )
}

// This lets us respond from our handler methods with a Settings (or Result<Settings>)
impl_responder_for!(Settings, self, self);

/// This lets us respond from our handler methods with a HashMap (or Result<HashMap>) for metadata
struct MetadataResponse(HashMap<String, Value>);
impl_responder_for!(MetadataResponse, self, self.0);

/// This lets us respond from our handler methods with a Services (or Result<Services>)
struct ServicesResponse(Services);
impl_responder_for!(ServicesResponse, self, self.0);

/// This lets us respond from our handler methods with a ConfigurationFiles (or
/// Result<ConfigurationFiles>)
struct ConfigurationFilesResponse(ConfigurationFiles);
impl_responder_for!(ConfigurationFilesResponse, self, self.0);

struct ChangedKeysResponse(HashSet<Key>);
impl_responder_for!(ChangedKeysResponse, self, self.0);
