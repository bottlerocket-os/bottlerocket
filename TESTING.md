# Testing Bottlerocket

🚧 👷

This section is under active development.
We are working on tooling for running Bottlerocket integration tests.
While the work is underway, there will be frequent changes to this document.

## Unit Tests

Unit tests are easy, you can run them from the root of the repo with `cargo make unit-tests`.
Note that some code in Bottlerocket is conditionally compiled based on variant.
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

To do this, testsys uses a Kubernetes operator.
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
First, set the `KUBECONFIG` variable to some path (there should be no pre-existing file there).
It doesn't really matter where this is, since this is a throwaway cluster.

```shell
export KUBECONFIG="${HOME}/testsys-kubeconfig.yaml"
```

Create a kind cluster (any name will suffice):

```shell
kind create cluster --name testsys
```

Install the testsys cluster components:

```shell
cargo make setup-testsys
```

Testsys containers will need AWS credentials.

**Reminder**: this is for your developer workflow only, do not share this cluster with other users.

```shell
./tools/bin/testsys add secret map  \
 --name "creds" \
 "access-key-id=$(aws configure get default.aws_access_key_id)" \
 "secret-access-key=$(aws configure get default.aws_secret_access_key)"
```

### Conveniences

Optional: If you want to use the testsys command without its `cargo make` wrapper, you can create a symlink in your system path.
This would be better than installing another copy of it since we want to make sure we stay in sync with the version used by Bottlerocket's Makefile.toml.

From the root of the `bottlerocket` repo, you could do this:

```shell
sudo ln -s "$(pwd)/tools/bin/testsys" "/usr/local/bin/testsys"
which testsys
```

The Bottlerocket components are found in the `testsys-bottlerocket-aws` Kubernetes namespace.
This is a lot to type everytime you use kubectl.
Instead, you can set it as the default namespace in your kubeconfig context.

```shell
kubectl config set-context --current --namespace="testsys-bottlerocket-aws"
```

## Run

Now that you have the testsys cluster set up, it's time to run a Bottlerocket integration test!

### Variants

Different Bottlerocket variants require different implementations in the test system.
For example, to ensure that Kubernetes variants are working correctly, we use [Sonobuoy].
For ECS, we run a [task] on Bottlerocket to make sure Bottlerocket is working.
We use EC2 and EKS for `aws-k8s` variants and `vSphere` for `vmware-k8s` variants, and so on.

[Sonobuoy]: https://sonobuoy.io/
[task]: https://docs.aws.amazon.com/AmazonECS/latest/developerguide/welcome-features.html

We have attempted use sensible defaults for these behaviors when calling the `cargo make test` command.

🚧 👷 **Variant Support**

We haven't yet enabled `cargo make test` for every variant, though much of the underlying foundation has been laid.
If you run `cargo make test` for a variant that is not yet enabled, it will print an error message.
Check back here and follow the issues relevant to your variant of interest.

- `aws-k8s` conformance testing is working!
- `aws-ecs`: https://github.com/bottlerocket-os/bottlerocket/issues/2150
- `vmware-k8s`: https://github.com/bottlerocket-os/bottlerocket/issues/2151
- `metal-k8s`: https://github.com/bottlerocket-os/bottlerocket/issues/2152

### aws-k8s

You need to build Bottlerocket and create an AMI before you can run a test.
Change the commands below to the desired `aws-k8s` variant and AWS region:

**Caution**: An EKS cluster will be created for you.
Because these take a long time to create, the default testsys behavior is to leave this in place so you can re-use it.
You will need to delete the EKS cluster manually when you are done using it.
(EC2 instances are terminated automatically, but it's worth double-checking to make sure they were terminated.)

```shell
cargo make \
  -e BUILDSYS_VARIANT=aws-k8s-1.21 \
  -e BUILDSYS_ARCH="x86_64" \
  build
  
cargo make \
  -e BUILDSYS_VARIANT=aws-k8s-1.21 \
  -e BUILDSYS_ARCH="x86_64" \
  -e PUBLISH_REGIONS="us-west-2"
  ami
 
cargo make \
  -e BUILDSYS_VARIANT=aws-k8s-1.21 \
  -e BUILDSYS_ARCH="x86_64" \
  test
```

Once the test is running, you can monitor its progress with the following
(`-r` tells testsys to also the status of resources like the cluster and instances in addition to tests):

```shell
watch ./tools/bin/testsys status -r
```
