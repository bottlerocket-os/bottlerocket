# apiclient

Current version: 0.1.0

## apiclient binary

The `apiclient` binary provides high-level methods to interact with the Bottlerocket API.
There's a [set](#set-mode) subcommand for changing settings, an [update](#update-mode) subcommand for updating the host, and an [exec](#exec-mode) subcommand for running commands in host containers.
There's also a low-level [raw](#raw-mode) subcommand for direct interaction with the HTTP API.

It talks to the Bottlerocket socket by default.
It can be pointed to another socket using `--socket-path`, for example for local testing.

The most important use is probably checking your current settings:

```shell
apiclient get settings
```

`get` will request all settings whose names start with the given prefix, so you can drill down into specific areas of interest:
```shell
apiclient get settings.host-containers.admin
```

Or, request some specific settings:
```shell
apiclient get settings.motd settings.kernel.lockdown
```

### Set mode

This allows you to change settings on the system.

After the settings are changed, they'll be committed and applied.
For example, if you change an NTP setting, the NTP configuration will be updated and the daemon will be restarted.

#### Key=value input

There are two input methods.
The simpler method looks like this:

```shell
apiclient set settings.x.y.z=VALUE
```

The "settings." prefix on the setting names is optional; this makes it easy to copy and paste settings from documentation, but you can skip the prefix when typing them manually.
Here's an example call:

```shell
apiclient set kernel.lockdown=integrity motd="hi there"
```

If you're changing a setting whose name requires quoting, please quote the whole key=value argument, so the inner quotes aren't eaten by the shell:

```shell
apiclient set 'kubernetes.node-labels."my.label"=hello'
```

#### JSON input

This simpler key=value form is convenient for most changes, but sometimes you'll want to specify input in JSON form.
This can be useful if you have multiple changes within a subsection:

```shell
apiclient set --json '{"kernel": {"sysctl": {"vm.max_map_count": "262144", "user.max_user_namespaces": "16384"}}}'
```

It can also be useful if your desired value is "complex" or looks like a different type.
For example, the "vm.max_map_count" value set above looks like an integer, but the kernel requires a string, so it has to be specified in JSON form and as a string.

As another example, if you want settings.motd to be "42", running `apiclient set motd=42` would fail because `42` is seen as an integer, and motd is a string.
You can use JSON form to set it:

```shell
apiclient set --json '{"motd": "42"}'
```

### Update mode

To start, you can check what updates are available:

```shell
apiclient update check
```

This will show you the current state of the system along with any updates available in the repo; see the [updater README](../../updater/README.md#walkthrough) for details.

Assuming you want to accept the chosen update, you can apply it:

```shell
apiclient update apply
```

This downloads and writes the update to the alternate partition set, then marks it as active.
The next time you reboot, for example with `apiclient reboot`, the update will take effect.

If you're confident that you want to update immediately to the latest version, you can do all of the above in one step:

```shell
apiclient update apply --check --reboot
```

> Note that available updates are controlled by your settings under `settings.updates`; see [README](../../../README.md#updates-settings) for details.

### Reboot mode

This will reboot the system.
You should use this after updating if you didn't specify the `--reboot` flag.

```shell
apiclient reboot
```

### Exec mode

This mode lets you run commands in host containers.
This can be helpful for debugging, particularly if it's hard to access a host container in your configuration, for example getting to the admin container without SSH access.

You can think of it like a slim `ssh` that reuses the communication channel and authorization we already have with the API.

Just tell it the container to run in and the command you want to run.
For example, if you used SSM to get into the control container and want to access the admin container:
```shell
apiclient exec admin bash
```

You can also run noninteractive commands, and redirect output, so you can use it to copy files between containers:
```shell
apiclient exec admin cat /file > file
```

This works OK because apiclient detects if you have a TTY by checking if stdout and stdin are connected to TTYs.
If that doesn't work for your use case, you can pass `-t`/`--tty` to specifically request a TTY, or `-T`/`--no-tty` to request no TTY.

See the [exec documentation](../api-exec.md) for more detail on how this feature works.

### Raw mode

Raw mode lets you make HTTP requests to a UNIX socket.
You can think of it kind of like `curl`, but with more understanding of the Bottlerocket API server; for example, it understands the default path to the API socket, the hostname, and the content type.

The URI path is specified with `-u` or `--uri`, for example `-u /settings`.
This should include the query string, if any.

The HTTP method defaults to GET, and can be changed with `-m`, `-X`, or `--method`.

If you change the method to POST or PATCH, you may also want to send data in the request body.
Specify the data after `-d` or `--data`.

To see verbose response data, including the HTTP status code, use `-v` or `--verbose`.

#### Raw mode walkthrough

To fetch the current settings:

```shell
apiclient raw -u /settings
```

This will return all of the current settings in JSON format.
For example, here's an abbreviated response:
```text
{"motd":"...", {"kubernetes": ...}}
```

You can change settings by sending back the same type of JSON data in a PATCH request.
This can include any number of settings changes.
```shell
apiclient raw -m PATCH -u /settings -d '{"motd": "my own value!"}'
```

This will *stage* the setting in a "pending" area - a transaction.
You can see all your pending settings like this:
```shell
apiclient raw -u /tx
```

To *commit* the settings, and let the system apply them to any relevant configuration files or services, do this:
```shell
apiclient raw -m POST -u /tx/commit_and_apply
```

Behind the scenes, these commands are working with the "default" transaction.
This keeps the interface simple.
System services use their own transactions, so you don't have to worry about conflicts.
For example, there's a "bottlerocket-launch" transaction used to coordinate changes at startup.

If you want to group sets of changes yourself, pick a transaction name and append a `tx` parameter to the URLs above.
For example, if you want the name "FOO", you can `PATCH` to `/settings?tx=FOO` and `POST` to `/tx/commit_and_apply?tx=FOO`.
(Transactions are created automatically when used, and are cleaned up on reboot.)

### Report mode

This allows you to generate certain reports based on the current state of the system.

#### CIS Benchmark Reports

These reports can be used to evaluate the current system state and settings for compliance with the [Center for Internet Security (CIS) Benchmarks].

Additional arguments may be provided to control the CIS Benchmark Level to evaluate and the output format.

**Level**

CIS Benchmarks have a "Level 1" compliance that includes some basic checks that provide a clear security benefit without inhibiting system operation.
By default, Bottlerocket instances meet level 1 compliance unless specific settings are changed that would invalidate that.

The "Level 2" compliance are things that provide more defense in depth and include settings that may be specific to the deployed environment and usage.
While these settings are considered more secure, there may be a trade off with inhibiting utility or performance.

**Format**

The default format of the CIS reports is a human readable text report.
To allow for programmatic parsing of the output, and to get more detailed output including any failure reasons, the format may be changed to `json`.

**Results**

The results from the evaluation of each benchmark item will be one of:

- **PASS**: The system has been evaluated to be in compliance with the benchmark requirement.
- **FAIL**: The system has been evaluated to not be in compliance with the benchmark requirement.
- **SKIP**: Compliance could not be evaluated in an automated fashion and the user must perform manual auditing to evaluate whether the system meets the expected state.

##### Bottlerocket CIS Benchmark report

This command can be used to evaluate the current system state and settings for compliance with the [Bottlerocket CIS Benchmark].

```shell
apiclient report cis
```

By default, the `report cis` command evaluates against the level 1 benchmark requirements.
To check level 2 compliance, use the command:

```shell
apiclient report cis -l 2
```

The format can be changed to `json` using:

```shell
apiclient report cis -f json
```

Refer to the [Bottlerocket CIS Benchmark] for detailed audit and remediation steps.

##### Kubernetes CIS Benchmark report

This command can be used to evaluate the current system state and settings for compliance with the [Kubernetes CIS Benchmark].

```shell
apiclient report cis-k8s
```

By default, the `report cis-k8s` command evaluates against the level 1 benchmark requirements.
To check level 2 compliance, use the command:

```shell
apiclient report cis-k8s -l 2
```

The format can be changed to `json` using:

```shell
apiclient report cis-k8s -f json
```

Refer to the [Kubernetes CIS Benchmark] for detailed audit and remediation steps.

[Center for Internet Security (CIS) Benchmarks]: https://www.cisecurity.org/benchmark/
[Bottlerocket CIS Benchmark]: https://www.cisecurity.org/benchmark/bottlerocket
[Kubernetes CIS Benchmark]: https://www.cisecurity.org/benchmark/kubernetes

## apiclient library

The apiclient library provides high-level methods to interact with the Bottlerocket API.  See
the documentation for submodules [`apply`], [`exec`], [`get`], [`reboot`], [`report`], [`set`],
and [`update`] for high-level helpers.

For more control, and to handle APIs without high-level wrappers, there are also 'raw' methods
to query an HTTP API over a Unix-domain socket.

The `raw_request` method takes care of the basics of making an HTTP request on a Unix-domain
socket, and requires you to specify the socket path, the URI (including query string), the
HTTP method, and any request body data.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
