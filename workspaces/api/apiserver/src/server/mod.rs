//! The server module owns the API surface and interfaces with the datastore.

mod controller;
#[macro_use]
mod rouille_ext;
mod router;

pub use router::handle_request;
