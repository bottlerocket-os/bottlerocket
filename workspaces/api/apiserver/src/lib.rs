/*!
# Background

This library provides an API server intended for use in an OS that is primarily accessible through the API.
It's intended to be the primary way to read and modify OS settings, to update services based on those settings, and more generally to learn about and change the state of the system.

The server listens to HTTP requests on a Unix-domain socket.
There is no built-in authentication - local access to the socket should be limited to processes and containers that should be able to configure the system.
Remote access should only be allowed through an authenticated control channel such as SSH or SSM.

# Design

## API

We present an HTTP interface to configurable settings and other state.
The interface is documented in [OpenAPI format](https://swagger.io/docs/specification/about/) in [openapi.yaml](openapi.yaml).

The Settings APIs are particularly important.
You can GET settings from the `/settings` endpoint.
You can also PATCH changes to the `/settings` endpoint.
Settings are stored as pending until a commit API is called.
Pending settings can be retrieved from `/settings/pending` to see what will change.

Upon making a `commit` API call, pending settings are made live.
Upon making an `apply` API call, an external settings applier tool is called to apply the changes to the system and restart services as necessary.
There's also a `commit_and_apply` API to do both, which is the most common case.

Requests are directed by `server::router`.
`server::controller` maps requests into our data model.

## Model

The API is driven by a data model (similar to a schema) defined in Rust.
See the 'models' workspace.
All input is deserialized into model types, and all output is serialized from model types, so we can be more confident that data is in the format we expect.

The data model describes system settings, services using those settings, and configuration files used by those services.
It also has a more general structure for metadata.
Metadata entries can be stored for any data field in the model.

## Data store

Data from the model is stored in a key/value data store.
Keys are dotted strings like "settings.service.abc".
This naturally implies some grouping and hierarchy of the data, corresponding to the model.

The current data store implementation maps keys to filesystem paths and stores the value in a file.
Metadata about a data key is stored in a file at the data key path + "." + the metadata key.
The default data store location is `/var/lib/thar/datastore/current`, and the filesystem format makes it fairly easy to inspect.

## Serialization and deserialization

The `datastore::serialization` module provides code to serialize Rust types into a mapping of datastore-acceptable keys (a.b.c) and values.

The `datastore::deserialization` module provides code to deserialize datastore-acceptable keys (a.b.c) and values into Rust types.

# Current limitations

* Data store locking is coarse; read requests can happen in parallel, but a write request will block everything else.
* There's no support for rolling back commits.
* There are no metrics.
* `datastore::serialization` can't handle complex types under lists; it assumes lists can be serialized as scalars.

# Example usage

You can start the API server from the development workspace with a command like:

`cargo run -- --datastore-path /tmp/thar/be/data --socket-path /tmp/thar/api.sock --log-level debug`

Then, from another shell, you can query or modify data.
See `../../apiclient/README.md` for client examples.
*/
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

pub mod datastore;
pub mod server;

pub use server::serve;
