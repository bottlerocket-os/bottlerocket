# apiserver

Current version: 0.1.0

## Background

This library provides an API server intended for use in an OS that is primarily accessible through the API.
It's intended to be the primary way to read and modify OS settings, to update services based on those settings, and more generally to learn about and change the state of the system.

The server listens to HTTP requests on a Unix-domain socket.
There is no built-in authentication - local access to the socket should be limited to processes and containers that should be able to configure the system.
Remote access should only be allowed through an authenticated control channel such as SSH or SSM.

## Design

### API

We present an HTTP interface to configurable settings and other state.
The interface is documented in [OpenAPI format](https://swagger.io/docs/specification/about/) in [openapi.yaml](../openapi.yaml).

The Settings APIs are particularly important.
You can GET settings from the `/settings` endpoint.
You can also PATCH changes to the `/settings` endpoint.
Settings are stored as a pending transaction until a commit API is called.
Pending settings can be retrieved from `/tx` to see what will change.

Upon making a `/tx/commit` POST call, the pending transaction is made live.
Upon making an `/tx/apply` POST call, an external settings applier tool is called to apply the changes to the system and restart services as necessary.
There's also `/tx/commit_and_apply` to do both, which is the most common case.

If you don't specify a transaction, the "default" transaction is used, so you usually don't have to think about it.
If you want to group changes into transactions yourself, you can add a `tx` parameter to the APIs mentioned above.
For example, if you want the name "FOO", you can `PATCH` to `/settings?tx=FOO` and `POST` to `/tx/commit_and_apply?tx=FOO`.

Requests are directed by `server::router`.
`server::controller` maps requests into our data model.

### Model

The API is driven by a data model (similar to a schema) defined in Rust.
(See the [models](../../models) directory for model definitions and more documentation.)
All input is deserialized into model types, and all output is serialized from model types, so we can be more confident that data is in the format we expect.

The data model describes system settings, services using those settings, and configuration files used by those services.
It also has a more general structure for metadata.
Metadata entries can be stored for any data field in the model.

### Data store

Data from the model is stored in a key/value data store.
Keys are dotted strings like "settings.service.abc".
This naturally implies some grouping and hierarchy of the data, corresponding to the model.

The current data store implementation maps keys to filesystem paths and stores the value in a file.
Metadata about a data key is stored in a file at the data key path + "." + the metadata key.
The default data store location is `/var/lib/bottlerocket/datastore/current`, and the filesystem format makes it fairly easy to inspect.

For more detail, see [datastore](../datastore).

## Current limitations

* Data store locking is coarse; read requests can happen in parallel, but a write request will block everything else.
* There's no support for rolling back commits.
* There are no metrics.

## Example usage

You can start the API server from the `apiserver` directory with a command like:

`cargo run -- --datastore-path /tmp/bottlerocket/data --socket-path /tmp/bottlerocket/api.sock --log-level debug`

Then, from another shell, you can query or modify data.
See `../../apiclient/README.md` for client examples.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
