/*!
# Background

This library provides an API server intended for use in an OS that is primarily accessible through the API.
It's intended to be the primary way to read and modify OS settings, to update services based on those settings, and more generally to learn about and change the state of the system.

# Design

## API

We present an HTTP interface to the configurable settings and other state.

Settings, in particular, are stored as pending until the commit API is called.
Pending settings can be retrieved to see what will change.
Upon commit, pending settings are made live, and an external settings applier tool is called to apply the changes to the system and restart services as necessary.

Requests are directed by `server::router`.
`server::controller` maps requests into our data model.

## Model

The API is driven by a data model (similar to a schema) defined in Rust.
See the 'model' module.
All input is deserialized into model types, and all output is serialized from model types, so we can be more confident that data is as we expect.

The model describes system settings, services using those settings, and configuration files used by those services.
It also has a more general structure for metadata.
Metadata entries can be stored for any data field in the model.

## Datastore

Data from the model is stored in a key/value datastore.
Keys are dotted strings like "settings.service.abc".
This naturally implies some grouping and hierarchy of the data, corresponding to the model.

## Serialization and deserialization

The `datastore::serialization` module provides code to serialize Rust types into a mapping of datastore-acceptable keys (a.b.c) and values.

The `datastore::deserialization` module provides code to deserialize datastore-acceptable keys (a.b.c) and values into Rust types.

# Current limitations

* There's no datastore locking, so the server is limited to one thread.
* There's no generated client to make HTTP requests easier.
* There's no support for rolling back commits.
* There are no metrics.
* The keys (schema) have no versioning.
* `datastore::serialization` can't handle complex types under lists; it assumes lists can be serialized as scalars.

# Example usage

You can start the API server from the development workspace with a command like:

`cargo run -- --datastore-path /tmp/thar/be/data`

(Add a few -v options to increase logging.)

Then, from another shell, you can query or modify data.
Here are some examples:

* `curl 'localhost:4242/settings'`
* `curl -X PATCH 'localhost:4242/settings' -d '{"settings": {"timezone": "NewLosAngeles"}}';`
* `curl 'localhost:4242/settings/pending'`
* `curl -X POST 'localhost:4242/settings/commit`
* `curl 'localhost:4242/services?names=hostname`
*/

#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rouille;

pub mod datastore;
pub mod model;
pub(crate) mod server;

pub use server::handle_request;

use std::io;

/// IoErrorDetail gives us more control over the user-visible message when an io error occurs.
/// For example, instead of just "No such file or directory", we can add a prefix that describes
/// the key that we couldn't retrieve.
#[derive(Debug)]
pub struct IoErrorDetail {
    msg: String,
}

impl IoErrorDetail {
    fn new(msg: String, err: io::Error) -> IoErrorDetail {
        let msg = if msg.is_empty() {
            format!("{}", err)
        } else {
            format!("{} - {}", msg, err)
        };
        IoErrorDetail { msg }
    }

    fn as_str(&self) -> &str {
        &self.msg
    }
}
