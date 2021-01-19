# apiclient

Current version: 0.1.0

## apiclient binary

The `apiclient` binary provides some high-level, synchronous methods of interacting with the API, for example an `update` subcommand that wraps the individual API calls needed to update the host.
There's also a low-level `raw` subcommand for direct interaction with the HTTP API.

It talks to the Bottlerocket socket by default.
It can be pointed to another socket using `--socket-path`, for example for local testing.

### Set mode

This allows you to change settings on the system.

After the settings are changed, they'll be committed and applied.
For example, if you change an NTP setting, the NTP configuration will be updated and the daemon will be restarted.

#### Key=value input

There are two input methods.
The simpler method looks like this:

```
apiclient set settings.x.y.z=VALUE
```

The "settings." prefix on the setting names is optional; this makes it easy to copy and paste settings from documentation, but you can skip the prefix when typing them manually.
Here's an example call:

```
apiclient set kernel.lockdown=integrity motd="hi there"
```

If you're changing a setting whose name requires quoting, please quote the whole key=value argument, so the inner quotes aren't eaten by the shell:

```
apiclient set 'kubernetes.node-labels."my.label"=hello'
```

#### JSON input

This simpler key=value form is convenient for most changes, but sometimes you'll want to specify input in JSON form.
This can be useful if you have multiple changes within a subsection:

```
apiclient set --json '{"kernel": {"sysctl": {"vm.max_map_count": "262144", "user.max_user_namespaces": "16384"}}}'
```

It can also be useful if your desired value is "complex" or looks like a different type.
For example, the "vm.max_map_count" value set above looks like an integer, but the kernel requires a string, so it has to be specified in JSON form and as a string.

As another example, if you want settings.motd to be "42", running `apiclient set motd=42` would fail because `42` is seen as an integer, and motd is a string.
You can use JSON form to set it:

```
apiclient set --json '{"motd": "42"}'
```

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

The apiclient library provides high-level methods to interact with the Bottlerocket API.  See
the documentation for submodules [`reboot`], [`set`], and [`update`] for high-level helpers.

For more control, and to handle APIs without high-level wrappers, there are also 'raw' methods
to query an HTTP API over a Unix-domain socket.

The `raw_request` method takes care of the basics of making an HTTP request on a Unix-domain
socket, and requires you to specify the socket path, the URI (including query string), the
HTTP method, and any request body data.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.