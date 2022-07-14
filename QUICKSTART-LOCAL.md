# Testing Bottlerocket in a local virtual machine

This quickstart will walk through launching a Bottlerocket VM guest on a local machine using QEMU and KVM.
The VM will not join an ECS or Kubernetes cluster.
This way of running Bottlerocket is therefore best used for testing purposes when developing Bottlerocket components that do not need to integrate with any orchestrators or to just get a feel for what a Bottlerocket node looks from the inside.


## Prerequisites

We assume you are following along on a machine running Fedora.
If you are using a cloud VM, ensure you can use hardware-assisted virtualization.
For example, on Amazon EC2 this requires the use of a .metal instance type.

You need a clone of the main Bottlerocket repository and a build of the metal-dev variant.
Please refer to [`BUILDING.md`](https://github.com/bottlerocket-os/bottlerocket/blob/develop/BUILDING.md) for instructions on how to build a Bottlerocket image and ensure you pass `-e BUILDSYS_VARIANT=metal-dev` to `cargo make`.

The use of QEMU requires extra packages which you may install using this dnf invocation:

```
sudo dnf install qemu
```

If you'd (optionally) like to make use of the control container, you'll need an AWS account and AWS CLI.


## Configuring Bottlerocket

Bottlerocket is configured [via an API](https://github.com/bottlerocket-os/bottlerocket/#using-the-api-client) or, if running in a cloud VM, [via user data](https://github.com/bottlerocket-os/bottlerocket/#using-user-data) upon boot.
For running a local VM, neither mechanism can be used to apply configuration on first boot: Bottlerocket is not yet running, making its API server unavailable, and the goal to have Bottlerocket running locally precludes use of the user data mechanism.
As an alternative, the `start-local-vm` wrapper script included in the `tools` directory of the main repository allows to inject configuration into well-known locations of the built image for Bottlerocket to find on boot.


### Set up networking

The `start-local-vm` wrapper configures QEMU to provide one virtual network interface to the VM.
To enable this interface, create a file named `net.toml` containing the following TOML snippet:

```
version = 1

[enp0s16]
dhcp4 = true
```

This will prompt [netdog](https://github.com/bottlerocket-os/bottlerocket/blob/develop/sources/api/netdog/README.md) to set up `enp0s16` as the primary network interface with IPv4 networking configured via DHCP.
No dedicated DHCP server needs to be running on the host as QEMU will act as one on the virtual network interface.
Note that for virtual machines launched with `start-local-vm`, the primary network interface will always be named `enp0s16`.
The name will differ when running on bare metal or in a cloud environment.


### Accessing your Bottlerocket guest via host containers

When running a Bottlerocket development variant such as metal-dev locally, you can directly interact with the system via the serial console that `start-local-vm` connects you to.
For remote access to your running Bottlerocket VMs, you will need to provide additional configuration to enable host containers.
The Bottlerocket metal images don't include any host containers enabled by default.
But don't worry!
You can use our [admin](https://github.com/bottlerocket-os/bottlerocket-admin-container) and/or [control](https://github.com/bottlerocket-os/bottlerocket-control-container) containers, they just need to be configured first.
Information about the roles these host containers play can be found [here](https://github.com/bottlerocket-os/bottlerocket/#exploration).


#### Admin container

If you would like to use the admin container, you will need to create some base64 encoded user data which will be passed to the container at runtime.
Full details are covered in the [admin container documentation](https://github.com/bottlerocket-os/bottlerocket-admin-container#authenticating-with-the-admin-container).
If we assume you have a public key at `${HOME}/.ssh/id_rsa.pub`, the below will add the correct user data to your `user-data.toml`.

```
PUBKEY_FILE="${HOME}/.ssh/id_rsa.pub"
PUBKEY=$(< "${PUBKEY_FILE}")
ADMIN_USER_DATA="$(echo '{"ssh": {"authorized-keys": ["'"${PUBKEY}"'"]}}' | base64 -w 0)"

cat <<EOF >>user-data.toml
[settings.host-containers.admin]
enabled = true
user-data = "${ADMIN_USER_DATA}"
source = "public.ecr.aws/bottlerocket/bottlerocket-admin:v0.9.0"
EOF
```


#### Control container

Enabling the control container is very similar to the admin container; you will create some base64 encoded user data that will be passed to the container at runtime.
This user data includes an activation ID and code retrieved from AWS SSM.
Full details can be found in the [control container documentation](https://github.com/bottlerocket-os/bottlerocket-control-container#connecting-to-aws-systems-manager-ssm).

You'll first need an AWS account, and AWS CLI installed.
Then you'll create a service role in that account to [grant AWS STS trust to the AWS Systems Manager service](https://docs.aws.amazon.com/systems-manager/latest/userguide/sysman-service-role.html).

```
cat <<EOF > ssmservice-trust.json
{
    "Version": "2012-10-17",
    "Statement": {
        "Effect": "Allow",
        "Principal": {
            "Service": "ssm.amazonaws.com"
        },
        "Action": "sts:AssumeRole"
    }
}
EOF

# Create the role using the above policy
aws iam create-role \
    --role-name SSMServiceRole \
    --assume-role-policy-document file://ssmservice-trust.json

# Attach the policy enabling the role to create session tokens
aws iam attach-role-policy \
    --role-name SSMServiceRole \
    --policy-arn arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore
```

Once the above is created, we can use the role to create an activation code and ID.

```
export SSM_ACTIVATION="$(aws ssm create-activation \
  --iam-role SSMServiceRole \
  --registration-limit 100 \
  --region us-west-2 \
  --output json)"
```

Using the above activation data we can create our user data to pass to the control container

```
SSM_ACTIVATION_ID="$(jq -r '.ActivationId' <<< ${SSM_ACTIVATION})"
SSM_ACTIVATION_CODE="$(jq -r '.ActivationCode' <<< ${SSM_ACTIVATION})"
CONTROL_USER_DATA="$(echo '{"ssm": {"activation-id": "'${SSM_ACTIVATION_ID}'", "activation-code": "'${SSM_ACTIVATION_CODE}'", "region": "us-west-2"}}' | base64 -w0)"

cat <<EOF >>user-data.toml
[settings.host-containers.control]
enabled = true
user-data = "${CONTROL_USER_DATA}"
source = "public.ecr.aws/bottlerocket/bottlerocket-control:v0.6.1"
EOF
```


## Launch!

We have now prepared all configuration we might need.
Assuming you are in the root of the main Bottlerocket repository, you can run

```
./tools/start-local-vm --variant metal-dev --arch $(uname -m) --inject-file net.toml --inject-file user-data.toml
```

to start a local VM with the Bottlerocket image you built earlier.

The `--inject-file` options add the listed files to the private partition of the image, where Bottlerocket's various services will find them on boot.
The final configuration files ending up in the image need to be named like in the examples above.
If you named yours differently, you can ensure they have the right name in the image by using a colon as the separator of local file name and file name in the image, e.g. `--inject-file admin-container-only.toml:user-data.toml`.
If you did not enable any host containers and thus have no `user-data.toml` you need to leave this option off.

Once the VM launches, boot output will be visible in the terminal.
The `start-local-vm` script connects you to the serial console of the VM, which can also be used to interact with the system if you are running a development variant such as metal-dev.
When prompted to login, any username will do.

The virtual serial console will capture most keyboard input, such as Ctrl-C.
If you want to terminate the VM, you can either instruct it to `systemctl poweroff` from within or exit QEMU via the Ctrl-A X shortcut.

By default, the `start-local-vm` wrapper will forward the host's TCP port 2222 to the VM's port 22.
If you enabled the admin host container, the SSH server running in it will therefore be available by connecting to localhost's port 2222:

```
ssh -p 2222 ec2-user@localhost
```
