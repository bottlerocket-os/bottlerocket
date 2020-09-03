# Building Bottlerocket

If you'd like to build your own image instead of relying on an Amazon-provided image, follow these steps.
You can skip to the [setup guide for Kubernetes](QUICKSTART-EKS.md) or the [setup guide for Amazon ECS](QUICKSTART-ECS.md) to use an existing image in Amazon EC2.
(We're still working on other use cases!)

## Build an image

### Dependencies

#### Operating System Packages

The Rust tooling requires certain operating system packages to be installed in order to complete builds.  
Error messages from the Rust tooling do not clearly identify the issues.

Ensure the following OS packages are installed:

- `build-essential`
- `openssl-dev` or `libssl-dev`
- `pkg-config`

#### Rust

The build system is based on the Rust language.
We recommend you install the latest stable Rust using [rustup](https://rustup.rs/), either from the official site or your development host's package manager.

To organize build tasks, we use [cargo-make](https://sagiegurari.github.io/cargo-make/).
We also use [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) during the build process.
To get these, run:

```
cargo install cargo-make
cargo install cargo-deny --version 0.6.2
```

#### Docker

Bottlerocket uses [Docker](https://docs.docker.com/install/#supported-platforms) to orchestrate package and image builds.

We recommend Docker 19.03 or later.
Builds rely on Docker's integrated BuildKit support, which has received many fixes and improvements in newer versions.

You'll need to have Docker installed and running, with your user account added to the `docker` group.
Docker's [post-installation steps for Linux](https://docs.docker.com/install/linux/linux-postinstall/) will walk you through that.

### Build process

To build an image, run:

```
cargo make
```

This will build an image for the default variant, `aws-k8s-1.17`.
All packages will be built in turn, and then compiled into an `img` file in the `build/` directory.

To build an image for a different variant, run:

```
cargo make -e BUILDSYS_VARIANT=my-variant-here
```

### Register an AMI

To use the image in Amazon EC2, we need to register the image as an AMI.
The `bin/amiize.sh` script does this for you.

The script has some assumptions about your setup, in particular that you:
  * have [aws-cli](https://aws.amazon.com/cli/) set up, and that its default profile can create and control EC2 resources
  * have [coldsnap](https://github.com/awslabs/coldsnap/) installed to upload snapshots
  * have a few other common tools installed, like `jq` and `du`

First, decompress the images.
(Note: these filenames assume an `x86_64` architecture and `aws-k8s-1.17` [variant](README.md).)

```
lz4 -d build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64.img.lz4 && \
lz4 -d build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64-data.img.lz4
```

Next, register an AMI:

```
bin/amiize.sh --name YOUR-AMI-NAME-HERE \
              --arch x86_64 \
              --region us-west-2 \
              --root-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64.img \
              --data-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64-data.img
```

Your new AMI ID will be printed at the end.

## Use your image

See the [setup guide for Kubernetes](QUICKSTART-EKS.md) or the [setup guide for Amazon ECS](QUICKSTART-ECS.md) for information on running Bottlerocket images.
