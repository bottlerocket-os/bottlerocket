# Provisioning Bottlerocket on metal

This guide will describe what is needed to properly provision Bottlerocket on bare metal.
Provisioning Bottlerocket on metal is different than provisioning other general-purpose distros.
Since Bottlerocket has a `dm-verity`-checked boot and root partition, and is immutable at runtime, a user cannot provision an image and directly write configuration files.
Bottlerocket requires a few files to be generated and written to disk at provisioning time in order to boot properly; these files are described below.

For more information about the hardware that Bottlerocket for bare metal is currently tested on, see [SUPPORTED-HARDWARE](SUPPORTED-HARDWARE.md).

## High level provisioning steps

The high level steps to provision Bottlerocket images for bare metal to your host are below.
Most provisioning systems provide methods to achieve the following:

* Decompress (`unlz4`) and write the Bottlerocket image to the desired disk
* Mount the `BOTTLEROCKET-PRIVATE` partition (partition 12)
* Write the below files to the mounted partition (these files are further described below):
  * (Required) [`user-data.toml`](#user-data)
  * (Required) [`net.toml`](#network-interface-configuration)
  * (Optional, recommended) [`bootconfig.data`](#boot-configuration)
* Reboot

### Fetch the Bottlerocket image for bare metal

The Bottlerocket image for bare metal is signed and uploaded alongside the rest of the Bottlerocket release artifacts.

You first need the Bottlerocket root role, which is used by `tuftool` to verify the image.
The following will download and verify the root role itself:

```shell
curl -O "https://cache.bottlerocket.aws/root.json"
sha512sum -c <<<"b81af4d8eb86743539fbc4709d33ada7b118d9f929f0c2f6c04e1d41f46241ed80423666d169079d736ab79965b4dd25a5a6db5f01578b397496d49ce11a3aa2  root.json"
```

Next, set your desired version and variant, and use `tuftool` to download the image:
To install `tuftool` you'll need to install Rust (via [rustup](https://rustup.rs/) or the official site), and then you can run `cargo install tuftool`.

```shell
ARCH="x86_64"
VERSION="v1.9.0"
VARIANT="metal-k8s-1.23"
IMAGE="bottlerocket-${VARIANT}-${ARCH}-${VERSION}.img.lz4"
OUTDIR="${VARIANT}-${VERSION}"

tuftool download "${OUTDIR}" --target-name "${IMAGE}" \
   --root ./root.json \
   --metadata-url "https://updates.bottlerocket.aws/2020-07-07/${VARIANT}/x86_64/" \
   --targets-url "https://updates.bottlerocket.aws/targets/"
```

### User data

Bottlerocket for bare metal expects a TOML-formatted file named `user-data.toml` that contains user data settings.
Acceptable settings can be found in the [settings docs](https://github.com/bottlerocket-os/bottlerocket#settings).

If you're just getting started and want to provision a host without connecting to a Kubernetes cluster, you can use the following example user data which will start `kubelet` in standalone mode.

```toml
[settings.kubernetes]
standalone-mode = true
```

For remote access to your running Bottlerocket hosts, you will need to add user data to enable host containers.
The Bottlerocket images for bare metal don't enable any host containers by default.
You can use our [admin](https://github.com/bottlerocket-os/bottlerocket-admin-container) and/or [control](https://github.com/bottlerocket-os/bottlerocket-control-container) containers, but they need to be configured first.
Full configuration details are covered in the [admin container documentation](https://github.com/bottlerocket-os/bottlerocket-admin-container#authenticating-with-the-admin-container) and the [control container documentation](https://github.com/bottlerocket-os/bottlerocket-control-container#connecting-to-aws-systems-manager-ssm).

### Network interface configuration

Bottlerocket for bare metal provides the means to configure the physical network interfaces in the system via TOML-formatted file `net.toml`.

`net.toml` is read at boot time and generates the proper configuration files in the correct format for each interface described; no default configuration is provided.
If no network configuration is provided, boot-time services like host containers, `containerd`, and `kubelet` will fail to start.
When these services fail, your machine will not connect to any cluster and will be unreachable via host containers.

#### `net.toml` structure

The configuration file must be valid TOML and have the filename `net.toml`.
The first and required top level key in the file is `version`; the latest is version `3`.
The rest of the file is a map of interface name or MAC address to supported settings.
Interface names are expected to be correct as per `udevd` naming, no interface naming or matching is supported.
(See the note below regarding `udevd` interface naming.)

#### Supported interface settings

* `primary` (boolean): Use this interface as the primary network interface. `kubelet` will use this interface's IP when joining the cluster.  If none of the interfaces has `primary` set, the first interface in the file is used as the primary interface.
* `dhcp4` (boolean or map): Turns on DHCP4 for the interface.  If additional DHCP4 configuration is required, the following settings are supported and may be provided as a map with the following keys:
  * `enabled` (boolean, required): Enables DHCP4.
  * `route-metric` (integer): Prioritizes routes by setting values for preferred interfaces.
  * `optional` (boolean): the system will request a lease using this protocol, but will not wait for a valid lease to consider this interface configured.
* `dhcp6` (boolean or map): Turns on DHCP6 for the interface.  If additional DHCP6 configuration is required, the following settings are supported and may be provided as a map with the following keys:
  * `enabled` (boolean, required): Enables DHCP6.
  * `optional` (boolean): the system will request a lease using this protocol, but will not wait for a valid lease to consider this interface configured.

As of version `2` static addressing with simple routes is supported via the below settings.
Please keep in mind that when using static addresses, DNS information must be supplied to the system via user data: [`settings.dns`](https://github.com/bottlerocket-os/bottlerocket#network-settings).

* `static4` (map): IPv4 static address settings.
  * `addresses` (list of quoted IPv4 address including prefix): The desired IPv4 IP addresses, including prefix i.e. `["192.168.14.2/24"]`.  The first IP in the list will be used as the primary IP which `kubelet` will use when joining the cluster.  If IPv4 and IPv6 static addresses exist, the first IPv4 address is used.
* `static6` (map): IPv6 static address settings.
  * `addresses` (list of quoted IPv6 address including prefix): The desired IPv6 IP addresses, including prefix i.e. `["2001:dead:beef::2/64"]`.  The first IP in the list will be used as the primary IP which `kubelet` will use when joining the cluster.  If IPv4 and IPv6 static addresses exist, the first IPv4 address is used.

* `route` (map): Static route; multiple routes can be added. (cannot be used in conjuction with DHCP)
  * `to` (`"default"` or IP address with prefix, required): Destination address.
  * `from` (IP address): Source IP address.
  * `via` (IP address): Gateway IP address.  If no gateway is provided, a scope of `link` is assumed.
  * `route-metric` (integer): Relative route priority.

Version `3` adds support for bonding, vlan tagging, and the ability to use a MAC address (colon or dash separated) as the identifier for an interface.
MAC address identification is limited to interface configuration *only* and may not be used in conjunction with bonds or vlans.
[Bonding](https://www.kernel.org/doc/Documentation/networking/bonding.txt) support is limited to mode `1` (`active-backup`).
Future support may include other bonding options - pull requests are welcome!
Version `3` adds the concept of virtual network devices in addition to interfaces.
The default type of device is an interface and the syntax is the same as previous versions.
The name of an interface must match an existing interface on the system such as `eno1` or `enp0s16`.
For virtual network devices, a `kind` is required.
If no `kind` is specified, it is assumed to be an interface.
Currently, `bond` and `vlan` are the two supported `kind`s.
Virtual network devices are created, and therefore a name has to be chosen.

Names for virtual network devices must conform to kernel naming restrictions:
* Names must not have line terminators in them
* Names must be between 1-15 characters
* Names must not contain `.`, `/` or whitespace

Bonding configuration creates a virtual network device across several other devices:

* Bonding configuration (map):
  * `kind = "bond"`: This setting is required to specify a bond device. Required.
  * `interfaces` (list of quoted strings of interface names, not MAC addresses): Which interfaces should be added to the bond (i.e. `["eno1"]`). The first in the list is considered the default `primary`. These interfaces are "consumed" so no other configuration can refer to them. Required.
  * `mode` (string): Currently `active-backup` is the only supported option. Required.
  * `min-links` (integer): Number of links required to bring up the device
  * `monitoring` (map): Values m ust all be of `miimon` or `arpmon` type.
    The user must choose one type of monitoring and configure it fully in order for the bond to properly function.
    See [section 7](https://www.kernel.org/doc/Documentation/networking/bonding.txt) for more background on what to choose.
    * `miimon-frequency-ms` (integer): MII Monitoring frequency in milliseconds
    * `miimon-updelay-ms` (integer): MII Monitoring delay before the link is enabled after link is detected in milliseconds
    * `miimon-downdelay-ms` (integer): MII Monitoring delay before the link is disabled after link is no longer detected in milliseconds
    * `arpmon-interval-ms` (integer): Number of milliseconds between intervals to determine link status, must be greater than 0
    * `arpmon-validate` (one of `all`, `none`, `active`, or `backup`): What packets should be used to validate link
    * `arpmon-targets` (list of quoted IPv4 address including prefix): List of targets to use for validating ARP. Min = 1, Max = 16

Vlan tagging is configured as a new virtual network device stacked on another device:

* Vlan configuration (map):
  * `kind = "vlan"`: This setting is required to specify a vlan device.
  * `device` (string for device name, not MAC address): Defines the device the vlan should be configured on. If VLAN tagging is required, this device should recieve all IP address configuration instead of the underlying device.
  * `id` (integer): Number between 0 and 4096 specifying the vlan tag on the device

Example `net.toml` version `3` with comments:

```toml
version = 3

# "eno1" is the interface name
[eno1]
# Users may turn on dhcp4 and dhcp6 via boolean
dhcp4 = true
dhcp6 = true
primary = true

# "eno2" is the second interface in this example
[eno2.dhcp4]
# `enabled` is a boolean and is a required key when
# setting up DHCP this way
enabled = true
# Route metric may be supplied for IPv4
route-metric = 200

[eno2.dhcp6]
enabled = true
optional = true

[eno3.static4]
addresses = ["10.0.0.10/24", "11.0.0.11/24"]

# Multiple routes may be configured
[[eno3.route]]
to = "default"
via = "10.0.0.1"
route-metric = 100

[[eno3.route]]
to = "default"
via = "11.0.0.1"
route-metric = 200

[eno4.static4]
addresses = ["192.168.14.5/24"]

# Using a source IP and non-default route
[[eno4.route]]
to = "10.10.10.0/24"
from = "192.168.14.5"
via = "192.168.14.25"

# Interfaces may be configured using their MAC address rather than the interface name.
# The MAC address must be quoted and colon or dash separated
["0e:b3:69:44:b6:33"]
dhcp4 = true

["3e:03:69:49:e6:31".static4]
addresses = ["10.0.0.15/24"]

[["3e:03:69:49:e6:31".route]]
to = "default"
via = "10.0.0.1"

# A bond is a network device that is of `kind` `bond`
[bond0]
kind = "bond"
# Currently `active-backup` is the only supported option
mode = "active-backup"
# In this case, the vlan will have addressing, the bond is simply there for use in the vlan
dhcp4 = false
dhcp6 = false
# The first interface in the array is considered `primary` by default, this list may not contain MAC addresses.
interfaces = ["eno11", "eno12"]

[bond0.monitoring]
miimon-frequency-ms = 100 # 100 milliseconds
miimon-updelay-ms = 200 # 200 milliseconds
miimon-downdelay-ms = 200 # 200 milliseconds

[bond1]
kind = "bond"
mode = "active-backup"
interfaces = ["eno51" , "eno52", "eno53"]
min-links = 2 # Optional min-links 
dhcp4 = true

[bond1.monitoring]
arpmon-interval-ms = 200 # 200 milliseconds
arpmon-validate = "all"
arpmon-targets = ["192.168.1.1", "10.0.0.2"]

# A vlan is a network device that is of `kind` `vlan`
# VLAN42 is the name of the device, can be anything that is a valid network interface name
[VLAN42]
kind = "vlan"
# `device` may not contain a MAC address.
device = "bond0"
id = 42
dhcp4 = true

[internal_vlan]
kind = "vlan"
device = "eno2"
id = 1234
dhcp6 = true
```

#### **An additional note on network device names**

Interface name policies are [specified in this file](https://github.com/bottlerocket-os/bottlerocket/blob/develop/packages/release/80-release.link#L6); with name precedence in the following order: onboard, slot, path.
Typically on-board devices are named `eno*`, hot-plug devices are named `ens*`, and if neither of those names are able to be generated, the “path” name is given, i.e `enp*s*f*`.

#### Networking configuration versions and Releases

Older networking configuration versions (such as `1` or `2`) are supported in newer releases. In order to use a newer version, the following table provides guidance on what release first enabled the version.

| Network Configuration Version | First Release                                                                   |
|-------------------------------|---------------------------------------------------------------------------------|
| Version 1                     | [v1.9.0](https://github.com/bottlerocket-os/bottlerocket/releases/tag/v1.9.0)   |
| Version 2                     | [v1.10.0](https://github.com/bottlerocket-os/bottlerocket/releases/tag/v1.10.0) |
| Version 3                     | [v1.12.0](https://github.com/bottlerocket-os/bottlerocket/releases/tag/v1.12.0) |

#### Validate network configuration

`netdog` has a command `validate-net-config` to validate that network configuration files parse correctly.
This command is intended to validate the format and structure of the file, it will not guarantee the generated configuration will work in a particular context.
The command can be run by passing the path to a `net.toml` file:

```bash
# from the root of the bottlerocket git repo
VARIANT=metal-dev cargo run --manifest-path sources/api/netdog/Cargo.toml -- validate-net-config -f sources/api/netdog/test_data/net_config/net_config.toml
...
eno2 found as primary interface
Found eno1
Found eno2
Found eno3
Found eno4
Found eno5
Found eno6
Found eno7
Found eno8
Found eno9
Found eno10
net.toml file provided successfully parsed!
```

Errors will also be detected and printed to standard out.

```bash
VARIANT=metal-dev cargo run --manifest-path sources/api/netdog/Cargo.toml -- validate-net-config -f sources/api/netdog/test_data/net_config/basic/bad_version.toml
...
Unable to read/parse network config from 'sources/api/netdog/test_data/net_config/basic/bad_version.toml': Invalid network configuration: Unknown network config version: 50
```

### Boot Configuration

Bottlerocket for bare metal uses a feature of the Linux kernel called [Boot Configuration](https://www.kernel.org/doc/html/latest/admin-guide/bootconfig.html), which allows a user to pass additional arguments to the kernel command line at runtime.
An immediate use of this feature for most users is setting `console` settings so boot messages can be seen on the appropriate consoles.

In order to make use of this feature, an initrd is created with the desired settings encoded inside it.
The initrd is empty save for the encoded boot config data.
To create the initrd, you must first create a configuration file containing key value pairs for the settings you would like to pass to kernel / init.
Full syntax is described in the [Boot Config documentation](https://www.kernel.org/doc/html/latest/admin-guide/bootconfig.html#config-file-syntax), but a simple example is provided below that shows the format of console settings as well as an example `systemd` parameter.

The two acceptable prefixes to settings are `kernel` and `init`.
Settings prefixed with `kernel` are added to the beginning of the kernel command line.
Settings prefixed with `init` are added to the kernel command line after the `--`, but before any existing init parameters.

In the example below, two console devices are set up, and `systemd`'s log level is set to `debug`.

Example Boot Configuration:

```
kernel {
    console = tty0, "ttyS1,115200n8"
}
init {
    systemd.log_level = debug
}
```

The Bottlerocket SDK provides the `bootconfig` CLI tool, which is used to create a Boot Configuration initrd.
To create the Boot Configuration initrd, create a config file named `bootconfig-input` containing your desired key/value pair kernel and init arguments.

Then run the following (you will need Docker installed):

```shell
ARCH=$(uname -m)
SDK_VERSION="v0.26.0"
SDK_IMAGE="public.ecr.aws/bottlerocket/bottlerocket-sdk-${ARCH}:${SDK_VERSION}"

touch $(pwd)/bootconfig.data

docker run --rm \
   --network=none \
   --user "$(id -u):$(id -g)" \
   --security-opt label:disable \
   -v $(pwd)/bootconfig-input:/tmp/bootconfig-input \
   -v $(pwd)/bootconfig.data:/tmp/bootconfig.data \
   "${SDK_IMAGE}" \
   bootconfig -a /tmp/bootconfig-input /tmp/bootconfig.data
```

The above command will create the properly named initrd `bootconfig.data` in your current directory.
This is the file you will write to disk during provisioning.

You can list a `bootconfig.data`'s contents, which also validates its format, by running:

```shell
ARCH=$(uname -m)
SDK_VERSION="v0.26.0"
SDK_IMAGE="public.ecr.aws/bottlerocket/bottlerocket-sdk-${ARCH}:${SDK_VERSION}"

docker run --rm \
   --network=none \
   --user "$(id -u):$(id -g)" \
   --security-opt label:disable \
   -v $(pwd)/bootconfig.data:/tmp/bootconfig.data \
   "${SDK_IMAGE}" \
   bootconfig -l /tmp/bootconfig.data
```
