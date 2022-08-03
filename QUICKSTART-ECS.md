# Using a Bottlerocket AMI with Amazon ECS

[Amazon Elastic Container Service (Amazon ECS)](https://ecs.aws) is a highly scalable, fast container management service that makes it easy to run, stop, and manage containers on a cluster.
Your containers are defined in a task definition which you use to run individual tasks or as a service.

This quickstart will walk through setting up an Amazon ECS cluster with Bottlerocket container instances (using the EC2 launch type).
Check out the [Amazon ECS developer guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/Welcome.html) for an overview of ECS.

## Prerequisites

Before you begin, be sure that you've completed the steps in
[Setting up with Amazon ECS](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/get-set-up-for-amazon-ecs.html)
and that your AWS user has either the [`AdministratorAccess`](https://console.aws.amazon.com/iam/home#policies/arn:aws:iam::aws:policy/AdministratorAccess) policy
or the permissions specified in the [Amazon ECS First Run Wizard Permissions](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/security_iam_id-based-policy-examples.html#first-run-permissions) IAM policy example.

You'll also need [aws-cli](https://aws.amazon.com/cli/) set up to interact with AWS.


## Create a cluster

An Amazon ECS cluster is a logical grouping of tasks, services, and container instances.
For more information about clusters, see
[Amazon ECS clusters](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/clusters.html).

You can create a cluster with the AWS CLI as follows:

```
aws ecs --region us-west-2 create-cluster --cluster-name bottlerocket
```

> Note: The command above and subsequent examples include the AWS region, so change it from `us-west-2` if you operate in another region.

## Finding an AMI

Amazon provides official AMIs in the following AWS regions:

```
af-south-1
ap-east-1
ap-northeast-1
ap-northeast-2
ap-northeast-3
ap-south-1
ap-southeast-1
ap-southeast-2
ap-southeast-3
ca-central-1
cn-north-1
cn-northwest-1
eu-central-1
eu-north-1
eu-south-1
eu-west-1
eu-west-2
eu-west-3
me-south-1
sa-east-1
us-east-1
us-east-2
us-west-1
us-west-2
us-gov-east-1
us-gov-west-1
```

The official AMI IDs are stored in [public SSM parameters](https://docs.aws.amazon.com/systems-manager/latest/userguide/parameter-store-public-parameters.html).
The parameter names look like this: `/aws/service/bottlerocket/aws-ecs-1/x86_64/latest/image_id`

Just change the variant (`aws-ecs-1`) and architecture (`x86_64`) to the ones you want to use.
Supported variants and architectures are described in the [README](README.md#variants).
For the purposes of SSM parameters, the valid architecture names are `x86_64` and `arm64` (also known as `aarch64`).
Also, if you know a specific Bottlerocket version you'd like to use, for example `1.0.6`, you can replace `latest` with that version.

Once you have the parameter name you want to use, the easiest way to use it is to pass it directly to EC2.
Just prefix the parameter name with `resolve:ssm:` and EC2 will fetch the current value for you.
(You can also use this method for CloudFormation and other services that launch EC2 instances for you.)

For example, to use the parameter above, you would pass this as the AMI ID in your launch request: `resolve:ssm:/aws/service/bottlerocket/aws-ecs-1/x86_64/latest/image_id`

#### Manually querying SSM

If you prefer to fetch the AMI ID yourself, you can use [aws-cli](https://aws.amazon.com/cli/) on the command line.
To fetch the example parameter above, for the us-west-2 region, you could run this:

```
aws ssm get-parameter --region us-west-2 --name "/aws/service/bottlerocket/aws-ecs-1/x86_64/latest/image_id" --query Parameter.Value --output text
```

If you have `jq` and would like a bit more information, try this:
```
aws ssm get-parameters --region us-west-2 \
   --names "/aws/service/bottlerocket/aws-ecs-1/x86_64/latest/image_id" \
           "/aws/service/bottlerocket/aws-ecs-1/x86_64/latest/image_version" \
   --output json | jq -r '.Parameters | .[] | "\(.Name): \(.Value) (updated \(.LastModifiedDate | gmtime | strftime("%c")) UTC)"'
```

## Launching your first instance

In order to launch a Bottlerocket instance into your ECS cluster, you'll first need some information about the resources in your AWS account.

### Subnet info

You should either have a default virtual private cloud (VPC) or have already
[created a VPC](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/get-set-up-for-amazon-ecs.html#create-a-vpc)
in your account.

To find your default VPC, run this command.
(If you use an AWS region other than "us-west-2", make sure to change that.)

```
aws ec2 describe-vpcs \
   --region us-west-2 \
   --filters=Name=isDefault,Values=true \
   | jq --raw-output '.Vpcs[].VpcId'
```

If you want to use a different VPC you created, run this to get the ID for your VPC.
Make sure to change VPC_NAME to the name of the VPC you created.
(If you use an EC2 region other than "us-west-2", make sure to change that too.)

```
aws ec2 describe-vpcs \
   --region us-west-2 \
   --filters=Name=tag:Name,Values=VPC_NAME \
   | jq --raw-output '.Vpcs[].VpcId'
```

Next, run this to get information about the subnets in your VPC.
It will give you a list of the subnets and tell you whether each is public or private.
Make sure to change VPC_ID to the value you received from the previous command.
(If you use an EC2 region other than "us-west-2", make sure to change that too.)

```
aws ec2 describe-subnets \
   --region us-west-2 \
   --filter=Name=vpc-id,Values=VPC_ID \
   | jq '.Subnets[] | {id: .SubnetId, public: .MapPublicIpOnLaunch, az: .AvailabilityZone}'
```

You'll want to pick one and save it for the launch command later.

You can choose whether you want public or private.
* Choose private for production deployments to get maximum isolation of instances.
* Choose public to more easily debug your instance.
  These subnets have an Internet Gateway, so if you add a public IP address to your instance, you can talk to it.
  (You can manually add an Internet Gateway to a private subnet later, so this is a reversible decision.)

Note that if you choose to use the public subnet, you'll need your instance to have a publicly accessible IP address.
That either means adding `--associate-public-ip-address` to the launch command below, or attaching an Elastic IP address after launch.
There will be a reminder about this when we talk about the launch command.

Finally, note that if you want to launch in a specific availability zone, make sure you pick a subnet that matches; the AZ is listed right below the public/private status.

### IAM role

The instance we launch needs to be associated with an IAM role that allows for communication with ECS.

ECS provides a
[managed policy](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs_managed_policies.html#AmazonEC2ContainerServiceforEC2Role)
with all of the appropriate permissions.
If you've used ECS before, you may already have an appropriate role in your account called `ecsInstanceRole`.
If you do not, you can
[follow the instructions in the ECS Developer Guide to create a role](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/instance_IAM_role.html).

Note down the instance role name in your account for the instructions below.

#### Enabling SSM

If you add SSM permissions, you can use Bottlerocket's default SSM agent to get a shell session on the instance.

To attach the role policy for SSM permissions, run the following (replacing INSTANCE_ROLE_NAME with the name of your instance role):

```
aws iam attach-role-policy \
   --role-name INSTANCE_ROLE_NAME \
   --policy-arn arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore
```

Next, to retrieve the instance profile name used to launch instances, run this:

```
aws iam list-instance-profiles-for-role --role-name INSTANCE_ROLE_NAME --query "InstanceProfiles[*].InstanceProfileName" --output text
```

Note this down as the INSTANCE_PROFILE_NAME for the final launch command.

### Connecting to your cluster

For the instance to be able to communicate with ECS, we need to make sure to configure the instance with the name of the cluster.

Create a file called `userdata.toml` with the following contents, where CLUSTER_NAME is the name of the cluster you created above (for example, "bottlerocket").

```
[settings.ecs]
cluster = "CLUSTER_NAME"
```

If you want to customize the behavior of your instance further, you can find the full set of supported settings [here](README.md#settings).

### Launch!

Now we can launch a Bottlerocket instance in our cluster!

There are a few values to make sure you change in this command:
* YOUR_KEY_NAME: your SSH keypair name, as registered with EC2
* SUBNET_ID: the subnet you selected earlier
  * If you chose a public subnet, either add `--associate-public-ip-address` to the command, or attach an Elastic IP afterward.
* BOTTLEROCKET_AMI_ID: the Amazon-provided AMI ID you found above, or the ID of an AMI you registered
* userdata.toml: the path to the user data file you created earlier
* INSTANCE_PROFILE_NAME: the IAM instance profile you created, e.g. `ecsInstanceRole`

```
aws ec2 run-instances --key-name YOUR_KEY_NAME \
   --subnet-id SUBNET_ID \
   --image-id BOTTLEROCKET_AMI_ID \
   --instance-type c3.large \
   --region us-west-2 \
   --tag-specifications 'ResourceType=instance,Tags=[{Key=bottlerocket,Value=quickstart}]' \
   --user-data file://userdata.toml \
   --iam-instance-profile Name=INSTANCE_PROFILE_NAME
```

And remember, if you used a public subnet, add `--associate-public-ip-address` or attach an Elastic IP after launch.

Once it launches, you should be able to run tasks on your Bottlerocket instance using the ECS API and console.


### aws-ecs-*-nvidia variants

The `aws-ecs-*-nvidia` variants include the required packages and configurations to leverage NVIDIA GPUs.
They come with the [NVIDIA Tesla driver](https://docs.nvidia.com/datacenter/tesla/drivers/index.html) along with the libraries required by the [CUDA toolkit](https://developer.nvidia.com/cuda-toolkit) included in your ECS tasks.
In hosts with multiple GPUs (ex. EC2 `g4dn` instances) you can assign one or multiple GPUs per container by specifying the resource requirements in your container definitions as described in the [official ECS documentation](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs-gpu.html):

```json
{
  "containerDefinitions": [
     {
        "resourceRequirements" : [
            {
               "type" : "GPU",
               "value" : "2"
            }
        ]
     }
  ]
}
```
