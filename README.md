# Thar, the Operating System

Welcome to Thar!

Thar is a free and open-source Linux-based operating system meant for hosting containers.

## Tenets

These tenets guide Thar's development.
They let you know what we value and what we're working toward, even if not every feature is ready yet.

### Open

Thar is **open** because the best OS can only be built through collaboration.
It is developed in full view of the world using open source tools and public infrastructure services.
It is not a Kubernetes distro, nor an Amazon distro.
We obsess over shared components like the kernel, but we are willing to accept support for other orchestrators or platforms.

### Small

Thar is **small** because a few big ideas scale better than many small ones.
It includes only the core set of components needed for development and for use at runtime.
Anything we ship, we must be prepared to fix, so our goal is to ship as little as possible while staying useful.

### Secure

Thar is **secure** so it can become a quiet piece of a platform you trust.
It uses a variety of mechanisms to provide defense-in-depth, and enables automatic updates by default.
It protects itself from persistent threats.
It enables kernel features that allow users to assert their own policies for locking down workloads.

### Simple

Thar is **simple** because simple lasts.
Users can pick the image they want, tweak a handful of settings, and then forget about it.
We favor settings that convey high-level intent over those that provide low-level control over specific details, because it is easier to preserve intent across months and years of automatic updates.

## Contact us

If you find a security issue, please [contact our security team](https://github.com/amazonlinux/PRIVATE-thar/security/policy) rather than opening an issue.

We use GitHub issues to track other bug reports and feature requests.
You can select from a few templates and get some guidance on the type of information that would be most helpful.

[Contact us with a new issue here.](https://github.com/amazonlinux/PRIVATE-thar/issues/new/choose)

We don't have other communication channels set up yet, but don't worry about making an issue!
You can let us know about things that seem difficult, or even ways you might like to help.

Thank you!

## Overview

To start, we're focusing on use of Thar as a host OS in Kubernetes clusters.
We’re excited to get early feedback and to continue working on more use cases.

### Setup

:walking: :running:

To get started, please see [INSTALL](INSTALL.md).
It describes:
* how to build an image
* how to register an EC2 AMI from an image
* how to set up a Kubernetes cluster, so your Thar instance can run pods
* how to launch a Thar instance in EC2

### Exploration

To improve security, there's no SSH server in a Thar image, and not even a shell.

Don't panic!

There are a couple out-of-band access methods you can use to explore Thar like you would a typical Linux system.
Either option will give you a shell within Thar.
From there, you can [change settings](#settings), manually [update Thar](#updates), debug problems, and generally explore.

#### Control container

Thar has a "control" container, enabled by default, that runs outside of the orchestrator in a separate instance of containerd.
This container runs the [AWS SSM agent](https://github.com/aws/amazon-ssm-agent) that lets you run commands, or start shell sessions, on Thar instances in EC2.
(You can easily replace this control container with your own just by changing the URI; see [Settings](#settings).

You need to give your instance the SSM role for this to work; see the [setup guide](INSTALL.md).

Once the instance is started, you can start a session:

* Go to AWS SSM's [Session Manager](https://console.aws.amazon.com/systems-manager/session-manager/sessions)
* Select “Start session” and choose your Thar instance
* Select “Start session” again to get a shell

If you prefer a command-line tool, you can start a session with a recent [AWS CLI](https://aws.amazon.com/cli/) and the [session-manager-plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html).
Then you'd be able to start a session using only your instance ID, like this:

```
aws ssm start-session --target INSTANCE_ID
```

With the default control container, you can make API calls to change settings in your Thar host.
To do even more, read the next section about the [admin container](#admin-container).

#### Admin container

Thar has an administrative container, disabled by default, that runs outside of the orchestrator in a separate instance of containerd.
This container has an SSH server that lets you log in as `ec2-user` using your EC2-registered SSH key.
(You can easily replace this admin container with your own just by changing the URI; see [Settings](#settings).

To enable the container, you can change the setting in user data when starting Thar, for example EC2 instance user data:

```
settings.host-containers.admin.enabled = true
```

If Thar is already running, you can change the setting with an API call:

```
apiclient -u /settings -m PATCH -d '{"host-containers": {"admin": {"enabled": true}}}'
apiclient -u /settings/commit_and_apply -m POST
```

(To make an API call like this, you need to use an authenticated channel like [SSM](#control-container).)

Once you're in the admin container, you can run `sheltie` to get a full root shell in the Thar host.
Be careful; while you can inspect and change even more as root, Thar's filesystem and dm-verity setup will prevent most changes from persisting over a restart - see [Security](#security).

### Updates

Rather than a package manager that updates individual pieces of software, Thar downloads a full filesystem image and reboots into it.
It can automatically roll back if boot failures occur, and workload failures can trigger manual rollbacks.

Currently, you can update using a CLI tool, updog.
Here's how you can see whether there's an update:

```
updog check-update
```

Here's how you initiate an update:

```
updog update
reboot
```

The system will automatically roll back if it's unable to boot.
If the update is not functional for a given container workload, you can do a manual rollback:

```
signpost rollback-to-inactive
reboot
```

We're working on more automated update methods.

The update process uses images secured by [TUF](https://theupdateframework.github.io/).
For more details, see the [update system documentation](workspaces/updater/).

## Settings

Here we'll describe the settings you can configure on your Thar instance, and how to do it.

(API endpoints are defined in our [OpenAPI spec](workspaces/api/openapi.yaml) if you want more detail.)

### Interacting with settings

#### Using the API client

You can see the current settings with an API request:
```
apiclient -u /settings
```

This will return all of the current settings in JSON format.
For example, here's an abbreviated response:
```
{"timezone":"America/Los_Angeles","kubernetes":{...}}
```

You can change settings by sending back the same type of JSON data in a PATCH request.
This can include any number of settings changes.
```
apiclient -m PATCH -u /settings -d '{"timezone": "America/Thunder_Bay"}'
```

This will *stage* the setting in a "pending" area.
You can see all the pending settings like this:
```
apiclient -u /settings/pending
```

To *commit* the settings, and let the system apply them to any relevant configuration files or services, do this:
```
apiclient -m POST -u /settings/commit_and_apply
```

For more details on using the client, see the [apiclient documentation](workspaces/api/apiclient/).

#### Using user data

If you know what settings you want to change when you start your Thar instance, you can send them in the user data.

In user data, we structure the settings in TOML form to make things a bit simpler.
Here's the user data to change the time zone setting, as we did in the last section:

```
[settings]
timezone = "America/Thunder_Bay"
```

### Description of settings

Here we'll describe each setting you can change.

**Note:** You can see the [default values](workspaces/api/storewolf/defaults.toml) for any settings that have defaults.

When you're sending settings to the API, or receiving settings from the API, they're in a structured JSON format.
This allows allow modification of any number of keys at once.
It also lets us ensure that they fit the definition of the Thar data model - requests with invalid settings won't even parse correctly, helping ensure safety.

Here, however, we'll use the shortcut "dotted key" syntax for referring to keys.
This is used in some API endpoints with less-structured requests or responses.
It's also more compact for our needs here.

In this format, "settings.kubernetes.cluster-name" refers to the same key as in the JSON `{"settings": {"kubernetes": {"cluster-name": "value"}}}`.

#### Top-level settings

* `settings.timezone`: This doesn't function currently, but is intended to let you set the system timezone, and is specified in [tz database format](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).
* `settings.hostname`: This lets you override the system hostname retrieved from DHCP.

#### Kubernetes settings

The following settings must be specified in order to join a Kubernetes cluster.
You should [specify them in user data](#using-user-data).
See the [setup guide](INSTALL.md) for *much* more detail on setting up Thar and Kubernetes.
* `settings.kubernetes.cluster-name`: The cluster name you chose during setup; the [setup guide](INSTALL.md) uses "thar".
* `settings.kubernetes.cluster-certificate`: This is the base64-encoded certificate authority of the cluster.
* `settings.kubernetes.api-server`: This is the cluster's Kubernetes API endpoint.

The following settings are set for you automatically by [pluto](workspaces/api/) based on runtime instance information, but you can override them if you know what you're doing!
* `settings.kubernetes.max-pods`: The maximum number of pods that can be scheduled on this node (limited by number of available IPv4 addresses)
* `settings.kubernetes.cluster-dns-ip`: The CIDR block of the primary network interface.
* `settings.kubernetes.node-ip`: The IPv4 address of this node.
* `settings.kubernetes.pod-infra-container-image`: The URI of the "pause" container.

#### Updates settings

* `settings.updates.metadata-base-url`: The common portion of all URIs used to download update metadata.
* `settings.updates.target-base-url`: The common portion of all URIs used to download update files.

#### Host containers settings
* `settings.host-containers.admin.source`: The URI of the [admin container](#admin-container).
* `settings.host-containers.admin.enabled`: Whether the admin container is enabled.
* `settings.host-containers.admin.superpowered`: Whether the admin container has high levels of access to the Thar host.
* `settings.host-containers.control.source`: The URI of the [control container](#control-container).
* `settings.host-containers.control.enabled`: Whether the control container is enabled.
* `settings.host-containers.control.superpowered`: Whether the control container has high levels of access to the Thar host.

##### Custom host containers

`admin` and `control` are our default host containers, but you're free to change this.
Beyond just changing the settings above to affect the `admin` and `control` containers, you can add and remove host containers entirely.
As long as you define the three fields above -- `source` with a URI, and `enabled` and `superpowered` with true/false -- you can add host containers with an API call.

Here's an example of adding a custom host container:
```
apiclient -u /settings -X PATCH -d '{"host-containers": {"custom": {"source": "MY-CONTAINER-URI", "enabled": true, "superpowered": false}}}'
apiclient -u /settings/commit_and_apply -X POST
```

If the `enabled` flag is `true`, it will be started automatically.

There are a few important caveats to understand about host containers:
* They're not orchestrated.  They only start or stop according to that `enabled` flag.
* They run in a separate instance of containerd than the one used for orchestrated containers like Kubernetes pods.
* They're not updated automatically.  You need to update the `source`, disable the container, then enable it.
* If you set `superpowered` to true, they'll essentially have root access to the host.

Because of these caveats, host containers are only intended for special use cases.
We use it for the control container because it needs to be available early to give you access to the OS, and we use it for the admin container because it needs high levels of privilege and because you need it to debug when orchestration isn't working.

Be careful, and make sure you have a similar low-level use case before reaching for host containers.

## Details

### Security

We use [dm-verity](https://gitlab.com/cryptsetup/cryptsetup/wikis/DMVerity) to load a verified read-only root filesystem, preventing some classes of persistent security threats.
Only a few locations are made writable:
* some through [tmpfs mounts](workspaces/preinit/laika), used for configuration, that don't persist over a restart.
* one [persistent location](packages/release/var-lib-thar.mount) for the data store.

Almost all first-party components are written in [Rust](https://www.rust-lang.org/).
Rust eliminates some classes of memory safety issues, and encourages design patterns that help security.

### Packaging

Thar is built from source using a container toolchain.
We use RPM package definitions to build and install individual packages into an image.
RPM itself is not in the image - it's just a common and convenient package definition format.

We currently package the following major third-party components:
* Linux kernel ([background](https://en.wikipedia.org/wiki/Linux), [packaging](packages/kernel/))
* glibc ([background](https://www.gnu.org/software/libc/), [packaging](packages/glibc/))
* Buildroot as build toolchain ([background](https://buildroot.org/), [packaging](packages/sdk/))
* GRUB, with patches for partition flip updates ([background](https://www.gnu.org/software/grub/), [packaging](packages/grub/))
* systemd as init ([background](https://en.wikipedia.org/wiki/Systemd), [packaging](packages/systemd/))
* wicked for networking ([background](https://github.com/openSUSE/wicked), [packaging](packages/wicked/))
* containerd ([background](https://containerd.io/), [packaging](packages/containerd/))
* Kubernetes ([background](https://kubernetes.io/), [packaging](packages/kubernetes/))
* Some helpers to make usage in AWS easier:
  * aws-iam-authenticator ([background](https://github.com/kubernetes-sigs/aws-iam-authenticator), [packaging](packages/aws-iam-authenticator/))
  * SSM agent ([background](https://github.com/aws/amazon-ssm-agent), [packaging](packages/ssm/))

For further documentation or to see the rest of the packages, see the [packaging directory](packages/).

### Updates

The Thar image has two identical sets of partitions, A and B.
When updating Thar, the partition table is updated to point from set A to set B, or vice versa.

We also track successful boots, and if there are failures it will automatically revert back to the prior working partition set.

The update process uses images secured by [TUF](https://theupdateframework.github.io/).
For more details, see the [update system documentation](workspaces/updater/).

### API

There are two main ways you'd interact with a production Thar instance.
(There are a couple more [exploration](#exploration) methods above for test instances.)

The first method is through a container orchestrator, for when you want to run or manage containers.
This uses the standard channel for your orchestrator, for example a tool like `kubectl` for Kubernetes.

The second method is through the Thar API, for example when you want to configure the system.

There's an HTTP API server that listens on a local Unix-domain socket.
Remote access to the API requires an authenticated transport such as SSM's RunCommand or Session Manager, as described above.
For more details, see the [apiserver documentation](workspaces/api/apiserver/).

The [apiclient](workspaces/api/apiclient/) can be used to make requests.
They're just HTTP requests, but the API client simplifies making requests with the Unix-domain socket.

To make configuration easier, we have [moondog](workspaces/api/moondog/), which can send an API request for you based on instance user data.
If you start a virtual machine, like an EC2 instance, it will read TOML-formatted Thar configuration from user data and send it to the API server.
This way, you can configure your Thar instance without having to make API calls after launch.

See [Settings](#settings) above for examples and to understand what you can configure.

The server and client are the user-facing components of the API system, but there are a number of other components that work together to make sure your settings are applied, and that they survive upgrades of Thar.

For more details, see the [API system documentation](workspaces/api/).
