# Building Bottlerocket

If you'd like to build your own image instead of relying on an Amazon-provided image, follow these steps.
You can skip to the [setup guide for Kubernetes](QUICKSTART-EKS.md) or the [setup guide for Amazon ECS](QUICKSTART-ECS.md) to use an existing image in Amazon EC2.
(We're still working on other use cases!)

## Build an image

### Dependencies

#### System Requirements

The build process artifacts and resulting images can consume in excess of 80GB in the local directory.

The build process is also fairly demanding on your CPU, since we build all included software from scratch.
(The first time.  Package builds are cached, and only changes are built afterward.)
The build scales well to 32+ cores.
The first time you build, the fastest machines can take about 12 minutes while slower machines with only a couple cores can take 3-4 hours.

#### Linux

The build system requires certain operating system packages to be installed.

Ensure the following OS packages are installed:

##### Ubuntu

```
apt install build-essential openssl libssl-dev pkg-config liblz4-tool
```

##### Fedora

```
yum install make automake gcc openssl openssl-devel pkg-config lz4 perl-FindBin perl-lib
```


#### Rust

The build system is based on the Rust language.
We recommend you install the latest stable Rust using [rustup](https://rustup.rs/), either from the official site or your development host's package manager.
Rust 1.51.0 or higher is required.

To organize build tasks, we use [cargo-make](https://sagiegurari.github.io/cargo-make/).
To get it, run:

```
cargo install cargo-make
```

#### Docker

Bottlerocket uses [Docker](https://docs.docker.com/install/#supported-platforms) to orchestrate package and image builds.

We recommend Docker 20.10.10 or later.
Builds rely on Docker's integrated BuildKit support, which has received many fixes and improvements in newer versions.
The default seccomp policy of older versions of Docker do not support the `clone3` syscall in recent versions of Fedora or Ubuntu, on which the Bottlerocket SDK is based.

You'll need to have Docker installed and running, with your user account added to the `docker` group.
Docker's [post-installation steps for Linux](https://docs.docker.com/install/linux/linux-postinstall/) will walk you through that.

> Note: If you're on a newer Linux distribution using the unified cgroup hierarchy with cgroups v2, you may need to disable it to work with current versions of runc.
> You'll know this is the case if you see an error like `docker: Error response from daemon: OCI runtime create failed: this version of runc doesn't work on cgroups v2: unknown.`
> Set the kernel parameter `systemd.unified_cgroup_hierarchy=0` in your boot configuration (e.g. GRUB) and reboot.

### Build process

To build an image, run:

```
cargo make
```

This will build an image for the default variant, `aws-k8s-1.21`.
All packages will be built in turn, and then compiled into an `img` file in the `build/images/` directory.

The version number in [Release.toml](Release.toml) will be used in naming the file, and will be used inside the image as the release version.
If you're planning on [publishing your build](PUBLISHING.md), you may want to change the version.

To build an image for a different variant, run:

```
cargo make -e BUILDSYS_VARIANT=my-variant-here
```

To build an image for a different architecture, run:

```
cargo make -e BUILDSYS_ARCH=my-arch-here
```

(You can use variant and arch arguments together, too.)

#### Package licenses

Most packages will include license files extracted from upstream source archives.
However, in some rare cases there are multiple licenses that could apply to a package.
Bottlerocket's build system uses the `Licenses.toml` file in conjunction with the `licenses` directory to configure the licenses used for such special packages.
Here is an example of a simple `Licenses.toml` configuration file:

```toml
[package]
spdx-id = "SPDX-ID"
licenses = [
  { path = "the-license.txt" }
]
```

In the previous example, it is expected that the file `the-license.txt` is present in `licenses`.
You can retrieve the licenses from a remote endpoint, or the local filesystem if you specify the `license-url` field:

```toml
[package]
spdx-id = "SPDX-ID AND SPDX-ID-2" # Package with multiple licenses
licenses = [
  # This file is copied from a file system, and will be saved as `path`
  { license-url = "file:///path/to/spdx-id-license.txt", path = "spdx-id-license.txt" },
  # This file is fetched from an https endpoint, and will be saved as `path`
  { license-url = "https://localhost/spdx-id-license-v2.txt", path = "spdx-id-license-2.txt" }
]
```

#### NVIDIA variants

If you want to build any of the NVIDIA variants, you can follow these steps to prepare a `Licenses.toml` file using the [License for customer use of NVIDIA software](https://www.nvidia.com/en-us/drivers/nvidia-license/):

1. Create a `Licenses.toml` file in your Bottlerocket root directory, with the following content:

```toml
[nvidia]
spdx-id = "LicensesRef-NVIDIA-Customer-Use"
licenses = [
  { path = "LICENSE", license-url = "https://www.nvidia.com/en-us/drivers/nvidia-license/" }
]
```

2. Fetch the licenses with this command:

```shell
cargo make -e BUILDSYS_UPSTREAM_LICENSE_FETCH=true fetch-licenses
```

3. Build your image, setting the `BUILDSYS_UPSTREAM_SOURCE_FALLBACK` flag to `true`, if you haven't cached the driver's sources:

```shell
K8S_VERSION=1.22
cargo make \
  -e BUILDSYS_VARIANT=aws-k8s-${K8S_VERSION}-nvidia \
  -e BUILDSYS_UPSTREAM_SOURCE_FALLBACK="true"
```

### Register an AMI

To use the image in Amazon EC2, we need to register the image as an AMI.

To do this, you'll need to have your AWS account credentials setup on your system.
There are lots of ways to do this; one method is using [the `aws` CLI](https://aws.amazon.com/cli/) via its `configure` command with your user's access and secret keys.
If you're using an EC2 instance, the [EC2 instance's IAM role](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/iam-roles-for-amazon-ec2.html) will be used automatically if available.

For a simple start, pick an [EC2 region](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html#concepts-available-regions), then run:

```
cargo make -e PUBLISH_REGIONS=your-region-here ami
```

Note that the task ("ami") must come **after** the arguments to `cargo make` that are specified with `-e`.

Your new AMI ID will be printed after it's registered.

If you built your image for a different architecture or variant, just use the same arguments here:

```
cargo make -e PUBLISH_REGIONS=your-region-here -e BUILDSYS_VARIANT=my-variant-here ami
```

(There's a lot more detail on building and managing AMIs in the [PUBLISHING](PUBLISHING.md) guide.)

## Use your image

See any of the setup guides tailored to the various execution environments for information on running Bottlerocket images:

* [Setup guide for Kubernetes](QUICKSTART-EKS.md)
* [Setup guide for Amazon ECS](QUICKSTART-ECS.md)
* [Setup guide for VMware](QUICKSTART-VMWARE.md)
* [Setup guide for QEMU/KVM](QUICKSTART-LOCAL.md)

## Publish your image

See the [PUBLISHING](PUBLISHING.md) guide for information on deploying Bottlerocket images and repositories.

## Building out-of-tree kernel modules

To further extend Bottlerocket, you may want to build extra kernel modules.
The specifics of building an out-of-tree module will vary by project, but the first step is to download the "kmod kit" that contains the kernel headers and toolchain you'll need to use.

### Downloading the kmod kit

kmod kits are included in the official Bottlerocket repos starting with Bottlerocket v1.0.6.
Let's say you want to download the kit for building x86_64 modules for v1.7.0 and variant aws-k8s-1.21.

First, you need tuftool:
```bash
cargo install tuftool
```

Next, you need the Bottlerocket root role, which is used by tuftool to verify the kmod kit.
This will download and verify the root role itself:
```bash
curl -O "https://cache.bottlerocket.aws/root.json"
sha512sum -c <<<"b81af4d8eb86743539fbc4709d33ada7b118d9f929f0c2f6c04e1d41f46241ed80423666d169079d736ab79965b4dd25a5a6db5f01578b397496d49ce11a3aa2  root.json"
```

Next, set your desired parameters, and download the kmod kit:
```bash
ARCH=x86_64
VERSION=v1.7.0
VARIANT=aws-k8s-1.21
OUTDIR="${VARIANT}-${VERSION}"

tuftool download "${OUTDIR}" --target-name ${VARIANT}-${ARCH}-kmod-kit-${VERSION}.tar.xz \
   --root ./root.json \
   --metadata-url "https://updates.bottlerocket.aws/2020-07-07/${VARIANT}/${ARCH}/" \
   --targets-url "https://updates.bottlerocket.aws/targets/"
```

### Using the kmod kit

To use the kmod kit, extract it, and update your PATH to use its toolchain:
```bash
tar xf "${VARIANT}-${ARCH}-kmod-kit-${VERSION}.tar.xz"

export CROSS_COMPILE="${ARCH}-bottlerocket-linux-musl-"
export KERNELDIR="${PWD}/${VARIANT}-${ARCH}-kmod-kit-${VERSION}/kernel-devel
export PATH="${PWD}/${VARIANT}-${ARCH}-kmod-kit-${VERSION}/toolchain/usr/bin:${PATH}"
```

Now you can compile modules against the kernel headers in `${KERNELDIR}`.
