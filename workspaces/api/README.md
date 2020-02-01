# Thar API system

This document describes the Thar API system.

In the [Background](#background) section, the API system's components are described in the order in which they start, so it's a handy reference for the boot process.

The [Development](#development) section has an overview of how to work on the API system.

## Background

Thar is intended to be an API-first operating system - direct user interaction with Thar is usually through the API.
However, the [API server](#apiserver) that handles requests is just one piece.
The remaining components make sure the system is up to date, and that requests are persisted and applied correctly.
Overall, it's the bridge between the user and the underlying system.
It aims to simplify common configuration, improve reliability, and reduce the need for the user to log in and debug.

Several components below mention the *data store*.
This is a key/value store that serves as the central storage location for the API system and tools using the API.
It's described in context in the [API server docs](apiserver/).

### API access

[Further docs](apiclient/)

Users can access the API through the `apiclient` binary.
It's available in Thar, whether you're accessing it through a control channel like SSM or the admin container.
(See the top-level [README](../../) for information about those.)

Rust code can use the `apiclient` library to make requests to the Unix-domain socket of the [apiserver](#apiserver).

## API system components

![API system boot diagram](api-system.png)

### migrator

Further docs:
* [migrator](migration/migrator/)
* [Migration system](migration/)

The migrator ensures the data store is up to date by running any applicable data store migrations.
The existing data store format version is found by looking at the symlink naming in `/var/lib/thar/datastore`, and the incoming data store format version is found by looking at `/usr/share/thar/data-store-version` in the booting image.

On first boot, [storewolf](#storewolf) hasn’t run yet, so there’s no data store, so the migrator has nothing to do.

### storewolf

[Further docs](storewolf/)

storewolf owns the creation and initial population of the data store.

storewolf ensures the default values (defined in [defaults.toml](../models/defaults.toml)) are populated in the data store.
First, it has to create the data store directories and symlinks if they don’t exist.
Then, it goes key-by-key through the defaults, and if a key isn’t already set, sets it with the default value.

The settings are written to the *pending* section of the data store, meaning they’re not available until committed later by [settings-committer](#settings-committer).

If there are any pending settings in the data store, they’re discarded.
We’re unable to guarantee users that any pending settings they haven’t committed will survive a reboot, because we have to be able to commit changes ourselves during the boot process (see later services), and we don’t yet have a way of separating transactions.

### apiserver

[Further docs](apiserver/)

The API server for Thar starts next.
This gives users (and later components) the ability to read or change settings in the data store, and have any changes applied to the system.

### moondog

[Further docs](moondog/)

Moondog applies settings changes that the user requests through EC2 user data.
Think of it as cloud-init but smaller in scope; we only accept TOML-formatted settings that the API understands, right now.

It only runs on first boot.
Users wouldn’t expect settings they send to the API to be overridden every reboot by settings they sent at instance launch time.

The settings are PATCHed to the API and *not* committed, meaning they’re not available until committed later by [settings-committer](#settings-committer).

### sundog

[Further docs](sundog/)

Sundog sets any settings that can't be determined until after the OS is running.
For example, the primary IP address is needed in some config files but can only be determined after a network interface has been attached.

Sundog finds any settings with metadata (“setting-generator”) indicating that they must be generated after boot.
Each key is checked on every boot.
If the key is already set, we don’t need to generate it - either it was generated before, or overridden by the user.
If it’s not set, we could be handling a new key added in a Thar upgrade.

The settings are PATCHed to the API and *not* committed, meaning they’re not available until committed later by [settings-committer](#settings-committer).

#### Pluto

[Further docs](pluto/)

Pluto generates settings needed for Kubernetes configuration, for example cluster DNS.

### settings-committer

[Further docs](settings-committer/)

This binary sends a commit request to the API, which moves all the pending settings from the above services into the live part of the data store.
It's called as a prerequisite of other services, like [sundog](#sundog) and [settings-applier](#settings-applier), that rely on settings being committed.

### settings-applier

Further docs:
* [thar-be-settings](thar-be-settings/), the tool settings-applier uses
* [defaults.toml](../models/defaults.toml), which defines our configuration files and services

This is a simple startup service that runs `thar-be-settings --all` to write out all of the configuration files that are based on our settings.

Most of our root filesystem is not persistent.
We have this service so we can consistently write configuration files, regardless of whether there are any changes to commit during boot.

**Note:** `thar-be-settings --all` also runs service restart commands, which are written so that they don’t start services that haven’t been started yet, so it shouldn’t have an affect during boot.
None of the services above currently use API-configured settings.
Some day we may need to make an earlier service (say, apiserver) configurable through user settings, and that would correctly be restarted here.

**Note:** `thar-be-settings` is also run after the user applies changes through the API.
This usage is scoped to the keys that have changed, updating relevant config files and restarting affected services.
See [thar-be-settings](thar-be-settings/) docs.

### configured.target

This is a systemd target that depends on [settings-applier](#settings-applier) and represents the point at which the system is fully configured.

Applications can depend on this in their service definition.
Services like Kubernetes and containerd depend on this.

## Development

### Local testing

#### Setup

First, you need a data store for the API server.
You can create a data store with storewolf.
These commands create one in `/tmp`, but you can create it in a more permanent location if desired.

From the `workspaces/api/storewolf` directory:

```
cargo run -- --data-store-base-path /tmp/data-store --version 0.1
```

Now you can start the API server.
From the `workspaces/api/apiserver` directory:

```
cargo run -- --datastore-path /tmp/data-store/current --socket-path /tmp/thar-api.sock --log-level debug
```

You can leave that running in a terminal, or background it, whatever you like.

When storewolf creates the data store, it puts settings into `pending` state.
This is so we can commit all settings generated at startup at once.

We can use settings-committer to do the same thing with our development data store.
From the `workspaces/api/settings-committer` directory:

```
cargo run -- --socket-path /tmp/thar-api.sock
```

Now you can inspect settings in the API or do any other testing you like.

You won't have dynamic settings generated by [sundog](#sundog) during a normal Thar launch, but you're probably not locally running the software that needs those, like Kubernetes.
If you are, you can set them manually; see the top-level README for descriptions of those settings.
