# Publishing a Bottlerocket OVA on VMware

This guide will walk through some VMware specific details around making your OVA available as a VM or VM template in one or more software defined datacenters.

### Configuration details

As mentioned in the [PUBLISHING](PUBLISHING.md) guide, the process uses a configuration file called `Infra.toml`.
For VMware, you can specify details about your various vSphere instances and datacenters in `Infra.toml`, as well as configuration that may be common between datacenters.

It's important to note that we use [`govc`](https://github.com/vmware/govmomi/tree/master/govc) under the hood for interactions with vSphere, so at runtime **all datacenter configuration in `Infra.toml` is overridden by `GOVC_` environment variables.**
`govc` is run in a container, so you do not need to install it on your machine.
We first check for environment variables, then use `Infra.toml` for datacenter specific configuration, and finally common configuration.
The following `GOVC_` environment variables are supported:
* `GOVC_URL`
* `GOVC_DATACENTER`
* `GOVC_DATASTORE`
* `GOVC_NETWORK`
* `GOVC_FOLDER`
* `GOVC_RESOURCE_POOL`
* `GOVC_USERNAME`
* `GOVC_PASSWORD`

Credentials for your various datacenters may be stored at `~/.config/pubsys/vsphere-credentials.toml`.
The format of the file is below; each datacenter gets its own `[datacenter.NAME]` block, where `NAME` corresponds to a datacenter name in `Infra.toml`
Similar to other datacenter configuration, at runtime we first check for the environment variables `GOVC_USERNAME` and `GOVC_PASSWORD` and use one or both of them if they are set.

```toml
[datacenter.foo]
username = "username"
password = "password"

[datacenter.bar]
username = "bar"
password = "baz"
```

### Uploading a Bottlerocket OVA

You can specify the datacenters to which you would like to upload your OVA in `Infra.toml`.

```toml
[vmware]
datacenters = ["foo", "bar"]
```

Then you can easily upload your OVA, specifying the variant you wish to upload (currently only VMware variants).

```shell
cargo make -e BUILDSYS_VARIANT=vmware-k8s-1.24 upload-ova
```

If you would like to upload your OVA as a VM template, you can do this in a single step:

```shell
cargo make -e BUILDSYS_VARIANT=vmware-k8s-1.24 vmware-template
```

You can override the list of datacenters to upload to by specifying `VMWARE_DATACENTERS`:

```shell
cargo make vmware-template \
  -e BUILDSYS_VARIANT=vmware-k8s-1.24 \
  -e VMWARE_DATACENTERS="foo,bar"
```

If you would like to override the name of the VM, you can add on `-e VMWARE_VM_NAME=my-name`.

You can also override the import spec used when uploading the OVA by specifying `VMWARE_IMPORT_SPEC_PATH`.
Our [import spec template](tools/pubsys/support/vmware/import_spec.template) can be used as a starting point for further customization.

```shell
cargo make vmware-template \
  -e BUILDSYS_VARIANT=vmware-k8s-1.24 \
  -e VMWARE_IMPORT_SPEC_PATH=/path/to/my/spec.toml
```
