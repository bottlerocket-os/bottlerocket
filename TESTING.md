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
testsys bing used. 
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

```yaml
[aws-k8s]
# Set the default instance type for all `aws-k8s` variants
instance-type = "m5.xlarge"

[aws-k8s-nvidia]
# Override the instance type for `nvidia` `aws-k8s` variants
instance-type = "g5g.2xlarge"
```

Since `aws-k8s-nvidia` is a `<FAMILY>-<FLAVOR>` level configuration it will take precedence over `aws-k8s` which is `<FAMILY>` level configuration.

### Variants

Different Bottlerocket variants require different implementations in the test system.
For example, to ensure that Kubernetes variants are working correctly, we use [Sonobuoy] to run through the K8s E2E conformance test suite.
For ECS, we run a [task] on Bottlerocket to make sure Bottlerocket is working.
We use EC2 and EKS for `aws-k8s` variants and vSphere for `vmware-k8s` variants, and so on.

[Sonobuoy]: https://sonobuoy.io/
[task]: https://docs.aws.amazon.com/AmazonECS/latest/developerguide/welcome-features.html

We have attempted use sensible defaults for these behaviors when calling the `cargo make test` command.

🚧 👷 **Variant Support**

We haven't yet enabled `cargo make test` for every variant, though much of the underlying foundation has been laid.
If you run `cargo make test` for a variant that is not yet enabled, it will print an error message.
Check back here and follow the issues relevant to your variant of interest.

- `aws-k8s` conformance testing is working!
- `aws-ecs`: quick and migration testing are working!
- `vmware-k8s`: https://github.com/bottlerocket-os/bottlerocket/issues/2151
- `metal-k8s`: https://github.com/bottlerocket-os/bottlerocket/issues/2152

### aws-k8s

You need to [build](BUILDING.md) Bottlerocket and create an AMI before you can run a test.
Change the commands below to the desired `aws-k8s` variant and AWS region:

**Caution**: An EKS cluster will be created for you.
Because these take a long time to create, the default testsys behavior is to leave this in place so you can re-use it.
You will need to delete the EKS cluster manually when you are done using it.
(EC2 instances are terminated automatically, but it's worth double-checking to make sure they were terminated.)

```shell
cargo make \
  -e BUILDSYS_VARIANT="aws-k8s-1.21" \
  -e BUILDSYS_ARCH="x86_64" \
  build
  
cargo make \
  -e BUILDSYS_VARIANT="aws-k8s-1.21" \
  -e BUILDSYS_ARCH="x86_64" \
  -e PUBLISH_REGIONS="us-west-2"
  ami
 
cargo make \
  -e BUILDSYS_VARIANT="aws-k8s-1.21" \
  -e BUILDSYS_ARCH="x86_64" \
  test
```

```shell
cargo make watch-test
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
  -e PUBLISH_REGIONS="us-west-2"
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
