//! The server module owns the API surface.  It interfaces with the datastore through the
//! server::controller module.

mod controller;
mod error;
pub use error::Error;

use actix_web::{
    error::ResponseError, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use bottlerocket_release::BottlerocketRelease;
use datastore::{Committed, FilesystemDataStore, Key, Value};
use error::Result;
use fs2::FileExt;
use futures::future;
use http::StatusCode;
use log::info;
use model::{ConfigurationFiles, Model, Services, Settings};
use nix::unistd::{chown, Gid};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{set_permissions, File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;
use std::sync;
use thar_be_updates::status::{UpdateStatus, UPDATE_LOCKFILE};

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// sd_notify helper
fn notify_unix_socket_ready() -> Result<()> {
    if env::var_os("NOTIFY_SOCKET").is_some() {
        ensure!(
            Command::new("systemd-notify")
                .arg("--ready")
                .arg("--no-block")
                .status()
                .context(error::SystemdNotify)?
                .success(),
            error::SystemdNotifyStatus
        );
        env::remove_var("NOTIFY_SOCKET");
    } else {
        info!("NOTIFY_SOCKET not set, not calling systemd-notify");
    }
    Ok(())
}

// Router

/// This is the primary interface of the module.  It defines the server and application that actix
/// spawns for requests.  It creates a shared datastore handle that can be used by handler methods
/// to interface with the controller.
pub async fn serve<P1, P2>(
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
            // In our implementation of ResponseError on our own error type below, we include the
            // error message in the response for debugging purposes.  If actix rejects a request
            // early because it doesn't fit our model, though, it doesn't even get to the
            // ResponseError implementation.  This configuration of the Json extractor allows us to
            // add the error message into the response.
            .app_data(web::Json::<Settings>::configure(|cfg| {
                cfg.error_handler(|err, _req| {
                    HttpResponse::BadRequest().body(err.to_string()).into()
                })
            }))
            // This makes the data store available to API methods merely by having a Data
            // parameter.
            .app_data(shared_datastore.clone())
            // Retrieve the full API model; not all data is writable, so we only support GET.
            .route("/", web::get().to(get_model))
            .service(
                web::scope("/settings")
                    .route("", web::get().to(get_settings))
                    .route("", web::patch().to(patch_settings)),
            )
            .service(
                // Transaction support
                web::scope("/tx")
                    .route("/list", web::get().to(get_transaction_list))
                    .route("", web::get().to(get_transaction))
                    .route("", web::delete().to(delete_transaction))
                    .route("/commit", web::post().to(commit_transaction))
                    .route("/apply", web::post().to(apply_changes))
                    .route(
                        "/commit_and_apply",
                        web::post().to(commit_transaction_and_apply),
                    ),
            )
            .service(web::scope("/os").route("", web::get().to(get_os_info)))
            .service(
                web::scope("/metadata")
                    .route("/affected-services", web::get().to(get_affected_services))
                    .route("/setting-generators", web::get().to(get_setting_generators))
                    .route("/templates", web::get().to(get_templates)),
            )
            .service(web::scope("/services").route("", web::get().to(get_services)))
            .service(
                web::scope("/configuration-files")
                    .route("", web::get().to(get_configuration_files)),
            )
            .service(
                web::scope("/actions")
                    .route("/reboot", web::post().to(reboot))
                    .route("/refresh-updates", web::post().to(refresh_updates))
                    .route("/prepare-update", web::post().to(prepare_update))
                    .route("/activate-update", web::post().to(activate_update))
                    .route("/deactivate-update", web::post().to(deactivate_update)),
            )
            .service(web::scope("/updates").route("/status", web::get().to(get_update_status)))
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
    notify_unix_socket_ready()?;

    http_server.run().await.context(error::ServerStart)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// Handler methods called by the router

/// Returns all data in the API model.
async fn get_model(data: web::Data<SharedDataStore>) -> Result<ModelResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    let settings = Some(controller::get_settings(&*datastore, &Committed::Live)?);
    let services = Some(controller::get_services(&*datastore)?);
    let configuration_files = Some(controller::get_configuration_files(&*datastore)?);
    let os = Some(controller::get_os_info()?);

    let model = Model {
        settings,
        services,
        configuration_files,
        os,
    };
    Ok(ModelResponse(model))
}

// actix-web doesn't support Query for enums, so we use a HashMap and check for the expected keys
// ourselves.
/// Return the live settings from the data store; if 'keys' or 'prefix' are specified in query
/// parameters, return the subset of matching settings.
async fn get_settings(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<SettingsResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    let settings = if let Some(keys_str) = query.get("keys") {
        let keys = comma_separated("keys", keys_str)?;
        controller::get_settings_keys(&*datastore, &keys, &Committed::Live)
    } else if let Some(prefix_str) = query.get("prefix") {
        if prefix_str.is_empty() {
            return error::EmptyInput { input: "prefix" }.fail();
        }
        // Note: the prefix should not include "settings."
        controller::get_settings_prefix(&*datastore, prefix_str, &Committed::Live)
    } else {
        controller::get_settings(&*datastore, &Committed::Live)
    }?;

    Ok(SettingsResponse(settings))
}

/// Apply the requested settings to the pending data store
async fn patch_settings(
    settings: web::Json<Settings>,
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<HttpResponse> {
    let transaction = transaction_name(&query);
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;
    controller::set_settings(&mut *datastore, &settings, transaction)?;
    Ok(HttpResponse::NoContent().finish()) // 204
}

async fn get_transaction_list(data: web::Data<SharedDataStore>) -> Result<TransactionListResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
    let data = controller::list_transactions(&*datastore)?;
    Ok(TransactionListResponse(data))
}

/// Get any pending settings in the given transaction, or the "default" transaction if unspecified.
async fn get_transaction(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<SettingsResponse> {
    let transaction = transaction_name(&query);
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
    let data = controller::get_transaction(&*datastore, transaction)?;
    Ok(SettingsResponse(data))
}

/// Delete the given transaction, or the "default" transaction if unspecified.
async fn delete_transaction(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ChangedKeysResponse> {
    let transaction = transaction_name(&query);
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;
    let deleted = controller::delete_transaction(&mut *datastore, transaction)?;
    Ok(ChangedKeysResponse(deleted))
}

/// Save settings changes from the given transaction, or the "default" transaction if unspecified,
/// to the live data store.  Returns the list of changed keys.
async fn commit_transaction(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ChangedKeysResponse> {
    let transaction = transaction_name(&query);
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;

    let changes = controller::commit_transaction(&mut *datastore, transaction)?;

    if changes.is_empty() {
        return error::CommitWithNoPending.fail();
    }

    Ok(ChangedKeysResponse(changes))
}

/// Starts settings appliers for any changes that have been committed to the data store.  This
/// updates config files, runs restart commands, etc.
async fn apply_changes(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse> {
    if let Some(keys_str) = query.get("keys") {
        let keys = comma_separated("keys", keys_str)?;
        controller::apply_changes(Some(&keys))?;
    } else {
        controller::apply_changes(None as Option<&HashSet<&str>>)?;
    }

    Ok(HttpResponse::NoContent().json(()))
}

/// Usually you want to apply settings changes you've committed, so this is a convenience method to
/// perform both a commit and an apply.  Commits the given transaction, or the "default"
/// transaction if unspecified.
async fn commit_transaction_and_apply(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ChangedKeysResponse> {
    let transaction = transaction_name(&query);
    let mut datastore = data.ds.write().ok().context(error::DataStoreLock)?;

    let changes = controller::commit_transaction(&mut *datastore, transaction)?;

    if changes.is_empty() {
        return error::CommitWithNoPending.fail();
    }

    let key_names = changes.iter().map(|k| k.name()).collect();
    controller::apply_changes(Some(&key_names))?;

    Ok(ChangedKeysResponse(changes))
}

async fn get_os_info() -> Result<BottlerocketReleaseResponse> {
    Ok(BottlerocketReleaseResponse(controller::get_os_info()?))
}

/// Get the affected services for a list of data keys
async fn get_affected_services(
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
async fn get_setting_generators(data: web::Data<SharedDataStore>) -> Result<MetadataResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
    let resp = controller::get_metadata_for_all_data_keys(&*datastore, "setting-generator")?;
    Ok(MetadataResponse(resp))
}

/// Get the template metadata for a list of data keys
async fn get_templates(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<MetadataResponse> {
    if let Some(keys_str) = query.get("keys") {
        let data_keys = comma_separated("keys", keys_str)?;
        let datastore = data.ds.read().ok().context(error::DataStoreLock)?;
        let resp = controller::get_metadata_for_data_keys(&*datastore, "template", &data_keys)?;

        Ok(MetadataResponse(resp))
    } else {
        return error::MissingInput { input: "keys" }.fail();
    }
}

/// Get all services, or if 'names' is specified, services with those names
async fn get_services(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ServicesResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    let resp = if let Some(names_str) = query.get("names") {
        let names = comma_separated("names", names_str)?;
        controller::get_services_names(&*datastore, &names, &Committed::Live)
    } else {
        controller::get_services(&*datastore)
    }?;

    Ok(ServicesResponse(resp))
}

/// Get all configuration files, or if 'names' is specified, configuration files with those names
async fn get_configuration_files(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<SharedDataStore>,
) -> Result<ConfigurationFilesResponse> {
    let datastore = data.ds.read().ok().context(error::DataStoreLock)?;

    let resp = if let Some(names_str) = query.get("names") {
        let names = comma_separated("names", names_str)?;
        controller::get_configuration_files_names(&*datastore, &names, &Committed::Live)
    } else {
        controller::get_configuration_files(&*datastore)
    }?;

    Ok(ConfigurationFilesResponse(resp))
}

/// Get the update status from 'thar-be-updates'
async fn get_update_status() -> Result<UpdateStatusResponse> {
    let lockfile = File::create(UPDATE_LOCKFILE).context(error::UpdateLockOpen)?;
    lockfile.try_lock_shared().context(error::UpdateShareLock)?;
    let result = thar_be_updates::status::get_update_status(&lockfile);
    match result {
        Ok(update_status) => Ok(UpdateStatusResponse(update_status)),
        Err(e) => match e {
            thar_be_updates::error::Error::NoStatusFile { .. } => {
                error::UninitializedUpdateStatus.fail()
            }
            _ => error::UpdateError.fail(),
        },
    }
}

/// Refreshes the list of updates and checks if an update is available matching the configured version lock
async fn refresh_updates() -> Result<HttpResponse> {
    controller::dispatch_update_command(&["refresh"])
}

/// Prepares update by downloading the images to the staging partition set
async fn prepare_update() -> Result<HttpResponse> {
    controller::dispatch_update_command(&["prepare"])
}

/// "Activates" an already staged update by bumping the priority bits on the staging partition set
async fn activate_update() -> Result<HttpResponse> {
    controller::dispatch_update_command(&["activate"])
}

/// "Deactivates" an already activated update by rolling back actions done by 'activate-update'
async fn deactivate_update() -> Result<HttpResponse> {
    controller::dispatch_update_command(&["deactivate"])
}

/// Reboots the machine
async fn reboot() -> Result<HttpResponse> {
    debug!("Rebooting now");
    let output = Command::new("/sbin/shutdown")
        .arg("-r")
        .arg("now")
        .output()
        .context(error::Shutdown)?;
    ensure!(
        output.status.success(),
        error::Reboot {
            exit_code: match output.status.code() {
                Some(code) => code,
                None => output.status.signal().unwrap_or(1),
            },
            stderr: String::from_utf8_lossy(&output.stderr),
        }
    );
    Ok(HttpResponse::NoContent().finish())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// Helpers for handler methods called by the router

fn comma_separated<'a>(key_name: &'static str, input: &'a str) -> Result<HashSet<&'a str>> {
    if input.is_empty() {
        return error::EmptyInput { input: key_name }.fail();
    }
    Ok(input.split(',').collect())
}

fn transaction_name(query: &web::Query<HashMap<String, String>>) -> &str {
    if let Some(name_str) = query.get("tx") {
        name_str
    } else {
        "default"
    }
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
            UpdateDoesNotExist { .. } => HttpResponse::NotFound(),
            NoStagedImage { .. } => HttpResponse::NotFound(),
            UninitializedUpdateStatus { .. } => HttpResponse::NotFound(),

            // 422 Unprocessable Entity
            CommitWithNoPending => HttpResponse::UnprocessableEntity(),

            // 423 Locked
            UpdateShareLock { .. } => HttpResponse::build(StatusCode::LOCKED),
            UpdateLockHeld { .. } => HttpResponse::build(StatusCode::LOCKED),

            // 409 Conflict
            DisallowCommand { .. } => HttpResponse::Conflict(),

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
            ReleaseData { .. } => HttpResponse::InternalServerError(),
            Shutdown { .. } => HttpResponse::InternalServerError(),
            Reboot { .. } => HttpResponse::InternalServerError(),
            UpdateDispatcher { .. } => HttpResponse::InternalServerError(),
            UpdateError { .. } => HttpResponse::InternalServerError(),
            UpdateStatusParse { .. } => HttpResponse::InternalServerError(),
            UpdateInfoParse { .. } => HttpResponse::InternalServerError(),
            UpdateLockOpen { .. } => HttpResponse::InternalServerError(),
        }
        // Include the error message in the response, and for all error types.  The Bottlerocket
        // API is only exposed locally, and only on the host filesystem and to authorized
        // containers, so we're not worried about exposing error details.
        .body(self.to_string())
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
            type Future = future::Ready<Result<HttpResponse>>;

            fn respond_to($self, _req: &HttpRequest) -> Self::Future {
                let body = match serde_json::to_string(&$serialize_expr) {
                    Ok(s) => s,
                    Err(e) => return future::ready(Err(e).context(error::ResponseSerialization)),
                };
                future::ready(Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)))
            }
        }
    )
}

/// This lets us respond from our handler methods with a Model (or Result<Model>)
struct ModelResponse(Model);
impl_responder_for!(ModelResponse, self, self.0);

/// This lets us respond from our handler methods with a Settings (or Result<Settings>)
struct SettingsResponse(Settings);
impl_responder_for!(SettingsResponse, self, self.0);

/// This lets us respond from our handler methods with a BottlerocketRelease (or Result<BottlerocketRelease>)
struct BottlerocketReleaseResponse(BottlerocketRelease);
impl_responder_for!(BottlerocketReleaseResponse, self, self.0);

/// This lets us respond from our handler methods with a HashMap (or Result<HashMap>) for metadata
struct MetadataResponse(HashMap<String, Value>);
impl_responder_for!(MetadataResponse, self, self.0);

/// This lets us respond from our handler methods with a Services (or Result<Services>)
struct ServicesResponse(Services);
impl_responder_for!(ServicesResponse, self, self.0);

/// This lets us respond from our handler methods with a UpdateStatus (or Result<UpdateStatus>)
struct UpdateStatusResponse(UpdateStatus);
impl_responder_for!(UpdateStatusResponse, self, self.0);

/// This lets us respond from our handler methods with a ConfigurationFiles (or
/// Result<ConfigurationFiles>)
struct ConfigurationFilesResponse(ConfigurationFiles);
impl_responder_for!(ConfigurationFilesResponse, self, self.0);

struct ChangedKeysResponse(HashSet<Key>);
impl_responder_for!(ChangedKeysResponse, self, self.0);

struct TransactionListResponse(HashSet<String>);
impl_responder_for!(TransactionListResponse, self, self.0);
