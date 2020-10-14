## Bottlerocket terms

* [**block-party**](sources/updater/block-party): A library that helps retrieve information about Linux block devices.
* [**bork**](sources/api/bork): A setting generator called by sundog to generate the random seed for updog, determining where the host falls in the update order.
* [**buildsys**](tools/buildsys): A build tool that runs package and image builds inside containers.
  cargo-make starts the build of each package, each of which calls buildsys, which in turn starts a Docker-based build using the SDK image.
* [**corndog**](sources/api/corndog): A program that sets kernel sysctl values based on API settings.
* [**early-boot-config**](sources/api/early-boot-config): A program run at boot to read platform-specific data, such as EC2 user data, and send requested configuration to the API.
* **gptprio:** A structure of bits in GPT partition headers that specifies priority, tries remaining, and whether the partition booted successfully before.
  signpost sets these and GRUB uses them to determine which partition set to boot.
* [**ghostdog**](sources/ghostdog): A program used to manage ephemeral disks.
* [**growpart**](sources/growpart): A program used to expand disk partitions upon boot.
* **host containers**: Containers that run in a separate instance of containerd than "user" containers spawned by an orchestrator (e.g. Kubernetes).
  Used for system maintenance and connectivity.
* [**host-ctr**](sources/host-ctr): The program started by `host-containers@.service` for each host container.
  Its job is to start the specified host container on the “host” instance of containerd, which is separate from the “user” instance of containerd used for Kubernetes pods.
* [**model**](sources/models): The API system has a data model defined for each variant, and this model is used by other programs to serialize and deserialize requests while maintaining safety around data types.
* [**netdog**](sources/api/netdog): A program called by wicked to retrieve and write out network configuration from DHCP.
* [**pluto**](sources/api/pluto): A setting generator called by sundog to find networking settings required by Kubernetes.
* [**schnauzer**](sources/api/schnauzer): A setting generator called by sundog to build setting values that contain template variables referencing other settings.
* **setting generator**: A binary that generates the default value of a setting.
* [**signpost**](sources/updater/signpost): A program used to manipulate the GPT header of the OS disk; fields in the header are used by GRUB to determine the partition set we should boot from.
* [**storewolf**](sources/api/storewolf): A program that sets up the data store for the API upon boot.
* [**sundog**](sources/api/sundog): A program run during boot that generates any settings that depend on runtime system information.
  It finds settings that need generation by way of metadata in the API, and calls helper programs specified by that metadata.
* [**thar-be-settings**](sources/api/thar-be-settings): A program that writes out system configuration files, replacing template variables with settings from the API.
* [**updog**](sources/updater/updog): An update client that interfaces with a specified TUF updates repository to upgrade or downgrade Bottlerocket hosts to different image versions.

## Non-Bottlerocket terms

* **k8s**: [Kubernetes](https://kubernetes.io/), a container orchestration system.
* [**CNI**](https://github.com/containernetworking/cni): Container Network Interface, a standard for writing plugins to configure network interfaces in containers.
* **IMDS**: [Amazon EC2's Instance Metadata Service](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html).
  Used to retrieve user and platform configuration on an EC2 instance.
* [**sonobuoy**](https://github.com/vmware-tanzu/sonobuoy): A diagnostic tool and runs Kubernetes conformance tests for Kubernetes clusters.
* **SSM**: [AWS Systems Manager](https://aws.amazon.com/systems-manager/).
  The [SSM agent](https://docs.aws.amazon.com/systems-manager/latest/userguide/prereqs-ssm-agent.html) can be used for secure remote management.
* [**tough**](https://crates.io/crates/tough): a Rust implementation of The Update Framework (TUF).
* [**tuftool**](https://crates.io/crates/tuftool): a command line program for interacting with a TUF repo.
* **TUF**: [The Update Framework](https://theupdateframework.io/).
  A framework that helps developers maintain the security of software update systems.
* [**wicked**](https://github.com/openSUSE/wicked): A network interface framework and management system.
