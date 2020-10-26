# apiclient

Current version: 0.1.0

## apiclient binary

The `apiclient` binary helps you talk to an HTTP API over a Unix-domain socket.

It talks to the Bottlerocket socket by default.
It can be pointed to another socket using `--socket-path`, for example for local testing.

The URI path is specified with `-u` or `--uri`, for example `-u /settings`.
This should include the query string, if any.

The HTTP method defaults to GET, and can be changed with `-m`, `-X`, or `--method`.

If you change the method to POST or PATCH, you may also want to send data in the request body.
Specify the data after `-d` or `--data`.

To see verbose response data, including the HTTP status code, use `-v` or `--verbose`.

### Example usage

Getting settings:

```
apiclient -m GET -u /settings
```

Changing settings:

```
apiclient -X PATCH -u /settings -d '{"motd": "my own value!"}'
apiclient -m POST -u /tx/commit_and_apply
```

You can also check what you've changed but not commited by looking at the pending transaction:

```
apiclient -m GET -u /tx
```

(You can group changes into transactions by adding a parameter like `?tx=FOO` to the calls above.)

## apiclient library

The apiclient library provides simple methods to query an HTTP API over a Unix-domain socket.

The `raw_request` method takes care of the basics of making an HTTP request on a Unix-domain
socket, and requires you to specify the socket path, the URI (including query string), the
HTTP method, and any request body data.

In the future, we intend to add methods that understand the Bottlerocket API and help more with common
types of requests.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.