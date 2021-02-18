# Bottlerocket OS

Welcome to Bottlerocket!

Bottlerocket is a free and open-source Linux-based operating system meant for hosting containers.

If you’re ready to jump right in, read our [QUICKSTART for Kubernetes](QUICKSTART-EKS.md) to try Bottlerocket in an Amazon EKS cluster or our [QUICKSTART for Amazon ECS](QUICKSTART-ECS.md) to try Bottlerocket in an Amazon ECS cluster.

Bottlerocket focuses on security and maintainability, providing a reliable, consistent, and safe platform for container-based workloads.
This is a reflection of what we've learned building operating systems and services at Amazon.
You can read more about what drives us in [our charter](CHARTER.md).

The base operating system has just what you need to run containers reliably, and is built with standard open-source components.
Bottlerocket-specific additions focus on reliable updates and on the API.
Instead of making configuration changes manually, you can change settings with an API call, and these changes are automatically migrated through updates.

Some notable features include:
* [API access](#api) for configuring your system, with secure out-of-band [access methods](#exploration) when you need them.
* [Updates](#updates) based on partition flips, for fast and reliable system updates.
* [Modeled configuration](#settings) that's automatically migrated through updates.
* [Security](#security) as a top priority.

## Contact us

If you find a security issue, please [contact our security team](https://github.com/bottlerocket-os/bottlerocket/security/policy) rather than opening an issue.

If you're interested in contributing, thank you!
Please see our [contributor's guide](CONTRIBUTING.md).

We use GitHub issues to track other bug reports and feature requests.
You can look at [existing issues](https://github.com/bottlerocket-os/bottlerocket/issues) to see whether your concern is already known.

If not, you can select from a few templates and get some guidance on the type of information that would be most helpful.
[Contact us with a new issue here.](https://github.com/bottlerocket-os/bottlerocket/issues/new/choose)

If you just have questions about Bottlerocket, please feel free to [start or join a discussion](https://github.com/bottlerocket-os/bottlerocket/discussions).

We don't have other communication channels set up quite yet, but don't worry about making an issue or a discussion thread!
You can let us know about things that seem difficult, or even ways you might like to help.

## Variants

To start, we're focusing on the use of Bottlerocket as a host OS in AWS EKS Kubernetes clusters and Amazon ECS clusters.
We’re excited to get early feedback and to continue working on more use cases!

Bottlerocket is architected such that different cloud environments and container orchestrators can be supported in the future.
A build of Bottlerocket that supports different features or integration characteristics is known as a 'variant'.
The artifacts of a build will include the architecture and variant name.
For example, an `x86_64` build of the `aws-k8s-1.17` variant will produce an image named `bottlerocket-aws-k8s-1.17-x86_64-<version>-<commit>.img`.

Our first supported variants, `aws-k8s-1.15`, `aws-k8s-1.16`, and `aws-k8s-1.17`, support EKS as described above.
We also have a new `aws-ecs-1` variant designed to work with ECS.

## Architectures

Our supported architectures include `x86_64` and `aarch64` (written as `arm64` in some contexts).

## Setup

:walking: :running:

Bottlerocket is best used with a container orchestrator.
To get started with Kubernetes, please see [QUICKSTART-EKS](QUICKSTART-EKS.md).
To get started with Amazon ECS, please see [QUICKSTART-ECS](QUICKSTART-ECS.md).
These guides describe:
* how to set up a cluster with the orchestrator, so your Bottlerocket instance can run containers
* how to launch a Bottlerocket instance in EC2

To build your own Bottlerocket images, please see [BUILDING](BUILDING.md).
It describes:
* how to build an image
* how to register an EC2 AMI from an image

To publish your built Bottlerocket images, please see [PUBLISHING](PUBLISHING.md).
It describes:
* how to make TUF repos including your image
* how to copy your AMI across regions
* how to mark your AMIs public or grant access to specific accounts
* how to make your AMIs discoverable using [SSM parameters](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)

## Exploration

To improve security, there's no SSH server in a Bottlerocket image, and not even a shell.

Don't panic!

There are a couple out-of-band access methods you can use to explore Bottlerocket like you would a typical Linux system.
Either option will give you a shell within Bottlerocket.
From there, you can [change settings](#settings), manually [update Bottlerocket](#updates), debug problems, and generally explore.

**Note:** These methods require that your instance has permission to access the ECR repository where these containers live; the appropriate policy to add to your instance's IAM role is `AmazonEC2ContainerRegistryReadOnly`.

### Control container

Bottlerocket has a ["control" container](https://github.com/bottlerocket-os/bottlerocket-control-container), enabled by default, that runs outside of the orchestrator in a separate instance of containerd.
This container runs the [AWS SSM agent](https://github.com/aws/amazon-ssm-agent) that lets you run commands, or start shell sessions, on Bottlerocket instances in EC2.
(You can easily replace this control container with your own just by changing the URI; see [Settings](#settings).)

You need to give your instance the SSM role for this to work; see the [setup guide](QUICKSTART-EKS.md#enabling-ssm).

Once the instance is started, you can start a session:

* Go to AWS SSM's [Session Manager](https://console.aws.amazon.com/systems-manager/session-manager/sessions)
* Select “Start session” and choose your Bottlerocket instance
* Select “Start session” again to get a shell

If you prefer a command-line tool, you can start a session with a recent [AWS CLI](https://aws.amazon.com/cli/) and the [session-manager-plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html).
Then you'd be able to start a session using only your instance ID, like this:

```
aws ssm start-session --target INSTANCE_ID
```

With the [default control container](https://github.com/bottlerocket-os/bottlerocket-control-container), you can make [API calls](#api) to configure and manage your Bottlerocket host.
To do even more, read the next section about the [admin container](#admin-container).

### Admin container

Bottlerocket has an [administrative container](https://github.com/bottlerocket-os/bottlerocket-admin-container), disabled by default, that runs outside of the orchestrator in a separate instance of containerd.
This container has an SSH server that lets you log in as `ec2-user` using your EC2-registered SSH key.
(You can easily replace this admin container with your own just by changing the URI; see [Settings](#settings).

To enable the container, you can change the setting in user data when starting Bottlerocket, for example EC2 instance user data:

```
[settings.host-containers.admin]
enabled = true
```

If Bottlerocket is already running, you can enable the admin container from the default [control container](#control-container) like this:

```
enable-admin-container
```

If you're using a custom control container, or want to make the API calls directly, you can enable the admin container like this instead:

```
apiclient -u /settings -m PATCH -d '{"host-containers": {"admin": {"enabled": true}}}'
apiclient -u /tx/commit_and_apply -m POST
```

Once you're in the admin container, you can run `sheltie` to get a full root shell in the Bottlerocket host.
Be careful; while you can inspect and change even more as root, Bottlerocket's filesystem and dm-verity setup will prevent most changes from persisting over a restart - see [Security](#security).

## Updates

Rather than a package manager that updates individual pieces of software, Bottlerocket downloads a full filesystem image and reboots into it.
It can automatically roll back if boot failures occur, and workload failures can trigger manual rollbacks.

The update process uses images secured by [TUF](https://theupdateframework.github.io/).
For more details, see the [update system documentation](sources/updater/).

### Update methods

There are several ways of updating your Bottlerocket hosts.

For EKS variants of Bottlerocket, we recommend using the [Bottlerocket update operator](https://github.com/bottlerocket-os/bottlerocket-update-operator) for automated updates.
You can also use one of the methods below for direct control of updates.

For the ECS preview variant of Bottlerocket, we recommend updating hosts using one of the methods below, until further automation is ready.

#### Update API

The [Bottlerocket API](#api) includes methods for checking and starting system updates.  You can read more about the update APIs in our [update system documentation](sources/updater/README.md#update-api).

#### Updog

You can update Bottlerocket using a CLI tool, `updog`, if you [connect through the admin container](#admin-container).

Here's how you can see whether there's an update:

```
updog check-update
```

Here's how you initiate an update:

```
updog update
reboot
```

(If you know what you're doing and want to update *now*, you can run `updog update --reboot --now`)

#### Bottlerocket Update Operator

If you are running the Kubernetes variant of Bottlerocket, you can use the [Bottlerocket update operator](https://github.com/bottlerocket-os/bottlerocket-update-operator) to automate Bottlerocket updates.

### Update rollback

The system will automatically roll back if it's unable to boot.
If the update is not functional for a given container workload, you can do a manual rollback:

```
signpost rollback-to-inactive
reboot
```

## Settings

Here we'll describe the settings you can configure on your Bottlerocket instance, and how to do it.

(API endpoints are defined in our [OpenAPI spec](sources/api/openapi.yaml) if you want more detail.)

### Interacting with settings

#### Using the API client

You can see the current settings with an API request:
```
apiclient -u /settings
```

This will return all of the current settings in JSON format.
For example, here's an abbreviated response:
```
{"motd":"...", {"kubernetes": ...}}
```

You can change settings by sending back the same type of JSON data in a PATCH request.
This can include any number of settings changes.
```
apiclient -m PATCH -u /settings -d '{"motd": "my own value!"}'
```

This will *stage* the setting in a "pending" area - a transaction.
You can see all your pending settings like this:
```
apiclient -u /tx
```

To *commit* the settings, and let the system apply them to any relevant configuration files or services, do this:
```
apiclient -m POST -u /tx/commit_and_apply
```

Behind the scenes, these commands are working with the "default" transaction.
This keeps the interface simple.
System services use their own transactions, so you don't have to worry about conflicts.
For example, there's a "bottlerocket-launch" transaction used to coordinate changes at startup.

If you want to group sets of changes yourself, pick a transaction name and append a `tx` parameter to the URLs above.
For example, if you want the name "FOO", you can `PATCH` to `/settings?tx=FOO` and `POST` to `/tx/commit_and_apply?tx=FOO`.
(Transactions are created automatically when used, and are cleaned up on reboot.)

For more details on using the client, see the [apiclient documentation](sources/api/apiclient/).

#### Using user data

If you know what settings you want to change when you start your Bottlerocket instance, you can send them in the user data.

In user data, we structure the settings in TOML form to make things a bit simpler.
Here's the user data to change the message of the day setting, as we did in the last section:

```
[settings]
motd = "my own value!"
```

### Description of settings

Here we'll describe each setting you can change.

**Note:** You can see the default values (for any settings that are not generated at runtime) by looking in the `defaults.d` directory for a variant, for example [aws-ecs-1](sources/models/src/aws-ecs-1/defaults.d/).

When you're sending settings to the API, or receiving settings from the API, they're in a structured JSON format.
This allows modification of any number of keys at once.
It also lets us ensure that they fit the definition of the Bottlerocket data model - requests with invalid settings won't even parse correctly, helping ensure safety.

Here, however, we'll use the shortcut "dotted key" syntax for referring to keys.
This is used in some API endpoints with less-structured requests or responses.
It's also more compact for our needs here.

In this format, "settings.kubernetes.cluster-name" refers to the same key as in the JSON `{"settings": {"kubernetes": {"cluster-name": "value"}}}`.

#### Top-level settings

* `settings.motd`: This setting is just written out to /etc/motd. It's useful as a way to get familiar with the API!  Try changing it.

#### Kubernetes settings

See the [setup guide](QUICKSTART-EKS.md) for much more detail on setting up Bottlerocket and Kubernetes.

The following settings must be specified in order to join a Kubernetes cluster.
You should [specify them in user data](#using-user-data).
* `settings.kubernetes.cluster-name`: The cluster name you chose during setup; the [setup guide](QUICKSTART-EKS.md) uses "bottlerocket".
* `settings.kubernetes.cluster-certificate`: This is the base64-encoded certificate authority of the cluster.
* `settings.kubernetes.api-server`: This is the cluster's Kubernetes API endpoint.

The following settings can be optionally set to customize the node labels and taints. 
* `settings.kubernetes.node-labels`: [Labels](https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/) in the form of key, value pairs added when registering the node in the cluster.
* `settings.kubernetes.node-taints`: [Taints](https://kubernetes.io/docs/concepts/configuration/taint-and-toleration/) in the form of key, value and effect entries added when registering the node in the cluster.
  * Example user data for setting up labels and taints:
    ```
    [settings.kubernetes.node-labels]
    label1 = "foo"
    label2 = "bar"
    [settings.kubernetes.node-taints]
    dedicated = "experimental:PreferNoSchedule"
    special = "true:NoSchedule"
    ```

The following settings are optional and allow you to further configure your cluster.
* `settings.kubernetes.cluster-domain`: The DNS domain for this cluster, allowing all Kubernetes-run containers to search this domain before the host's search domains.  Defaults to `cluster.local`.

You can also optionally specify static pods for your node with the following settings.
* `settings.kubernetes.static-pods.<custom identifier>.manifest`: A base64-encoded pod manifest.
* `settings.kubernetes.static-pods.<custom identifier>.enabled`: Whether the static pod is enabled.

The following settings are set for you automatically by [pluto](sources/api/) based on runtime instance information, but you can override them if you know what you're doing!
* `settings.kubernetes.max-pods`: The maximum number of pods that can be scheduled on this node (limited by number of available IPv4 addresses)
* `settings.kubernetes.cluster-dns-ip`: The CIDR block of the primary network interface.
* `settings.kubernetes.node-ip`: The IPv4 address of this node.
* `settings.kubernetes.pod-infra-container-image`: The URI of the "pause" container.

#### Amazon ECS settings

See the [setup guide](QUICKSTART-ECS.md) for much more detail on setting up Bottlerocket and ECS.

The following settings are optional and allow you to configure how your instance joins an ECS cluster.
Since joining a cluster happens at startup, they need to be [specified in user data](#using-user-data).
* `settings.ecs.cluster`: The name or [ARN](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html) of your Amazon ECS cluster.
  If left unspecified, Bottlerocket will join your `default` cluster.
* `settings.ecs.instance-attributes`: [Attributes](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-placement-constraints.html#attributes) in the form of key, value pairs added when registering the container instance in the cluster.
  * Example user data for setting up attributes:
    ```
    [settings.ecs.instance-attributes]
    attribute1 = "foo"
    attribute2 = "bar"
    ```

The following settings are optional and allow you to further configure your cluster.
These settings can be changed at any time.
* `settings.ecs.logging-drivers`: The list of logging drivers available on the container instance.
  The ECS agent running on a container instance must register available logging drivers before tasks that use those drivers are eligible to be placed on the instance.
  Bottlerocket enables the `json-file`, `awslogs`, and `none` drivers by default.
* `settings.ecs.allow-privileged-containers`: Whether launching privileged containers is allowed on the container instance.
  If this value is set to false, privileged containers are not permitted.
  Bottlerocket sets this value to false by default. 
* `settings.ecs.loglevel`: The level of verbosity for the ECS agent's logs.
  Supported values are `debug`, `info`, `warn`, `error`, and `crit`, and the default is `info`.
* `settings.ecs.enable-spot-instance-draining`: If the instance receives a spot termination notice, the agent will set the instance's state to `DRAINING`, so the workload can be moved gracefully before the instance is removed. Defaults to `false`.

#### Updates settings

* `settings.updates.metadata-base-url`: The common portion of all URIs used to download update metadata.
* `settings.updates.targets-base-url`: The common portion of all URIs used to download update files.
* `settings.updates.seed`: A `u32` value that determines how far into the update schedule this machine will accept an update.  We recommend leaving this at its default generated value so that updates can be somewhat randomized in your cluster.
* `settings.updates.version-lock`: Controls the version that will be selected when you issue an update request.  Can be locked to a specific version like `v1.0.0`, or `latest` to take the latest available version.  Defaults to `latest`.
* `settings.updates.ignore-waves`: Updates are rolled out in waves to reduce the impact of issues.  For testing purposes, you can set this to `true` to ignore those waves and update immediately.

#### Network settings

##### Proxy settings

These settings will configure the proxying behavior of the following services:
* For all variants:
    * [containerd.service](packages/containerd/containerd.service)
    * [host-containerd.service](packages/host-ctr/host-containerd.service)
* For Kubernetes variants:
    * [kubelet.service](packages/kubernetes-1.18/kubelet.service)
* For the ECS variant:
    * [docker.service](packages/docker-engine/docker.service)
    * [ecs.service](packages/ecs-agent/ecs.service)

* `settings.network.https-proxy`: The HTTPS proxy server to be used by services listed above.
* `settings.network.no-proxy`: A list of hosts that are excluded from proxying.

The no-proxy list will automatically include entries for localhost.

If you're running a Kubernetes variant, the no-proxy list will automatically include the Kubernetes API server endpoint and other commonly used Kubernetes DNS suffixes to facilitate intra-cluster networking.

#### Time settings

* `settings.ntp.time-servers`: A list of NTP servers used to set and verify the system time.

#### Kernel settings

* `settings.kernel.lockdown`: This allows further restrictions on what the Linux kernel will allow, for example preventing the loading of unsigned modules.
  May be set to "none" (the default), "integrity", or "confidentiality".
  **Important note:** this setting cannot be lowered (toward 'none') at runtime.
  You must reboot for a change to a lower level to take effect.
* `settings.kernel.sysctl`: Key/value pairs representing Linux kernel parameters.
  Remember to quote keys (since they often contain ".") and to quote all values.
  * Example user data for setting up sysctl:
    ```
    [settings.kernel.sysctl]
    "user.max_user_namespaces" = "16384"
    "vm.max_map_count" = "262144"
    ```


#### Host containers settings
* `settings.host-containers.admin.source`: The URI of the [admin container](#admin-container).
* `settings.host-containers.admin.enabled`: Whether the admin container is enabled.
* `settings.host-containers.admin.superpowered`: Whether the admin container has high levels of access to the Bottlerocket host.
* `settings.host-containers.control.source`: The URI of the [control container](#control-container).
* `settings.host-containers.control.enabled`: Whether the control container is enabled.
* `settings.host-containers.control.superpowered`: Whether the control container has high levels of access to the Bottlerocket host.

##### Custom host containers

[`admin`](https://github.com/bottlerocket-os/bottlerocket-admin-container) and [`control`](https://github.com/bottlerocket-os/bottlerocket-control-container) are our default host containers, but you're free to change this.
Beyond just changing the settings above to affect the `admin` and `control` containers, you can add and remove host containers entirely.
As long as you define the three fields above -- `source` with a URI, and `enabled` and `superpowered` with true/false -- you can add host containers with an API call or user data.

You can optionally define a `user-data` field with arbitrary base64-encoded data, which will be made available in the container at `/.bottlerocket/host-containers/$HOST_CONTAINER_NAME/user-data`.
(It was inspired by instance user data, but is entirely separate; it can be any data your host container feels like interpreting.)

Here's an example of adding a custom host container with API calls:
```
apiclient -u /settings -X PATCH -d '{"host-containers": {"custom": {"source": "MY-CONTAINER-URI", "enabled": true, "superpowered": false}}}'
apiclient -u /tx/commit_and_apply -X POST
```

Here's the same example, but with the settings you'd add to user data:
```
[settings.host-containers.custom]
enabled = true
source = "MY-CONTAINER-URI"
superpowered = false
```

If the `enabled` flag is `true`, it will be started automatically.

All host containers will have the `apiclient` binary available at `/usr/local/bin/apiclient` so they're able to [interact with the API](#using-the-api-client).

In addition, all host containers come with persistent storage at `/.bottlerocket/host-containers/$HOST_CONTAINER_NAME` that is persisted across reboots and container start/stop cycles.
The default `admin` host-container, for example, stores its SSH host keys under `/.bottlerocket/host-containers/admin/etc/ssh/`.

There are a few important caveats to understand about host containers:
* They're not orchestrated.  They only start or stop according to that `enabled` flag.
* They run in a separate instance of containerd than the one used for orchestrated containers like Kubernetes pods.
* They're not updated automatically.  You need to update the `source`, disable the container, commit those changes, then re-enable it.
* If you set `superpowered` to true, they'll essentially have root access to the host.

Because of these caveats, host containers are only intended for special use cases.
We use it for the control container because it needs to be available early to give you access to the OS, and we use it for the admin container because it needs high levels of privilege and because you need it to debug when orchestration isn't working.

Be careful, and make sure you have a similar low-level use case before reaching for host containers.

#### Platform-specific settings

Platform-specific settings are automatically set at boot time by [early-boot-config](sources/api/early-boot-config) based on metadata available on the running platform.
They can be overridden for testing purposes in [the same way as other settings](#interacting-with-settings).

##### AWS-specific settings

AWS-specific settings are automatically set based on calls to the Instance MetaData Service (IMDS).

* `settings.aws.region`: This is set to the AWS region in which the instance is running, for example `us-west-2`.

### Logs

You can use `logdog` through the [admin container](#admin-container) to obtain an archive of log files from your Bottlerocket host.
SSH to the Bottlerocket host, then run:

```bash
sudo sheltie
logdog
```

This will write an archive of the logs to `/tmp/bottlerocket-logs.tar.gz`.
You can use SSH to retrieve the file.
Once you have exited from the Bottlerocket host, run a command like:

```bash
ssh -i YOUR_KEY_FILE \
    ec2-user@YOUR_HOST \
    "cat /.bottlerocket/rootfs/tmp/bottlerocket-logs.tar.gz" > bottlerocket-logs.tar.gz
```

For a list of what is collected, see the logdog [command list](sources/logdog/src/log_request.rs).

## Details

### Security

:shield: :crab:

To learn more about security features in Bottlerocket, please see [SECURITY FEATURES](SECURITY_FEATURES.md).
It describes how we use features like [dm-verity](https://gitlab.com/cryptsetup/cryptsetup/wikis/DMVerity) and [SELinux](https://selinuxproject.org/) to protect the system from security threats.

To learn more about security recommendations for Bottlerocket, please see [SECURITY GUIDANCE](SECURITY_GUIDANCE.md).
It documents additional steps you can take to secure the OS, and includes resources such as a [Pod Security Policy](https://kubernetes.io/docs/concepts/policy/pod-security-policy/) for your reference.

In addition, almost all first-party components are written in [Rust](https://www.rust-lang.org/).
Rust eliminates some classes of memory safety issues, and encourages design patterns that help security.

### Packaging

Bottlerocket is built from source using a container toolchain.
We use RPM package definitions to build and install individual packages into an image.
RPM itself is not in the image - it's just a common and convenient package definition format.

We currently package the following major third-party components:
* Linux kernel ([background](https://en.wikipedia.org/wiki/Linux), [packaging](packages/kernel/))
* glibc ([background](https://www.gnu.org/software/libc/), [packaging](packages/glibc/))
* Buildroot as build toolchain ([background](https://buildroot.org/), via the [SDK](https://github.com/bottlerocket-os/bottlerocket-sdk))
* GRUB, with patches for partition flip updates ([background](https://www.gnu.org/software/grub/), [packaging](packages/grub/))
* systemd as init ([background](https://en.wikipedia.org/wiki/Systemd), [packaging](packages/systemd/))
* wicked for networking ([background](https://github.com/openSUSE/wicked), [packaging](packages/wicked/))
* containerd ([background](https://containerd.io/), [packaging](packages/containerd/))
* Kubernetes ([background](https://kubernetes.io/), [packaging](packages/kubernetes-1.15/))
* aws-iam-authenticator ([background](https://github.com/kubernetes-sigs/aws-iam-authenticator), [packaging](packages/aws-iam-authenticator/))
* Amazon ECS agent ([background](https://github.com/aws/amazon-ecs-agent), [packaging](packages/ecs-agent/))

For further documentation or to see the rest of the packages, see the [packaging directory](packages/).

### Updates

The Bottlerocket image has two identical sets of partitions, A and B.
When updating Bottlerocket, the partition table is updated to point from set A to set B, or vice versa.

We also track successful boots, and if there are failures it will automatically revert back to the prior working partition set.

The update process uses images secured by [TUF](https://theupdateframework.github.io/).
For more details, see the [update system documentation](sources/updater/).

### API

There are two main ways you'd interact with a production Bottlerocket instance.
(There are a couple more [exploration](#exploration) methods above for test instances.)

The first method is through a container orchestrator, for when you want to run or manage containers.
This uses the standard channel for your orchestrator, for example a tool like `kubectl` for Kubernetes.

The second method is through the Bottlerocket API, for example when you want to configure the system.

There's an HTTP API server that listens on a local Unix-domain socket.
Remote access to the API requires an authenticated transport such as SSM's RunCommand or Session Manager, as described above.
For more details, see the [apiserver documentation](sources/api/apiserver/).

The [apiclient](sources/api/apiclient/) can be used to make requests.
They're just HTTP requests, but the API client simplifies making requests with the Unix-domain socket.

To make configuration easier, we have [early-boot-config](sources/api/early-boot-config/), which can send an API request for you based on instance user data.
If you start a virtual machine, like an EC2 instance, it will read TOML-formatted Bottlerocket configuration from user data and send it to the API server.
This way, you can configure your Bottlerocket instance without having to make API calls after launch.

See [Settings](#settings) above for examples and to understand what you can configure.

The server and client are the user-facing components of the API system, but there are a number of other components that work together to make sure your settings are applied, and that they survive upgrades of Bottlerocket.

For more details, see the [API system documentation](sources/api/).
