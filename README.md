# Bottlerocket OS

Welcome to Bottlerocket!

Bottlerocket is a free and open-source Linux-based operating system meant for hosting containers.

If you’re ready to jump right in, read one of our setup guides for running Bottlerocket in [Amazon EKS](QUICKSTART-EKS.md), [Amazon ECS](QUICKSTART-ECS.md), or [VMware](QUICKSTART-VMWARE.md).

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
For example, an `x86_64` build of the `aws-k8s-1.19` variant will produce an image named `bottlerocket-aws-k8s-1.19-x86_64-<version>-<commit>.img`.

The following variants support EKS, as described above:

- `aws-k8s-1.17`
- `aws-k8s-1.18`
- `aws-k8s-1.19`
- `aws-k8s-1.20`
- `aws-k8s-1.21`

The following variant supports ECS:

- `aws-ecs-1`

We also have variants in preview status that are designed to be Kubernetes worker nodes in VMware:

- `vmware-k8s-1.20`
- `vmware-k8s-1.21`

The `aws-k8s-1.16` variant is deprecated and will no longer be supported in Bottlerocket releases after June, 2021.
The `aws-k8s-1.15` variant is no longer supported.
We recommend users replace `aws-k8s-1.15` and `aws-k8s-1.16` nodes with the [latest variant compatible with their cluster](variants/).

## Architectures

Our supported architectures include `x86_64` and `aarch64` (written as `arm64` in some contexts).

## Setup

:walking: :running:

Bottlerocket is best used with a container orchestrator.
To get started with Kubernetes in Amazon EKS, please see [QUICKSTART-EKS](QUICKSTART-EKS.md).
To get started with Kubernetes in VMware, please see [QUICKSTART-VMWARE](QUICKSTART-VMWARE.md).
To get started with Amazon ECS, please see [QUICKSTART-ECS](QUICKSTART-ECS.md).
These guides describe:
* how to set up a cluster with the orchestrator, so your Bottlerocket instance can run containers
* how to launch a Bottlerocket instance in EC2 or VMware

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

In AWS, you need to give your instance the SSM role for this to work; see the [setup guide](QUICKSTART-EKS.md#enabling-ssm).
Outside of AWS, you can use [AWS Systems Manager for hybrid environments](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-managedinstances.html).
There's more detail about hybrid environments in the [control container documentation](https://github.com/bottlerocket-os/bottlerocket-control-container/#connecting-to-aws-systems-manager-ssm).

Once the instance is started, you can start a session:

* Go to AWS SSM's [Session Manager](https://console.aws.amazon.com/systems-manager/session-manager/sessions)
* Select "Start session" and choose your Bottlerocket instance
* Select "Start session" again to get a shell

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
Outside of AWS, you can [pass in your own SSH keys](https://github.com/bottlerocket-os/bottlerocket-admin-container#authenticating-with-the-admin-container).
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
apiclient set host-containers.admin.enabled=true
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
We provide tools for automatically updating hosts, as well as an API for direct control of updates.

#### Automated updates

For EKS variants of Bottlerocket, we recommend using the [Bottlerocket update operator](https://github.com/bottlerocket-os/bottlerocket-update-operator) for automated updates.

For the ECS variant of Bottlerocket, we recommend using the [Bottlerocket ECS updater](https://github.com/bottlerocket-os/bottlerocket-ecs-updater/) for automated updates.

#### Update API

The [Bottlerocket API](#api) includes methods for checking and starting system updates.
You can read more about the update APIs in our [update system documentation](sources/updater/README.md#update-api).

apiclient knows how to handle those update APIs for you, and you can run it from the [control](#control-container) or [admin](#admin-container) containers.

To see what updates are available:
```
apiclient update check
```
If an update is available, it will show up in the `chosen_update` field.
The `available_updates` field will show the full list of available versions, including older versions, because Bottlerocket supports safely rolling back.

To apply the latest update:
```
apiclient update apply
```

The next time you reboot, you'll start up in the new version, and system configuration will be automatically [migrated](sources/api/migration/).
To reboot right away:
```
apiclient reboot
```

If you're confident about updating, the `apiclient update apply` command has `--check` and `--reboot` flags to combine the above actions, so you can accomplish all of the above steps like this:
```
apiclient update apply --check --reboot
```

See the [apiclient documentation](sources/api/apiclient/) for more details.

### Update rollback

The system will automatically roll back if it's unable to boot.
If the update is not functional for a given container workload, you can do a manual rollback:

```
signpost rollback-to-inactive
reboot
```

This doesn't require any external communication, so it's quicker than `apiclient`, and it's made to be as reliable as possible.

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

You can change settings like this:
```
apiclient set motd="hi there" kubernetes.node-labels.environment=test
```

You can also use a JSON input mode to help change many related settings at once, and a "raw" mode if you want more control over how the settings are committed and applied to the system.
See the [apiclient README](sources/api/apiclient/) for details.

#### Using user data

If you know what settings you want to change when you start your Bottlerocket instance, you can send them in the user data.

In user data, we structure the settings in TOML form to make things a bit simpler.
Here's the user data to change the message of the day setting, as we did in the last section:

```
[settings]
motd = "my own value!"
```

If your user data is over the size limit of the platform (e.g. 16KiB for EC2) you can compress the contents with gzip.
(With [aws-cli](https://aws.amazon.com/cli/), you can use `--user-data fileb:///path/to/gz-file` to pass binary data.)

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

See the [EKS setup guide](QUICKSTART-EKS.md) for much more detail on setting up Bottlerocket and Kubernetes in AWS EKS.
For more details about running Bottlerocket as a Kubernetes worker node in VMware, see the [VMware setup guide](QUICKSTART-VMWARE.md).

The following settings must be specified in order to join a Kubernetes cluster.
You should [specify them in user data](#using-user-data).
* `settings.kubernetes.cluster-certificate`: This is the base64-encoded certificate authority of the cluster.
* `settings.kubernetes.api-server`: This is the cluster's Kubernetes API endpoint.

For Kubernetes variants in AWS, you must also specify:
* `settings.kubernetes.cluster-name`: The cluster name you chose during setup; the [setup guide](QUICKSTART-EKS.md) uses "bottlerocket".

For Kubernetes variants in VMware, you must specify:
* `settings.kubernetes.cluster-dns-ip`: The IP of the DNS service running in the cluster.
* `settings.kubernetes.bootstrap-token`: The token used for [TLS bootstrapping](https://kubernetes.io/docs/reference/command-line-tools-refe    rence/kubelet-tls-bootstrapping/).

The following settings can be optionally set to customize the node labels and taints. Remember to quote keys (since they often contain ".") and to quote all values.
* `settings.kubernetes.node-labels`: [Labels](https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/) in the form of key, value pairs added when registering the node in the cluster.
* `settings.kubernetes.node-taints`: [Taints](https://kubernetes.io/docs/concepts/configuration/taint-and-toleration/) in the form of key, value and effect entries added when registering the node in the cluster.
  * Example user data for setting up labels and taints:
    ```
    [settings.kubernetes.node-labels]
    "label1" = "foo"
    "label2" = "bar"
    [settings.kubernetes.node-taints]
    "dedicated" = "experimental:PreferNoSchedule"
    "special" = "true:NoSchedule"
    ```

The following settings are optional and allow you to further configure your cluster.
* `settings.kubernetes.cluster-domain`: The DNS domain for this cluster, allowing all Kubernetes-run containers to search this domain before the host's search domains.  Defaults to `cluster.local`.
* `settings.kubernetes.standalone-mode`: Whether to run the kubelet in standalone mode, without connecting to an API server.  Defaults to `false`.
* `settings.kubernetes.cloud-provider`: The cloud provider for this cluster. Defaults to `aws` for AWS variants, and `external` for other variants.
* `settings.kubernetes.authentication-mode`: Which authentication method the kubelet should use to connect to the API server, and for incoming requests.  Defaults to `aws` for AWS variants, and `tls` for other variants.
* `settings.kubernetes.server-tls-bootstrap`: Enables or disables server certificate bootstrap.  When enabled, the kubelet will request a certificate from the certificates.k8s.io API.  This requires an approver to approve the certificate signing requests (CSR).  Defaults to `true`.
* `settings.kubernetes.bootstrap-token`: The token to use for [TLS bootstrapping](https://kubernetes.io/docs/reference/command-line-tools-reference/kubelet-tls-bootstrapping/).  This is only used with the `tls` authentication mode, and is otherwise ignored.
* `settings.kubernetes.eviction-hard`: The signals and thresholds that trigger pod eviction.
  Remember to quote signals (since they all contain ".") and to quote all values.
  * Example user data for setting up eviction hard:
    ```
    [settings.kubernetes.eviction-hard]
    "memory.available" = "15%"
    ```
* `settings.kubernetes.allowed-unsafe-sysctls`: Enables specified list of unsafe sysctls.
  * Example user data for setting up allowed unsafe sysctls:
    ```
    allowed-unsafe-sysctls = ["net.core.somaxconn", "net.ipv4.ip_local_port_range"]
    ```
* `settings.kubernetes.system-reserved`: Resources reserved for system components.
  * Example user data for setting up system reserved:
    ```
    [settings.kubernetes.system-reserved]
    cpu = "10m"
    memory = "100Mi"
    ephemeral-storage= "1Gi"
    ```
* `settings.kubernetes.registry-qps`: The registry pull QPS.
* `settings.kubernetes.registry-burst`: The maximum size of bursty pulls.
* `settings.kubernetes.event-qps`: The maximum event creations per second.
* `settings.kubernetes.event-burst`: The maximum size of a burst of event creations.
* `settings.kubernetes.kube-api-qps`: The QPS to use while talking with kubernetes apiserver.
* `settings.kubernetes.kube-api-burst`: The burst to allow while talking with kubernetes.
* `settings.kubernetes.container-log-max-size`: The maximum size of container log file before it is rotated.
* `settings.kubernetes.container-log-max-files`: The maximum number of container log files that can be present for a container.
* `settings.kubernetes.cpu-manager-policy`: Specifies the CPU manager policy. Possible values are `static` and `none`. Defaults to `none`. If you want to allow pods with certain resource characteristics to be granted increased CPU affinity and exclusivity on the node, you can set this setting to `static`. You should reboot if you change this setting after startup - try `apiclient reboot`.
* `settings.kubernetes.cpu-manager-reconcile-period`: Specifies the CPU manager reconcile period, which controls how often updated CPU assignments are written to cgroupfs. The value is a duration like `30s` for 30 seconds or `1h5m` for 1 hour and 5 minutes.

You can also optionally specify static pods for your node with the following settings.
Static pods can be particularly useful when running in standalone mode.
* `settings.kubernetes.static-pods.<custom identifier>.manifest`: A base64-encoded pod manifest.
* `settings.kubernetes.static-pods.<custom identifier>.enabled`: Whether the static pod is enabled.

For Kubernetes variants in AWS and VMware, the following are set for you automatically, but you can override them if you know what you're doing!
In AWS, [pluto](sources/api/) sets these based on runtime instance information.
In VMware, Bottlerocket uses [netdog](sources/api/) (for `node-ip`) or relies on [default values](sources/models/src/vmware-k8s-1.21/defaults.d/).
* `settings.kubernetes.node-ip`: The IPv4 address of this node.
* `settings.kubernetes.pod-infra-container-image`: The URI of the "pause" container.
* `settings.kubernetes.kube-reserved`: Resources reserved for node components.
  * Bottlerocket provides default values for the resources by [schnauzer](sources/api/):
    * `cpu`: in millicores from the total number of vCPUs available on the instance.
    * `memory`: in mebibytes from the max num of pods on the instance. `memory_to_reserve = max_num_pods * 11 + 255`.
    * `ephemeral-storage`: defaults to `1Gi`.

For Kubernetes variants in AWS, the following settings are set for you automatically by [pluto](sources/api/).
* `settings.kubernetes.max-pods`: The maximum number of pods that can be scheduled on this node (limited by number of available IPv4 addresses)
* `settings.kubernetes.cluster-dns-ip`: Derived from the EKS IPV4 Service CIDR or the CIDR block of the primary network interface.

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

#### Metrics settings

By default, Bottlerocket sends anonymous metrics when it boots, and once every six hours.
This can be disabled by setting `send-metrics` to false.
Here are the metrics settings:

* `settings.metrics.metrics-url`: The endpoint to which metrics will be sent. The default is `https://metrics.bottlerocket.aws/v1/metrics`.
* `settings.metrics.send-metrics`: Whether Bottlerocket will send anonymous metrics.
* `settings.metrics.service-checks`: A list of systemd services that will be checked to determine whether a host is healthy.

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

You can optionally define a `user-data` field with arbitrary base64-encoded data, which will be made available in the container at `/.bottlerocket/host-containers/$HOST_CONTAINER_NAME/user-data` and (since Bottlerocket v1.0.8) `/.bottlerocket/host-containers/current/user-data`.
(It was inspired by instance user data, but is entirely separate; it can be any data your host container feels like interpreting.)

Keep in mind that the default admin container (since Bottlerocket v1.0.6) relies on `user-data` to store SSH keys.  You can set `user-data` to [customize the keys](https://github.com/bottlerocket-os/bottlerocket-admin-container/#authenticating-with-the-admin-container), or you can use it for your own purposes in a custom container.

Here's an example of adding a custom host container with API calls:
```
apiclient set \
   host-containers.custom.source=MY-CONTAINER-URI \
   host-containers.custom.enabled=true \
   host-containers.custom.superpowered=false
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

In addition, all host containers come with persistent storage that survives reboots and container start/stop cycles.
It's available at `/.bottlerocket/host-containers/$HOST_CONTAINER_NAME` and (since Bottlerocket v1.0.8) `/.bottlerocket/host-containers/current`.
The default `admin` host-container, for example, stores its SSH host keys under `/.bottlerocket/host-containers/admin/etc/ssh/`.

There are a few important caveats to understand about host containers:
* They're not orchestrated.  They only start or stop according to that `enabled` flag.
* They run in a separate instance of containerd than the one used for orchestrated containers like Kubernetes pods.
* They're not updated automatically.  You need to update the `source`, disable the container, commit those changes, then re-enable it.
* If you set `superpowered` to true, they'll essentially have root access to the host.

Because of these caveats, host containers are only intended for special use cases.
We use it for the control container because it needs to be available early to give you access to the OS, and we use it for the admin container because it needs high levels of privilege and because you need it to debug when orchestration isn't working.

Be careful, and make sure you have a similar low-level use case before reaching for host containers.

#### Bootstrap containers settings
* `settings.bootstrap-containers.<name>.source`: the image for the container
* `settings.bootstrap-containers.<name>.mode`: the mode of the container, it could be one of `off`, `once` or `always`. See below for a description of modes.
* `settings.bootstrap-containers.<name>.essential`: whether or not the container should fail the boot process, defaults to `false`
* `settings.bootstrap-containers.<name>.user-data`: field with arbitrary base64-encoded data

Bootstrap containers are host containers that can be used to "bootstrap" the host before services like ECS Agent, Kubernetes, and Docker start.

Bootstrap containers are very similar to normal host containers; they come with persistent storage and with optional user data.
Unlike normal host containers, bootstrap containers can't be treated as `superpowered` containers.
However, bootstrap containers do have additional permissions that normal host containers do not have.
Bootstrap containers have access to the underlying root filesystem on `/.bottlerocket/rootfs` as well as to all the devices in the host, and they are set up with the `CAP_SYS_ADMIN` capability.
This allows bootstrap containers to create files, directories, and mounts that are visible to the host.

Bootstrap containers are set up to run after the systemd `configured.target` unit is active.
The containers' systemd unit depends on this target (and not on any of the bootstrap containers' peers) which means that bootstrap containers will not execute in a deterministic order
The boot process will "wait" for as long as the bootstrap containers run.
Bootstrap containers configured with `essential=true` will stop the boot process if they exit code is a non-zero value.

Bootstrap containers have three different modes:

* `always`: with this setting, the container is executed on every boot.
* `off`: the container won't run
* `once`: with this setting, the container only runs on the first boot where the container is defined. Upon completion, the mode is changed to `off`.

Here's an example of adding a bootstrap container with API calls:

```
apiclient set \
   bootstrap-containers.bootstrap.source=MY-CONTAINER-URI \
   bootstrap-containers.bootstrap.mode=once \
   bootstrap-containers.bootstrap.essential=true
```

Here's the same example, but with the settings you'd add to user data:

```
[settings.bootstrap-containers.bootstrap]
source = "MY-CONTAINER-URI"
mode = "once"
essential = true
```

##### Mount propagations in bootstrap and superpowered containers
Both bootstrap and superpowered host containers are configured with the `/.bottlerocket/rootfs/mnt` bind mount that points to `/mnt` in the host, which itself is a bind mount of `/local/mnt`.
This bind mount is set up with shared propagations, so any new mount point created underneath `/.bottlerocket/rootfs/mnt` in any bootstrap or superpowered host container will propagate across mount namespaces.
You can use this feature to configure ephemeral disks attached to your hosts that you may want to use on your workloads.

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

### Kdump Support

Bottlerocket provides support to collect kernel crash dumps whenever the system kernel panics.
Once this happens, both the dmesg log and vmcore dump are stored at `/var/log/kdump`, and the system reboots.

There are a few important caveats about the provided kdump support:

* Currently, only vmware variants have kdump support enabled
* The system kernel will reserve 256MB for the crash kernel, only when the host has at least 2GB of memory; the reserved space won't be available for processes running in the host
* The crash kernel will only be loaded when the `crashkernel` parameter is present in the kernel's cmdline and if there is memory reserved for it

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
* Linux kernel ([background](https://en.wikipedia.org/wiki/Linux), [packaging](packages/kernel-5.4/))
* glibc ([background](https://www.gnu.org/software/libc/), [packaging](packages/glibc/))
* Buildroot as build toolchain ([background](https://buildroot.org/), via the [SDK](https://github.com/bottlerocket-os/bottlerocket-sdk))
* GRUB, with patches for partition flip updates ([background](https://www.gnu.org/software/grub/), [packaging](packages/grub/))
* systemd as init ([background](https://en.wikipedia.org/wiki/Systemd), [packaging](packages/systemd/))
* wicked for networking ([background](https://github.com/openSUSE/wicked), [packaging](packages/wicked/))
* containerd ([background](https://containerd.io/), [packaging](packages/containerd/))
* Kubernetes ([background](https://kubernetes.io/), [packaging](packages/kubernetes-1.19/))
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

### Default Volumes

Bottlerocket operates with two default storage volumes.
* The root device, `/dev/xvda`, holds the active and passive [partition sets](#updates-1).
  It also contains the bootloader, the dm-verity hash tree for verifying the [immutable root filesystem](SECURITY_FEATURES.md#immutable-rootfs-backed-by-dm-verity), and the data store for the Bottlerocket API.
* The data device, `/dev/xvdb`, is used as persistent storage for container images, container orchestration, [host-containers](#Custom-host-containers), and [bootstrap containers](#Bootstrap-containers-settings).
