# Testing Bottlerocket

🚧 👷

This section is under active development.
We are working on tooling for running Bottlerocket integration tests.
While the work is underway, there will be frequent changes to this document.

## Unit Tests

It is easy to execute unit tests, you can run them from the root of the repo with `cargo make unit-tests`.
Note that some code in Bottlerocket is conditionally compiled based on variant thus some tests won't be executed.
Unless you intend to test the default variant, it is best to pass the relevant variant and architecture like this:

```shell
cargo make \
  -e BUILDSYS_VARIANT="aws-ecs-1" \
  -e BUILDSYS_ARCH="x86_64" \
  unit-tests
```

## Integration Tests

Unit tests will only get us so far.
Ultimately we want to know if Bottlerocket runs correctly as a complete system.
We have created a [command line utility] and [testing system] to help us test Bottlerocket holistically.

[command line utility]: ./tools/testsys
[testing system]: https://github.com/bottlerocket-os/bottlerocket-test-system

The test system coordinates:
- the creation of a cluster (or re-use of an existing cluster),
- creation of Bottlerocket instances,
- running tests that target the created cluster and instances,
- terminating the Bottlerocket instances,
- terminating the Kubernetes cluster (if desired)

Testsys uses a Kubernetes operator to test bottlerocket.
The operator runs in a cluster that is separate from the one where you are testing Bottlerocket nodes.
We call this control cluster the *testsys cluster*.
When you launch a Bottlerocket integration test, pods run in the testsys cluster to perform the steps described above.

## Setup

### EKS

It is possible to run your testsys cluster anywhere so long as it has the necessary authorization and networking.
We have plans to make this easy to do in EKS by providing the instructions and role permissions you need.
However, some work is still needed on the roles, so check back for those instructions in the future!

### Using a Temporary Kind Cluster

For developer workflows, the quickest way to run a testsys cluster is using [kind].

[kind]: https://kind.sigs.k8s.io/

**Important:** only use `kind` for temporary testsys clusters that you will be using yourself.
Do not use `kind` for long-lived clusters or clusters that you will share with other users.

Here are the steps to set up a testsys cluster using `kind`.

Create a kind cluster (any name will suffice):

```shell
kind create cluster --name testsys
```

If you want to store the kubeconfig file, set the `KUBECONFIG` variable to some path (there should be no pre-existing file there).
It doesn't really matter where this is, since this is a throwaway cluster and then write the
kubeconfig to that path.
The environment variable `TESTSYS_KUBECONFIG` is used by all testsys
related cargo make tasks.

```shell
export TESTSYS_KUBECONFIG="${HOME}/testsys-kubeconfig.yaml"
kind get kubeconfig --name testsys > $TESTSYS_KUBECONFIG
```

Install the testsys cluster components:

```shell
cargo make setup-test
```

Testsys containers will need AWS credentials.

**Reminder**: this is for your developer workflow only, do not share this cluster with other users.

```shell
cargo make testsys add secret map  \
 --name "creds" \
 "access-key-id=$(aws configure get aws_access_key_id)" \
 "secret-access-key=$(aws configure get aws_secret_access_key)"
```

If you have a named profile you can use the following.
```shell
PROFILE=<Your desired profile name>
cargo make testsys add secret map  \
 --name "creds" \
 "access-key-id=$(aws configure get aws_access_key_id --profile ${PROFILE})" \
 "secret-access-key=$(aws configure get aws_secret_access_key --profile ${PROFILE})"
```

If you added a secret, you then need to pass the secret's name to testsys
through an environment variable:
```shell
export TESTSYS_AWS_SECRET_NAME="awsCredentials=<Name of your secret>"
```

### Conveniences

All testsys commands can be run using cargo make to eliminate the chance of 2 different versions of
testsys being used.
Testsys requires the controller and the agent images to be of the same testsys version.

```shell
cargo make testsys <arguments>
```

The Bottlerocket components are found in the `testsys` Kubernetes namespace.

## Run

Now that you have the testsys cluster set up, it's time to run a Bottlerocket integration test!

### Configuration

There are many arguments that can be configured via environment variables with `cargo make`; however, it is possible to create a configuration file instead.
Check out the [example config file](tools/testsys/Test.toml.example) for a sample `Test.toml` file.

For example, the instance type can be specified based on variant requirements:

```toml
[aws-k8s]
# Set the default instance type for all `aws-k8s` variants
instance-type = "m5.xlarge"

[aws-k8s-nvidia]
# Override the instance type for `nvidia` `aws-k8s` variants
instance-type = "g5g.2xlarge"
```

Since `aws-k8s-nvidia` is a `<FAMILY>-<FLAVOR>` level configuration it will take precedence over `aws-k8s` which is `<FAMILY>` level configuration.

Tables can also be created for custom testing configurations. For a custom test type called `foo`, the config above can be updated: 

```toml
[aws-k8s]
# Set the default instance type for all `aws-k8s` variants
instance-type = "m5.xlarge"

[aws-k8s.configuration.foo]
# Set the default instance type for all `aws-k8s` variants when `TESTSYS_TEST=foo` is set
instance-type = "m5.8xlarge"

[aws-k8s-nvidia]
# Override the instance type for `nvidia` `aws-k8s` variants
instance-type = "g5g.2xlarge"

[aws-k8s-nvidia.configuration.foo]
# Override the instance type for `nvidia` `aws-k8s` variants when `TESTSYS_TEST=foo` is set
instance-type = "g5g.8xlarge"
```

### Variants

Different Bottlerocket variants require different implementations in the test system.
For example, to ensure that Kubernetes variants are working correctly, we use [Sonobuoy] to run through the K8s E2E conformance test suite.
For ECS, we run a [task] on Bottlerocket to make sure Bottlerocket is working.
We use EC2 and EKS for `aws-k8s` variants and vSphere for `vmware-k8s` variants, and so on.

[Sonobuoy]: https://sonobuoy.io/
[task]: https://docs.aws.amazon.com/AmazonECS/latest/developerguide/welcome-features.html

We have attempted use sensible defaults for these behaviors when calling the `cargo make test` command.

### aws-k8s

You need to [build](BUILDING.md) Bottlerocket and create an AMI before you can run a test.
Change the commands below to the desired `aws-k8s` variant and AWS region:

**Caution**: An EKS cluster will be created for you.
Because these take a long time to create, the default testsys behavior is to leave this in place so you can re-use it.
You will need to delete the EKS cluster manually when you are done using it.
(EC2 instances are terminated automatically, but it's worth double-checking to make sure they were terminated.)

```shell
cargo make \
  -e BUILDSYS_VARIANT="aws-k8s-1.24" \
  -e BUILDSYS_ARCH="x86_64" \
  build

cargo make \
  -e BUILDSYS_VARIANT="aws-k8s-1.24" \
  -e BUILDSYS_ARCH="x86_64" \
  -e PUBLISH_REGIONS="us-west-2" \
  ami

cargo make \
  -e BUILDSYS_VARIANT="aws-k8s-1.24" \
  -e BUILDSYS_ARCH="x86_64" \
  test
```

```shell
cargo make watch-test
```

**Note**: You can provision nodes with karpenter by specifying `resource-agent-type = "karpenter"` in `Test.toml`.
To follow the generic mapping, use the following configuration:

```toml
[aws-k8s.configuration.karpenter]
test-type = "quick"
resource-agent-type = "karpenter"
block-device-mapping = [
    {name = "/dev/xvda", volumeType = "gp3", volumeSize = 4, deleteOnTermination = true},
    {name = "/dev/xvdb", volumeType = "gp3", volumeSize = 20, deleteOnTermination = true},
]
```

This configuration creates a new test type for all `aws-k8s` variants called `karpenter` (the string following `.configuration` in the table heading).


Before launching nodes with karpenter you will need to add the karpenter role to your cluster's `aws-auth` config map.

```bash
# Change to your clusters name
CLUSTER_NAME=my-cluster
ACCOUNT_ID=your-account-id
REGION=us-west-2
eksctl create iamidentity mapping \
  -r ${REGION} \
  --cluster ${CLUSTER_NAME} \
  --arn arn:aws:iam::${ACCOUNT_ID}:role/KarpenterInstanceNodeRole \
  --username system:node:{{EC2PrivateDNSName}} \
  --group system:bootstrappers \
  --group system:nodes
```

You can run the test by calling,

```bash
cargo make -e TESTSYS_TEST=karpenter test
```

### aws-ecs

You need to [build](BUILDING.md) Bottlerocket and create an AMI before you can run a test.
The default instance type to be used is `m5.large` for `x86_64` and `m6g.large` for `aarch64`, but can be controlled by setting the environment variable `TESTSYS_INSTANCE_TYPE`.
This is useful while testing NVIDIA variants, since they require instance types with support for NVIDIA GPUs.
Change the commands below to the desired `aws-ecs` variant and AWS region:

```shell
cargo make \
  -e BUILDSYS_VARIANT="aws-ecs-1" \
  -e BUILDSYS_ARCH="x86_64" \
  build

cargo make \
  -e BUILDSYS_VARIANT="aws-ecs-1" \
  -e BUILDSYS_ARCH="x86_64" \
  -e PUBLISH_REGIONS="us-west-2" \
  ami

cargo make \
  -e BUILDSYS_VARIANT="aws-ecs-1" \
  -e BUILDSYS_ARCH="x86_64" \
  test
```

```shell
cargo make watch-test
```

**Note:** For more information on publishing AMIs see [publishing](PUBLISHING.md).

### vmware-k8s

First, an initial management cluster needs to be created using [`EKS Anywhere`](https://anywhere.eks.amazonaws.com/docs/getting-started/production-environment/vsphere-getstarted/#create-an-initial-cluster).
You can then set `TESTSYS_MGMT_CLUSTER_KUBECONFIG` to the path to the management clusters kubeconfig.
You need to [build](BUILDING.md) Bottlerocket and a publicly accessible [TUF repository](https://github.com/bottlerocket-os/bottlerocket/blob/develop/PUBLISHING.md#repo-location) to test VMware variants.
Either `Infra.toml` or your environment need to be configured.
If using environment variables make sure to set the following environment variables:
- GOVC_URL
- GOVC_USERNAME
- GOVC_PASSWORD
- GOVC_DATACENTER
- GOVC_DATASTORE
- GOVC_NETWORK
- GOVC_RESOURCE_POOL
- GOVC_FOLDER

Testsys will use the data center specified in `Test.toml` first.
If no data center is specified in `Test.toml`, testsys will use the first data center listed in `Infra.toml`
VMware testing also requires a `control-plane-endpoint` to be set in `Test.toml` for vSphere K8s cluster creation.
Change the commands below to the desired `vmware-k8s` variant:

First, build the VMware variant you want to test.

```shell
cargo make \
  -e BUILDSYS_VARIANT="vmware-k8s-1.23" \
  -e BUILDSYS_ARCH="x86_64" \
  build
```

Build the TUF repo containing the OVA templates.

```shell
cargo make \
  -e BUILDSYS_VARIANT="vmware-k8s-1.23" \
  -e BUILDSYS_ARCH="x86_64" \
  repo
```

Sync TUF repos containing the VMware variant's metadata and targets.
Make sure the TUF repos are accessible via unauthenticated HTTP or HTTPS and match the URLs in `Infra.toml`.

Now, you can run the test.

```shell
cargo make \
  -e BUILDSYS_VARIANT="vmware-k8s-1.23" \
  -e BUILDSYS_ARCH="x86_64" \
  test \
  --mgmt-cluster-kubeconfig ${TESTSYS_MGMT_CLUSTER_KUBECONFIG}
```

You can monitor the tests with:

```shell
cargo make watch-test
```

### metal-k8s

First, an initial baremetal management cluster needs to be created using [`EKS Anywhere`](https://anywhere.eks.amazonaws.com/docs/getting-started/production-environment/baremetal-getstarted/#create-an-initial-cluster).
You can then set `TESTSYS_MGMT_CLUSTER_KUBECONFIG` to the path to the management clusters kubeconfig.
You need to [build](BUILDING.md) Bottlerocket and a publicly accessible [TUF repository](https://github.com/bottlerocket-os/bottlerocket/blob/develop/PUBLISHING.md#repo-location) to test metal variants.
In addition to the management cluster, you will need to [prepare a hardware CSV file](https://anywhere.eks.amazonaws.com/docs/reference/baremetal/bare-preparation/#prepare-hardware-inventory) containing all machines you want to provision and a [cluster config](https://anywhere.eks.amazonaws.com/docs/reference/clusterspec/baremetal/) for the cluster.
Create a directory in `tests/shared/clusters` with an identifier for this cluster, i.e cluster1 (`tests/shared/clusters/cluster1`).
In that directory create 2 files, `cluster.yaml` with the EKS Anywhere cluster config, and `hardware.csv`.
In `Test.toml` set `cluster-names = ["cluster1"]` to tell TestSys that we want the cluster config and hardware csv from the directory we just created.

Metal testing also requires and additional manual step for testing.
The Bottlerocket build system compresses the metal images with lz4, but EKS Anywhere requires them to be gzipped, so before testing make sure to uncompress the lz4 image and gzip it.
Make sure it is downloadable from a URL accessible from the management cluster.
The directory used should be added to `Test.toml` as `os-image-dir`.

Change the commands below to the desired `metal-k8s` variant:

First, build the Metal variant you want to test.

```shell
cargo make \
  -e BUILDSYS_VARIANT="metal-k8s-1.23" \
  -e BUILDSYS_ARCH="x86_64" \
  build
```

Build the TUF repo containing the metal images.

```shell
cargo make \
  -e BUILDSYS_VARIANT="metal-k8s-1.23" \
  -e BUILDSYS_ARCH="x86_64" \
  repo
```

Make sure you gzip the metal image and add it to your `os-image-dir`.

Now, you can run the test.

```shell
cargo make \
  -e BUILDSYS_VARIANT="metal-k8s-1.23" \
  -e BUILDSYS_ARCH="x86_64" \
  -e TESTSYS_MGMT_CLUSTER_KUBECONFIG=${TESTSYS_MGMT_CLUSTER_KUBECONFIG}
  test
```

You can monitor the tests with:

```shell
cargo make watch-test
```

## Migration Testing

Migration testing is used to ensure Bottlerocket can update from one version to a new version and back.
This involves launching Bottlerocket instances, upgrading them, and downgrading them.

Migration testing launches instances of a starting Bottlerocket version, or a provided initial AMI and migrates instances to the target version.
In order to accomplish this a few artifacts need to be created:
* A publicly accessible TUF repository
* A previous release of Bottlerocket signed with available keys
* The AMI ID for the previous release
* Image artifacts and local TUF repos of said artifacts for current changes

### The setup

#### Prepare `Infra.toml`

We need the URL of an accessible TUF repo so the Bottlerocket instances know where to retrieve the update metadata and targets.
Follow our [publishing guide](PUBLISHING.md#repo-location) to set up TUF repos.
`Infra.toml` is used by testsys to determine TUF repo locations, so `metadata_base_url` and `targets_base_url` need to be set based on the repo that was just created.
The examples below also assume that the default repo is being used in `Infra.toml`, but any repo can be used by setting the `PUBLISH_REPO` environment variable.

#### Starting Bottlerocket images

In this example we will use `v1.9.0` as our starting Bottlerocket version, but any tag from Bottlerocket will work.
The following bash script will checkout the proper branch from git and create the build images and TUF repos for testing.

```shell
git checkout "v1.9.0"
cargo make
cargo make ami
cargo make repo
```

Remember to sync your TUF repos with the new metadata and targets.

#### Target Bottlerocket images

Now, it's time to create the Bottlerocket artifacts that need to be upgraded to.

Switch to the working git branch that should be built from.

```shell
WORKING_BRANCH="develop"
git checkout "${WORKING_BRANCH}"
```

Next, build Bottlerocket images and repos and sync TUF repos.
The architecture and variant can be configured with `BUILDSYS_ARCH` and `BUILDSYS_VARIANT`.

```shell
cargo make
cargo make ami
cargo make repo
```

Now, sync your TUF repos with the new metadata and targets.

This completes the setup and it is time to test migrations!

### Testing Migrations

The previous steps set up the artifacts necessary to perform migration testing using `testsys`.
Ensure all environment variables are still set and set them if they aren't.

To run the migration test set `TESTSYS_TEST=migration` in the `cargo make test` call.
This will automatically determine the AMI that should be used by finding the latest released version of bottlerocket and checking the user's AMIs to find the correct starting AMI ID.
Remember to set the environment variables for the architecture and variant.

```shell
cargo make -e TESTSYS_TEST=migration test
```

To see the state of the tests as they run use `cargo make watch-test`.

### Testing Workloads

Workload tests are tests designed to run as an orchestrated container.
A workload test is defined in `Test.toml` with a map named `workloads`.

```toml
[aws-nvidia]
workloads = { <WORKLOAD-NAME> = "<WORKLOAD-IMAGE-URI>" }
```

To run the workload test set `TESTSYS_TEST=workload` in the `cargo make test` call.

```shell
cargo make -e TESTSYS_TEST=workload test
```

To see the state of the tests as they run use `cargo make watch-test`.

For more information can be found in the [TestSys workload documentation](https://github.com/bottlerocket-os/bottlerocket-test-system/tree/develop/bottlerocket/tests/workload).

### Custom Test Types

Custom tests can be run with TestSys by calling `cargo make -e TESTSYS_TEST=<CUSTOM-TEST-NAME> test -f <PATH-TO-TEMPLATED-YAML>`.

First, a test agent needs to be constructed.
The `test-agent-cli` provides an interface for creating bash based testing agents.
Checkout the [runbook](https://github.com/bottlerocket-os/bottlerocket-test-system/blob/develop/agent/test-agent-cli/design/RUNBOOK.md) for instructions on creating an agent.

Once an agent has been created, the yaml template can be created.
Values from `Test.toml` can be inserted into a yaml manifest so that a single manifest can be used for all variants in a family.

```yaml
apiVersion: {{api-version}}
kind: Test
metadata:
  # The name of the crd created is dependent on the arch and variant for 
  # the test being run.
  name: {{kube-arch}}-{{kube-variant}}-custom
  namespace: {{namespace}}
spec:
  retries: 5
  agent:
    name: custom-test-agent
    image: example-test-agent-cli:latest
    keepRunning: false
    configuration:
      clusterName: {{cluster-name}}
      instanceType: {{instance-type}}
  resources: []
  dependsOn: []
  # The secrets will automatically be populated from the config file, 
  # no template is needed.
  secrets: {}
```

After the agent has been build and the yaml file is created, the test can be run using `cargo make -e TESTSYS_TEST=<CUSTOM-TEST-NAME> test -f <PATH-TO-YAML-FILE>` 
