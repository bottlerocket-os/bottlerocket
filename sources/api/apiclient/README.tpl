# {{crate}}

Current version: {{version}}

## apiclient binary

The `apiclient` binary provides some high-level, synchronous methods of interacting with the API, for example an `update` subcommand that wraps the individual API calls needed to update the host.
There's also a low-level `raw` subcommand for direct interaction with the HTTP API.

It talks to the Bottlerocket socket by default.
It can be pointed to another socket using `--socket-path`, for example for local testing.

### Update mode

To start, you can check what updates are available:

```
apiclient update check
```

This will show you the current state of the system along with any updates available in the repo; see the [updater README](../../updater/README.md#walkthrough) for details.

Assuming you want to accept the chosen update, you can apply it:

```
apiclient update apply
```

This downloads and writes the update to the alternate partition set, then marks it as active.
The next time you reboot, for example with `apiclient reboot`, the update will take effect.

If you're confident that you want to update immediately to the latest version, you can do all of the above in one step:

```
apiclient update apply --check --reboot
```

> Note that available updates are controlled by your settings under `settings.updates`; see [README](../../../README.md#updates-settings) for details.

### Reboot mode

This will reboot the system.
You should use this after updating if you didn't specify the `--reboot` flag.

```
apiclient reboot
```

### Raw mode

Raw mode lets you make HTTP requests to a UNIX socket.
You can think of it kind of like `curl`, but with more understanding of the Bottlerocket API server; for example, it understands the default path to the API socket, the hostname, and the content type.

The URI path is specified with `-u` or `--uri`, for example `-u /settings`.
This should include the query string, if any.

The HTTP method defaults to GET, and can be changed with `-m`, `-X`, or `--method`.

If you change the method to POST or PATCH, you may also want to send data in the request body.
Specify the data after `-d` or `--data`.

To see verbose response data, including the HTTP status code, use `-v` or `--verbose`.

#### Examples

Getting settings:

```
apiclient raw -m GET -u /settings
```

Changing settings:

```
apiclient raw -X PATCH -u /settings -d '{"motd": "my own value!"}'
apiclient raw -m POST -u /tx/commit_and_apply
```

You can also check what you've changed but not commited by looking at the pending transaction:

```
apiclient raw -m GET -u /tx
```

(You can group changes into transactions by adding a parameter like `?tx=FOO` to the calls above.)

## apiclient library

{{readme}}

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
