# Building Bottlerocket

If you'd like to build your own image instead of relying on an Amazon-provided image, follow these steps.
You can skip to the [setup guide](QUICKSTART.md) to use an existing image in Amazon EC2.
(We're still working on other use cases!)

## Build an image

### Dependencies

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

All packages will be built in turn, and then compiled into an `img` file in the `build/` directory.

### Register an AMI

To use the image in Amazon EC2, we need to register the image as an AMI.
The `bin/amiize.sh` script does this for you.

The script has some assumptions about your setup, in particular that you:
  * have [aws-cli v1](https://aws.amazon.com/cli/) set up, and that its default profile can create and control EC2 resources
  * have an SSH key that's registered with EC2 and is available to `ssh` (for example, loaded into `ssh-agent`)
  * have a few other common tools installed, like `jq`, `du`, and `rsync`

First, decompress the images.
(Note: these filenames assume an `x86_64` architecture and `aws-k8s-1.17` [variant](README.md).)

```
lz4 -d build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64.img.lz4 && \
lz4 -d build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64-data.img.lz4
```

Next, register an AMI:

```
bin/amiize.sh --name YOUR-AMI-NAME-HERE \
              --ssh-keypair YOUR-EC2-SSH-KEYPAIR-NAME-HERE \
              --root-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64.img \
              --data-image build/images/x86_64-aws-k8s-1.17/latest/bottlerocket-aws-k8s-1.17-x86_64-data.img \
              --region us-west-2 \
              --instance-type m3.xlarge \
              --arch x86_64 \
              --worker-ami ami-08d489468314a58df \
              --user-data 'I2Nsb3VkLWNvbmZpZwpyZXBvX3VwZ3JhZGU6IG5vbmUK'
```

Your new AMI ID will be printed at the end.

The amiize script starts an EC2 instance, which it uses to write the image to a new EBS volume.
It then registers this EBS volume as an AMI and terminates the instance.
In the example command above, the `--worker-ami` is an Amazon Linux AMI, and the `--user-data` disables updates at boot to speed up registration.
Make sure you use an up-to-date worker AMI.

## Use your image

See the [setup guide](QUICKSTART.md) for information on running Bottlerocket images.
