# Security Features

## Goals

Bottlerocket has the following high-level security goals.
They are listed in priority order.

1. Harden the OS against persistent threats.
2. Mitigate the impact of vulnerabilities in the OS.
3. Protect containers from other containers.

We will add and enhance security features over time based on these goals.
Please see our [security roadmap](https://github.com/orgs/bottlerocket-os/projects/1?card_filter_query=label%3Asecurity) for more details.

## Overview

| Feature                                                                           | Version |
| :-------------------------------------------------------------------------------- | :-----: |
| [Automated security updates](#automated-security-updates)                         |  0.3.0  |
| [Immutable rootfs backed by dm-verity](#immutable-rootfs-backed-by-dm-verity)     |  0.3.0  |
| [Stateless tmpfs for /etc](#stateless-tmpfs-for-etc)                              |  0.3.0  |
| [No shell or interpreters installed](#no-shell-or-interpreters-installed)         |  0.3.0  |
| [Executables built with hardening flags](#executables-built-with-hardening-flags) |  0.3.0  |
| [SELinux enabled in enforcing mode](#selinux-enabled-in-enforcing-mode)           |  0.3.0  |

The version listed indicates the first release of Bottlerocket that included the feature.
Features may evolve or improve over time.

## Details

### Automated security updates

Bottlerocket is designed for reliable security updates that can be applied through automation.

This is achieved through the following mechanisms:
* Two partition sets and an active/passive flip to swap OS images
* Declarative API with modeled settings for runtime configuration
* Variants to silo backwards-incompatible or breaking changes

Using partition sets and modeled settings removes the dependency on correct local state for reliable updates.
There is no package manager database or shared filesystem tree that can become corrupted and make the process non-deterministic.

Our philosophy for variants is that the right time for an unexpected major version update to the kernel or orchestrator agent is "never".

#### Kubernetes variants

Each Kubernetes variant will be supported for approximately one year after it is first released.
This follows the [Amazon EKS](https://docs.aws.amazon.com/eks/latest/userguide/kubernetes-versions.html) support policy.
Newer LTS releases of the kernel may be introduced in newer variants, but not in older ones.

We provide [a Kubernetes operator](https://github.com/bottlerocket-os/bottlerocket-update-operator) for automated updates to Bottlerocket.
We recommend deploying it on your Kubernetes clusters.

#### ECS variant

We currently have a single variant for Amazon ECS.
Newer LTS releases of the kernel may be introduced in newer variants, but not in the current one.

We provide [an updater](https://github.com/bottlerocket-os/bottlerocket-ecs-updater) for automated updates to Bottlerocket.
We recommend deploying it on your ECS clusters.


### Immutable rootfs backed by dm-verity

Bottlerocket uses [dm-verity](https://www.kernel.org/doc/html/latest/admin-guide/device-mapper/verity.html) for its root filesystem image.
This provides transparent integrity checking of the underlying block device using a cryptographic digest.

The root filesystem is marked as read-only and cannot be directly modified by userspace processes.
This protects against some container escape vulnerabilities such as [CVE-2019-5736](https://www.openwall.com/lists/oss-security/2019/02/11/2).

The kernel is configured to restart if corruption is detected.
That allows the system to fail closed if the underlying block device is unexpectedly modified.

Although this provides a powerful layer of protection, it is **incomplete**.
An attacker with full access to the block device could alter both the verity metadata and the contents of the root filesystem.

### Stateless tmpfs for /etc

Bottlerocket uses [tmpfs](https://www.kernel.org/doc/Documentation/filesystems/tmpfs.txt), a memory-backed filesystem, for /etc.

Direct modification of system configuration files such as `/etc/resolv.conf` or `/etc/containerd/config.toml` is not supported.
This makes OS updates more reliable, as it is not necessary to account for local edits that might have changed the behavior of system services.
It also makes it harder for an attacker to modify these files in a way that persists across a reboot.

There are two supported ways to configure the OS in the presence of these restrictions.

The first is through the API.
Settings are persisted across reboot and migrated through OS upgrades.
They are used to render system configuration files from templates on every boot.

The second is by using containers.
Specifications such as [CNI](https://github.com/containernetworking/cni) and [CSI](https://github.com/container-storage-interface/spec) provide ways to configure networking and storage devices.
Containers written to these specifications can be deployed to nodes using orchestrator-specific mechanisms like [DaemonSets](https://kubernetes.io/docs/concepts/workloads/controllers/daemonset/).

All variants will include a secondary filesystem for local storage.
It will be mounted at `/local` with bind mounts for `/var` and `/opt`.
Modifications to this area will survive an OS update or a reboot.

### No shell or interpreters installed

Bottlerocket does not have a shell installed in non-developer builds.
Interpreted languages such as Python are not installed or even available as packages.

Shells and interpreters enable administrators to write code that combines other programs on the system in new ways.
However, these properties can also be exploited by an attacker to pivot from a vulnerability that grants local code execution.

The lack of a shell also serves as a forcing function to ensure that new code for the OS is written in a preferred language such as Rust or Go.
These languages offer built-in protection against memory safety issues such as buffer overflows.

### Executables built with hardening flags

The GCC cross-compilers in the [Bottlerocket SDK](https://github.com/bottlerocket-os/bottlerocket-sdk) are built with these options:
* `--enable-default-pie` for `-fPIE` and `-pie` by default
* `--enable-default-ssp` for `-fstack-protector-strong` by default

Position-independent executables (PIE) have their address space randomized on every execution.
This makes addresses harder to predict for an attacker that attempts to exploit a memory corruption vulnerability.

The stack protector feature enables stack canaries to detect stack overflow and abort the program if it occurs.
The "strong" version enables it for additional functions.

All C and C++ programs are compiled with the following options:
* `-Wall` to warn about questionable constructs
* `-Werror=format-security` to warn about unsafe uses of format functions
* `-Wp,-D_FORTIFY_SOURCE=2` for runtime error checks in libc
* `-Wp,-D_GLIBCXX_ASSERTIONS` for runtime error checks in libstdc++
* `-fstack-clash-protection` for stack overflow detection

Although C and C++ lack the memory safety of Go and Rust, these options add a layer of defense during build and execution.

All binaries are linked with the following options:
* `-Wl,-z,relro` to mark segments read-only after relocation
* `-Wl,-z,now` to resolve all symbols at load time

Together these enable [full RELRO support](https://www.redhat.com/en/blog/hardening-elf-binaries-using-relocation-read-only-relro) which makes [ROP](https://en.wikipedia.org/wiki/Return-oriented_programming) attacks more difficult to execute.

**Note:** Certain variants, such as the ones for NVIDIA, include precompiled binaries that may not have been built with these hardening flags.

### SELinux enabled in enforcing mode

Bottlerocket enables SELinux by default, sets it to enforcing mode, and loads the policy during boot.
There is no way to disable it.

SELinux is a Linux Security Module (LSM) that provides a mechanism for mandatory access control (MAC).
Processes that run as root with full capabilities are still subject to the mandatory policy restrictions.
Host containers with the `superpowered = true` flag set are an exception, and will run in the permissive `super_t` domain.
Permissive means that actions that are not allowed by policy will be logged but not blocked.

The policy in Bottlerocket has the following objectives:
1) Prevent most components from directly modifying the API settings.
2) Block most components from modifying the container archives saved on disk.
3) Stop containers from directly modifying the layers for other running containers.

The policy is currently aimed at hardening the OS against persistent threats.
Future enhancements to the policy will focus on mitigating the impact of OS vulnerabilities, and protecting containers from other containers.
