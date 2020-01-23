# Building Thar

If you'd like to build your own image instead of relying on an Amazon-provided image, follow these steps.
You can skip to [Using a Thar AMI](#using-a-thar-ami) to use an existing image in Amazon EC2.
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
cargo install cargo-deny --version 0.2.6
```

#### Docker

Thar uses [Docker](https://docs.docker.com/install/#supported-platforms) to orchestrate package and image builds.

We recommend Docker 19.03 or later.
Builds rely on Docker's integrated BuildKit support, which has received many fixes and improvements in newer versions.

You'll need to have Docker installed and running, with your user account added to the `docker` group.
Docker's [post-installation steps for Linux](https://docs.docker.com/install/linux/linux-postinstall/) will walk you through that.

### Build process

To build an image, run:

```
cargo make world
```

All packages will be built in turn, and then compiled into an `img` file in the `build/` directory.

You may want to take advantage of multiple cores on your system by running `make -j7`, for example, which will build up to 7 components in parallel.

## Register an AMI

To use the image in Amazon EC2, we need to register the image as an AMI.
The `bin/amiize.sh` script does this for you.

The script has some assumptions about your setup, in particular that you:
  * have [aws-cli v1](https://aws.amazon.com/cli/) set up, and that its default profile can create and control EC2 resources.
  * An SSH key that's registered with EC2 is loaded into `ssh-agent`.

First, decompress the images.
(Note: these filenames assume an `x86_64` architecture and `aws-k8s` [variant](README.md#variant).)

```
lz4 -d build/thar-x86_64-aws-k8s.img.lz4 build/thar-x86_64-aws-k8s.img && \
lz4 -d build/thar-x86_64-aws-k8s-data.img.lz4 build/thar-x86_64-aws-k8s-data.img
```

Next, register an AMI:

```
bin/amiize.sh --name YOUR-AMI-NAME-HERE \
              --ssh-keypair YOUR-EC2-SSH-KEYPAIR-NAME-HERE \
              --root-image build/thar-aws-k8s-x86_64.img \
              --data-image build/thar-x86_64-aws-k8s-data.img \
              --region us-west-2 \
              --instance-type m3.xlarge \
              --arch x86_64 \
              --worker-ami ami-08d489468314a58df \
              --user-data 'I2Nsb3VkLWNvbmZpZwpyZXBvX3VwZ3JhZGU6IG5vbmUK'
```

The new AMI ID will be printed at the end.

The amiize script starts an EC2 instance, which it uses to write the image to a new EBS volume.
It then registers this EBS volume as an AMI and terminates the instance.
In the example command above, the `--worker-ami` is an Amazon Linux AMI, and the `--user-data` disables updates at boot to speed up registration.
Make sure you use an up-to-date worker AMI.

# Using a Thar AMI

The first release of Thar focuses on Kubernetes, in particular serving as the host OS for Kubernetes pods.

One easy way to get started is to use Amazon EKS, a service that manages a Kubernetes control plane for you.
This document will focus on EKS to make it easy to follow a single path.
There's nothing that limits Thar to EKS or AWS, though.

Most of this is one-time setup, and yes, we plan to automate more of it!
Once you have a cluster, you can skip to the last step, [Launch!](#launch)

## Dependencies

EKS has a command-line tool called `eksctl` that makes cluster setup easy.
First, get [eksctl set up](https://github.com/weaveworks/eksctl).

You'll also need to [install kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/) to augment `eksctl` during setup, and to run pods afterward.

Finally, you'll need [aws-cli v1](https://aws.amazon.com/cli/) set up to interact with AWS.
(You'll need a [recent v1 release](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-install.html#install-tool-bundled) with EKS support.)

## Cluster setup

*Note:* most commands will have a region argument; make sure to change it if you don't want to set up in us-west-2.

You can set up a new cluster like this:

```
eksctl create cluster --region us-west-2 --name thar
```

Now that the cluster is created, we can have `eksctl` create the configuration for `kubectl`:
```
eksctl utils write-kubeconfig --region us-west-2 --name thar
```

Now we can make a configuration change to use a CNI plugin that's compatible with Thar.
```
kubectl patch daemonset aws-node \
-n kube-system \
-p '{"spec": {"template": {"spec": {"containers": [{"image": "602401143452.dkr.ecr.us-west-2.amazonaws.com/amazon-k8s-cni:v1.6.0-rc4","name":"aws-node"}]}}}}'
```

## Cluster info

This section helps you determine some of the cluster information needed later by the instance launch command.

### Kubernetes cluster info

Run this to get the API endpoint and base64-encoded certificate authority, which we use in the next step.
```
eksctl get cluster --region us-west-2 --name thar -o json \
   | jq --raw-output '.[] | "Endpoint: " + .Endpoint,"\nCA: " + .CertificateAuthority.Data'
```

Using that information from eksctl, create a file like this, named `userdata.toml`.
This will be used at the end, in the instance launch command.

```
[settings.kubernetes]
api-server = "YOUR-API-ENDPOINT-HERE"
cluster-name = "thar"
cluster-certificate = "YOUR-CERTIFICATE-AUTHORITY-HERE"
```

### Subnet info

Next, run this to get information about the subnets that eksctl created.
It will give you a list of the subnets and tell you whether each is public or private.
(If you use an EC2 region other than "us-west-2", make sure to change that.)

```
aws ec2 describe-subnets \
   --subnet-ids $(eksctl get cluster --region us-west-2 --name thar -o json | jq --raw-output '.[].ResourcesVpcConfig.SubnetIds[]') \
   --region us-west-2 \
   --query "Subnets[].[SubnetId, Tags[?Key=='aws:cloudformation:logical-id'].Value]" \
   | xargs -L2
```

You'll want to pick one and save it for the launch command later.

You can choose whether you want public or private.
* Choose private for production deployments to get maximum isolation of worker nodes.
* Choose public to more easily debug your instance.  These subnets have an Internet Gateway, so if you add a public IP address to your instance, you can talk to it.  (You can manually add an Internet Gateway to a private subnet later, so this is a reversible decision.)

Note that if you choose to use the public subnet, you'll need your instance to have a publicly accessible IP address.
That either means adding `--associate-public-ip-address` to the launch command below, or attaching an Elastic IP address after launch.
There will be a reminder about this when we talk about the launch command.

Finally, note that if you want to launch in a specific availability zone, make sure you pick a subnet that matches; the AZ is listed right next to the public/private status.

## IAM role

The instance we launch needs to be associated with an IAM role that allows for communication with EKS.

`eksctl` by default already creates such a role (and an instance profile that allows use of the role) as part of the cluster nodegroup.

The ARN of the IAM role can be retrieved with:

```
eksctl get iamidentitymapping --region us-west-2 --name thar
```

The output should look like this:

```
ARN                                                               USERNAME                                GROUPS
arn:aws:iam::YOUR_AWS_ACCOUNT_ID:role/INSTANCE_ROLE_NAME          system:node:{{EC2PrivateDNSName}}       system:bootstrappers,system:nodes
```

Note down the INSTANCE_ROLE_NAME for the instructions below.

#### Enabling SSM

If you add SSM permissions, you can use Thar's default SSM agent to get a shell session on the instance.

To attach the role policy for SSM permissions, run the following:

```
aws iam attach-role-policy \
   --role-name INSTANCE_ROLE_NAME \
   --policy-arn arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore
```

If you receive the following error, you need to truncate INSTANCE_ROLE_NAME to 64 characters.
(We are working on improving this.)

```
1 validation error detected: Value 'INSTANCE_ROLE_NAME' at 'role Name' failed to satisfy constraint:
Member must have length less than or equal to 64
```

Next, to retrieve the instance profile name used to launch instances, run this:

```
aws iam list-instance-profiles-for-role --role-name INSTANCE_ROLE_NAME --query "InstanceProfiles[*].InstanceProfileName" --output text
```

There should only be one that looks like:
```
eksctl-thar-nodegroup-ng-IDENTIFIER-NodeInstanceProfile-IDENTIFIER
```

Note this down as the INSTANCE_PROFILE_NAME for the final launch command.

## kube-proxy settings

By default `kube-proxy` will set the `nf_conntrack_max` kernel parameter to a default value that may differ from what Thar originally sets at boot.
If you prefer to keep Thar's [default setting](packages/release/release-sysctl.conf), edit the kube-proxy configuration details with:

```
kubectl edit -n kube-system daemonset kube-proxy
```

Add `--conntrack-max-per-core` and `--conntrack-min` to the kube-proxy arguments like so (a setting of 0 implies no change):

```
      containers:
      - command:
        - kube-proxy
        - --v=2
        - --config=/var/lib/kube-proxy-config/config
        - --conntrack-max-per-core=0
        - --conntrack-min=0

```

## Final launch details

For the instance to be able to communicate with the EKS cluster control plane and other worker nodes, we need to make sure the instance is launched with the right security groups.

Run the following command:

```
aws ec2 describe-security-groups --filters 'Name=tag:Name,Values=*thar*' \
  --query "SecurityGroups[*].{Name:GroupName,ID:GroupId}"
```

This will output several security group names and IDs.
You want to save the IDs for the `...ClusterSharedNodeSecurityGroup...` and `...nodegroup...` entries.

Example:

```
[
    {
        "Name": "eksctl-thar-cluster-ClusterSharedNodeSecurityGroup-IDENTIFIER",
        "ID": "SECURITY_GROUP_ID_1"
    },
    {
        "Name": "eksctl-thar-cluster-ControlPlaneSecurityGroup-IDENTIFIER",
        "ID": *ignore*
    },
    {
        "Name": "eksctl-thar-nodegroup-ng-IDENTIFIER-SG-IDENTIFIER",
        "ID": "SECURITY_GROUP_ID_2"
    }
]
```

## Launch!

Now we can launch a Thar instance in our cluster!

There are a few values to make sure you change in this command:
* YOUR_KEY_NAME: your SSH keypair name, as registered with EC2
* SUBNET_ID: the subnet you selected earlier
* If you chose a public subnet, either add `--associate-public-ip-address` to the command, or attach an Elastic IP afterward.
* SECURITY_GROUP_ID_1, SECURITY_GROUP_ID_2: the two security groups you found earlier
* THAR-AMI-ID: the ID of the AMI you registered, or an Amazon-provided AMI ID
* userdata.toml: the path to the user data file you created earlier
* INSTANCE_PROFILE_NAME: the instance profile created by `eksctl` for the cluster nodegroups. 

```
aws ec2 run-instances --key-name YOUR_KEY_NAME \
   --subnet-id SUBNET_ID \
   --security-group-ids SECURITY_GROUP_ID_1 SECURITY_GROUP_ID_2 \
   --image-id THAR_AMI_ID \
   --instance-type c3.large \
   --region us-west-2 \
   --tag-specifications 'ResourceType=instance,Tags=[{Key=kubernetes.io/cluster/thar,Value=owned}]' \
   --user-data file://userdata.toml \
   --iam-instance-profile Name=INSTANCE_PROFILE_NAME
```

And remember, if you used a public subnet, add `--associate-public-ip-address` or attach an Elastic IP after launch.

Once it launches, you should be able to run pods on your Thar instance using normal Kubernetes workflows.

For example, to run busybox:
`kubectl run -i -t busybox --image=busybox --restart=Never`
