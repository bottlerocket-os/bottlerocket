# Bottlerocket OS

Welcome to Bottlerocket!

Bottlerocket is a free and open-source Linux-based operating system meant for hosting containers.

To learn more about Bottlerocket, visit the [official Bottlerocket website and documentation](https://bottlerocket.dev/).
Otherwise, if you’re ready to jump right in, read one of our setup guides for running Bottlerocket in [Amazon EKS](QUICKSTART-EKS.md), [Amazon ECS](QUICKSTART-ECS.md), or [VMware](QUICKSTART-VMWARE.md).
If you're interested in running Bottlerocket on bare metal servers, please refer to the [provisioning guide](PROVISIONING-METAL.md) to get started.

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

## Participate in the Community

There are many ways to take part in the Bottlerocket community:

- [Join us on Meetup](https://www.meetup.com/bottlerocket-community/) to hear about the latest Bottlerocket (virtual/in-person) events and community meetings.
  Community meetings are typically every other week.

  Details can be found under the [Events section on Meetup](https://www.meetup.com/bottlerocket-community/events/), and you will receive email notifications if you become a member of the Meetup group. (It's free to join!)

- [Start or join a discussion](https://github.com/bottlerocket-os/bottlerocket/discussions) if you have questions about Bottlerocket.
- If you're interested in contributing, thank you!
  Please see our [contributor's guide](CONTRIBUTING.md).

## Contact us

If you find a security issue, please [contact our security team](https://github.com/bottlerocket-os/bottlerocket/security/policy) rather than opening an issue.

We use GitHub issues to track other bug reports and feature requests.
You can look at [existing issues](https://github.com/bottlerocket-os/bottlerocket/issues) to see whether your concern is already known.

If not, you can select from a few templates and get some guidance on the type of information that would be most helpful.
[Contact us with a new issue here.](https://github.com/bottlerocket-os/bottlerocket/issues/new/choose)

We don't have other communication channels set up quite yet, but don't worry about making an issue or a discussion thread!
You can let us know about things that seem difficult, or even ways you might like to help.

## Variants

To start, we're focusing on the use of Bottlerocket as a host OS in AWS EKS Kubernetes clusters and Amazon ECS clusters.
We’re excited to get early feedback and to continue working on more use cases!

Bottlerocket is architected such that different cloud environments and container orchestrators can be supported in the future.
A build of Bottlerocket that supports different features or integration characteristics is known as a 'variant'.
The artifacts of a build will include the architecture and variant name.
For example, an `x86_64` build of the `aws-k8s-1.24` variant will produce an image named `bottlerocket-aws-k8s-1.24-x86_64-<version>-<commit>.img`.

The following variants support EKS, as described above:

* `aws-k8s-1.24`
* `aws-k8s-1.25`
* `aws-k8s-1.26`
* `aws-k8s-1.27`
* `aws-k8s-1.28`
* `aws-k8s-1.29`
* `aws-k8s-1.30`
* `aws-k8s-1.31`
* `aws-k8s-1.24-nvidia`
* `aws-k8s-1.25-nvidia`
* `aws-k8s-1.26-nvidia`
* `aws-k8s-1.27-nvidia`
* `aws-k8s-1.28-nvidia`
* `aws-k8s-1.29-nvidia`
* `aws-k8s-1.30-nvidia`
* `aws-k8s-1.31-nvidia`

The following variants support ECS:

* `aws-ecs-1`
* `aws-ecs-1-nvidia`
* `aws-ecs-2`
* `aws-ecs-2-nvidia`

We also have variants that are designed to be Kubernetes worker nodes in VMware:

* `vmware-k8s-1.28`
* `vmware-k8s-1.29`
* `vmware-k8s-1.30`
* `vmware-k8s-1.31`

The following variants are designed to be Kubernetes worker nodes on bare metal:

* `metal-k8s-1.28`
* `metal-k8s-1.29`

The following variants are no longer supported:

* All Kubernetes variants using Kubernetes 1.23 and earlier
* Bare metal and VMware variants using Kubernetes 1.27 and earlier

We recommend users replace nodes running these variants with the [latest variant compatible with their cluster](variants/).

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

To see how to provision Bottlerocket on bare metal, see [PROVISIONING-METAL](PROVISIONING-METAL.md).

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

```shell
aws ssm start-session --target INSTANCE_ID --region REGION_CODE
```

With the [default control container](https://github.com/bottlerocket-os/bottlerocket-control-container), you can make [API calls](#api) to configure and manage your Bottlerocket host.
To do even more, read the next section about the [admin container](#admin-container).
You can access the admin container from the control container like this:

```shell
enter-admin-container
```

### Admin container

Bottlerocket has an [administrative container](https://github.com/bottlerocket-os/bottlerocket-admin-container), disabled by default, that runs outside of the orchestrator in a separate instance of containerd.
This container has an SSH server that lets you log in as `ec2-user` using your EC2-registered SSH key.
Outside of AWS, you can [pass in your own SSH keys](https://github.com/bottlerocket-os/bottlerocket-admin-container#authenticating-with-the-admin-container).
(You can easily replace this admin container with your own just by changing the URI; see [Settings](#settings).

To enable the container, you can change the setting in user data when starting Bottlerocket, for example EC2 instance user data:

```toml
[settings.host-containers.admin]
enabled = true
```

If Bottlerocket is already running, you can enable the admin container from the default [control container](#control-container) like this:

```shell
enable-admin-container
```

Or you can start an interactive session immediately like this:

```shell
enter-admin-container
```

If you're using a custom control container, or want to make the API calls directly, you can enable the admin container like this instead:

```shell
apiclient set host-containers.admin.enabled=true
```

Once you've enabled the admin container, you can either access it through SSH or execute commands from the control container like this:

```shell
apiclient exec admin bash
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

```shell
apiclient update check
```

If an update is available, it will show up in the `chosen_update` field.
The `available_updates` field will show the full list of available versions, including older versions, because Bottlerocket supports safely rolling back.

To apply the latest update:

```shell
apiclient update apply
```

The next time you reboot, you'll start up in the new version, and system configuration will be automatically [migrated](sources/api/migration/).
To reboot right away:

```shell
apiclient reboot
```

If you're confident about updating, the `apiclient update apply` command has `--check` and `--reboot` flags to combine the above actions, so you can accomplish all of the above steps like this:

```shell
apiclient update apply --check --reboot
```

See the [apiclient documentation](sources/api/apiclient/) for more details.

### Update rollback

The system will automatically roll back if it's unable to boot.
If the update is not functional for a given container workload, you can do a manual rollback:

```shell
signpost rollback-to-inactive
reboot
```

This doesn't require any external communication, so it's quicker than `apiclient`, and it's made to be as reliable as possible.

## Settings

Here we'll describe the settings you can configure on your Bottlerocket instance, and how to do it.

(API endpoints are defined in our [OpenAPI spec](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/api/openapi.yaml) if you want more detail.)

### Interacting with settings

#### Using the API client

You can see the current settings with an API request:

```shell
apiclient get settings
```

This will return all of the current settings in JSON format.
For example, here's an abbreviated response:

```json
{"motd": "...", {"kubernetes": {}}}
```

You can change settings like this:

```shell
apiclient set motd="hi there" kubernetes.node-labels.environment=test
```

You can also use a JSON input mode to help change many related settings at once, and a "raw" mode if you want more control over how the settings are committed and applied to the system.
See the [apiclient README](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/api/apiclient/) for details.

#### Using user data

If you know what settings you want to change when you start your Bottlerocket instance, you can send them in the user data.

In user data, we structure the settings in TOML form to make things a bit simpler.
Here's the user data to change the message of the day setting, as we did in the last section:

```toml
[settings]
motd = "my own value!"
```

If your user data is over the size limit of the platform (e.g. 16KiB for EC2) you can compress the contents with gzip.
(With [aws-cli](https://aws.amazon.com/cli/), you can use `--user-data fileb:///path/to/gz-file` to pass binary data.)

### Description of settings

Here we'll describe each setting you can change.

**Note:** You can see the default values (for any settings that are not generated at runtime) by looking in the `defaults.d` directory for a variant, for example [aws-ecs-2](sources/models/src/aws-ecs-2/defaults.d/).

When you're sending settings to the API, or receiving settings from the API, they're in a structured JSON format.
This allows modification of any number of keys at once.
It also lets us ensure that they fit the definition of the Bottlerocket data model - requests with invalid settings won't even parse correctly, helping ensure safety.

Here, however, we'll use the shortcut "dotted key" syntax for referring to keys.
This is used in some API endpoints with less-structured requests or responses.
It's also more compact for our needs here.

In this format, "settings.kubernetes.cluster-name" refers to the same key as in the JSON `{"settings": {"kubernetes": {"cluster-name": "value"}}}`.

**NOTE:** [bottlerocket.dev](https://bottlerocket.dev/en/os/latest/#/api/settings/) now contains a complete, versioned setting reference.
This documents retains the headings below for existing link and bookmark compatability.
Please update your bookmarks and check out [bottlerocket.dev](https://bottlerocket.dev/) for future updates to the setting reference.

#### Top-level settings

See the [`settings.motd` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/motd/).

#### Kubernetes settings

See the [`settings.kubernetes.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/kubernetes/).

#### Amazon ECS settings

See the [`settings.ecs.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/ecs/).

#### CloudFormation signal helper settings

See the [`settings.cloudformation.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/cloudformation/).

#### Auto Scaling group settings

See the [`settings.autoscaling.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/autoscaling/).

#### OCI Hooks settings

See the [`settings.oci-hooks.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/oci-hooks/).

#### OCI Defaults settings

See the [`settings.oci-defaults.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/oci-defaults/).

##### OCI Defaults: Capabilities

See the ["Capabilities Settings" section in the `settings.oci-defaults.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/oci-defaults/).

##### OCI Defaults: Resource Limits

See the ["Resource Limits Settings" section in the `settings.oci-defaults.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/oci-defaults/).
  
#### Container image registry settings

See the [`settings.container-registry.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/container-registry/).

#### Container runtime settings

See the [`settings.container-runtime.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/container-runtime/).

#### Updates settings

See the [`settings.updates.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/updates/).

#### Network settings

See the [`settings.network.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/network/).

##### Proxy settings

See the ["Proxy Settings" section in the `settings.networks.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/network/).
  
#### Metrics settings

See the [`settings.metrics.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/metrics/).

#### Time settings

See the [`settings.ntp.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/ntp/).

#### Kernel settings

See the [`settings.kernel.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/kernel/).

#### Boot-related settings

See the [`settings.boot.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/boot/).

#### Custom CA certificates settings

See the [`settings.pki.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/pki/).

#### Host containers settings

See the [`settings.host-containers.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/host-containers/).

##### Custom host containers

See the [Host Containers documentation](https://bottlerocket.dev/en/os/latest/#/concepts/host-containers/).

#### Bootstrap commands settings

See the [`settings.bootstrap-commands.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/bootstrap-commands/) as well as the [Bootstrap Commands documentation](https://bottlerocket.dev/en/os/latest/#/concepts/bootstrap-commands/)

#### Bootstrap containers settings

See the [`settings.bootstrap-containers.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/bootstrap-containers/) as well as the [Bootstrap Containers documentation](https://bottlerocket.dev/en/os/latest/#/concepts/bootstrap-containers/)

##### Mount propagations in bootstrap and superpowered containers

Both bootstrap and superpowered host containers are configured with the `/.bottlerocket/rootfs/mnt` bind mount that points to `/mnt` in the host, which itself is a bind mount of `/local/mnt`.
This bind mount is set up with shared propagations, so any new mount point created underneath `/.bottlerocket/rootfs/mnt` in any bootstrap or superpowered host container will propagate across mount namespaces.
You can use this feature to configure ephemeral disks attached to your hosts that you may want to use on your workloads.

#### Platform-specific settings

Platform-specific settings are automatically set at boot time by [early-boot-config](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/early-boot-config/early-boot-config) based on metadata available on the running platform.
They can be overridden for testing purposes in [the same way as other settings](#interacting-with-settings).

##### AWS-specific settings

See the [`settings.aws.*` reference](https://bottlerocket.dev/en/os/latest/#/api/settings/aws/).

### Logs

You can use `logdog` through the [admin container](#admin-container) to obtain an archive of log files from your Bottlerocket host.

For a list of what is collected, see the logdog [command list](https://github.com/bottlerocket-os/bottlerocket-core-kit/blob/develop/sources/logdog/src/log_request.rs).

#### Generating logs

SSH to the Bottlerocket host or `apiclient exec admin bash` to access the admin container, then run:

```shell
sudo sheltie
logdog
```

This will write an archive of the logs to `/var/log/support/bottlerocket-logs.tar.gz`.
This archive is accessible from host containers at `/.bottlerocket/support`.

#### Fetching logs

There are multiple methods to retrieve the generated log archive.

- **Via SSH if already enabled**

    Once you have exited from the Bottlerocket host, run a command like:

    ```shell
    ssh -i YOUR_KEY_FILE \
    ec2-user@YOUR_HOST \
    "cat /.bottlerocket/support/bottlerocket-logs.tar.gz" > bottlerocket-logs.tar.gz
    ```

- **With `kubectl get` if running Kubernetes**

    ```shell
    kubectl get --raw \
    "/api/v1/nodes/NODE_NAME/proxy/logs/support/bottlerocket-logs.tar.gz" > bottlerocket-logs.tar.gz
    ```

- **Using [SSH over SSM](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-getting-started-enable-ssh-connections.html) if your instance isn't accessible through SSH or Kubernetes**

### Kdump Support

Bottlerocket provides support to collect kernel crash dumps whenever the system kernel panics.
Once this happens, both the dmesg log and vmcore dump are stored at `/var/log/kdump`, and the system reboots.

There are a few important caveats about the provided kdump support:

* Currently, only vmware variants have kdump support enabled
* The system kernel will reserve 256MB for the crash kernel, only when the host has at least 2GB of memory; the reserved space won't be available for processes running in the host
* The crash kernel will only be loaded when the `crashkernel` parameter is present in the kernel's cmdline and if there is memory reserved for it

### NVIDIA GPUs Support

Bottlerocket's `nvidia` variants include the required packages and configurations to leverage NVIDIA GPUs.
Currently, the following NVIDIA driver versions are supported in Bottlerocket:

* 470.X
* 515.X

The official AMIs for these variants can be used with EC2 GPU-equipped instance types such as: `p2`, `p3`, `p4`, `g3`, `g4dn`, `g5` and `g5g`.
Note that older instance types, such as `p2`, are not supported by NVIDIA driver `515.X` and above.
You need to make sure you select the appropriate AMI depending on the instance type you are planning to use.
Please see [QUICKSTART-EKS](QUICKSTART-EKS.md#aws-k8s--nvidia-variants) for further details about Kubernetes variants, and [QUICKSTART-ECS](QUICKSTART-ECS.md#aws-ecs--nvidia-variants) for ECS variants.

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

* Linux kernel ([background](https://en.wikipedia.org/wiki/Linux), [5.10 packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/kernel-5.10/), [5.15 packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/kernel-5.15/))
* glibc ([background](https://www.gnu.org/software/libc/), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/glibc/))
* Buildroot as build toolchain ([background](https://buildroot.org/), via the [SDK](https://github.com/bottlerocket-os/bottlerocket-sdk))
* GRUB, with patches for partition flip updates ([background](https://www.gnu.org/software/grub/), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/grub/))
* systemd as init ([background](https://en.wikipedia.org/wiki/Systemd), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/systemd/))
* wicked for networking ([background](https://github.com/openSUSE/wicked), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/wicked/))
* containerd ([background](https://containerd.io/), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/containerd/))
* Kubernetes ([background](https://kubernetes.io/), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/kubernetes-1.30/))
* aws-iam-authenticator ([background](https://github.com/kubernetes-sigs/aws-iam-authenticator), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/aws-iam-authenticator/))
* Amazon ECS agent ([background](https://github.com/aws/amazon-ecs-agent), [packaging](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/ecs-agent/))

For further documentation or to see the rest of the packages, see the [packaging directory](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/packages/).

### Updates

The Bottlerocket image has two identical sets of partitions, A and B.
When updating Bottlerocket, the partition table is updated to point from set A to set B, or vice versa.

We also track successful boots, and if there are failures it will automatically revert back to the prior working partition set.

The update process uses images secured by [TUF](https://theupdateframework.github.io/).
For more details, see the [update system documentation](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/updater).

### API

There are two main ways you'd interact with a production Bottlerocket instance.
(There are a couple more [exploration](#exploration) methods above for test instances.)

The first method is through a container orchestrator, for when you want to run or manage containers.
This uses the standard channel for your orchestrator, for example a tool like `kubectl` for Kubernetes.

The second method is through the Bottlerocket API, for example when you want to configure the system.

There's an HTTP API server that listens on a local Unix-domain socket.
Remote access to the API requires an authenticated transport such as SSM's RunCommand or Session Manager, as described above.
For more details, see the [apiserver documentation](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/api/apiserver/).

The [apiclient](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/api/apiclient/) can be used to make requests.
They're just HTTP requests, but the API client simplifies making requests with the Unix-domain socket.

To make configuration easier, we have [early-boot-config](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/early-boot-config/early-boot-config), which can send an API request for you based on instance user data.
If you start a virtual machine, like an EC2 instance, it will read TOML-formatted Bottlerocket configuration from user data and send it to the API server.
This way, you can configure your Bottlerocket instance without having to make API calls after launch.

See [Settings](#settings) above for examples and to understand what you can configure.

You can also access host containers through the API using [apiclient exec](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/api/apiclient#exec-mode).

The server and client are the user-facing components of the API system, but there are a number of other components that work together to make sure your settings are applied, and that they survive upgrades of Bottlerocket.

For more details, see the [API system documentation](https://github.com/bottlerocket-os/bottlerocket-core-kit/tree/develop/sources/api).

### Default Volumes

Bottlerocket operates with two default storage volumes.

* The root device, holds the active and passive [partition sets](#updates-1).
  It also contains the bootloader, the dm-verity hash tree for verifying the [immutable root filesystem](SECURITY_FEATURES.md#immutable-rootfs-backed-by-dm-verity), and the data store for the Bottlerocket API.
* The data device is used as persistent storage for container images, container orchestration, [host-containers](#Custom-host-containers), and [bootstrap containers](#Bootstrap-containers-settings).
  The operating system does not typically make changes to this volume during regular updates, though changes to upstream software such as containerd or kubelet could result in changes to their stored data.
  This device (mounted to `/local` on the host) can be used for application storage for orchestrated workloads; however, we recommend using an additional volume if possible for such cases.
  See [this section of the Security Guidance documentation](./SECURITY_GUIDANCE.md#limit-access-to-system-mounts) for more information.

On boot Bottlerocket will increase the data partition size to use all of the data device.
If you increase the size of the device, you can reboot Bottlerocket to extend the data partition.
If you need to extend the data partition without rebooting, have a look at this [discussion](https://github.com/bottlerocket-os/bottlerocket/discussions/2011).
