# Dogswatch: Update Operator

Dogswatch is a [Kubernetes operator](https://Kubernetes.io/docs/concepts/extend-Kubernetes/operator/) that coordinates update activities on Bottlerocket hosts in a Kubernetes cluster.

## How to Run on Kubernetes


To run the Dogswatch Operator in a Kubernetes cluster, the following are required resources and configuration ([suggested deployment is defined in `dogswatch.yaml`](./dogswatch.yaml)):

- **`dogswatch` Container Image**

  Holding the Dogswatch binaries and its supporting environment.

- **Controller Deployment**

  Scheduling a stop-restart-tolerant Controller process on available Nodes.

- **Agent DaemonSet**

  Scheduling Agent on Bottlerocket hosts

- **Bottlerocket Namespace**

  Grouping Bottlerocket related resources and roles.

- **Service Account for the Agent**

  Configured for authenticating the Agent process on Kubernetes APIs.

- **Cluster privileged credentials with read-write access to Nodes for Agent**

  Applied to Agent Service Account to update annotations on the Node resource that the Agent is running under.

- **Service Account for the Controller**

  Configured for authenticating the Controller process on Kubernetes APIs.

- **Cluster privileged credentials with access to Pods and Nodes for Controller**

  Applied to the Controller Service Account for manipulating annotations on Node resources as well as cordon & uncordoning for updates.
  The Controller also must be able to un-schedule (`delete`) Pods running on Nodes that will be updated.

Cluster administrators can deploy dogswatch with [suggested configuration defined here](./dogswatch.yaml) - this includes the above resources and Bottlerocket published container images.
The dogswatch deployment can be applied to a cluster by calling `kubectl apply -f ./dogswatch.yaml` with an appropriately configured `kubectl` client for the target cluster.

Once resources are in place one last step is required to let the Kubernetes schedule place the required Pods.
The deployments control scheduling of the dogswatch pods by limiting Pods to appropriate Bottlerocket hosts using labels.
For now, these labels are not applied automatically at boot and will need to be set on each Node resource using a tool like `kubectl`.

Each Node that is running Bottlerocket must be labeled with the Node's `platform-version` (a host compatibility indicator) in order to have `dogswatch` Pods scheduled on them, the label `bottlerocket.amazonaws.com/platform-version` is used for this:

``` text
bottlerocket.amazonaws.com/platform-version=1.0.0
```

`kubectl` may be used to set this label on a Node:

``` sh
: kubectl label node $NODE_NAME bottlerocket.amazonaws.com/platform-version=1.0.0
```

If all Nodes in the cluster are running Bottlerocket, they can all be labeled at the same time with a single command:

``` sh
: kubectl label node $(kubectl get nodes -o jsonpath='{.items[*].metadata.name}') bottlerocket.amazonaws.com/platform-version=1.0.0
```

In the [development example deployment](./dev/deployment.yaml) the resources specify conditions that the Kubernetes Schedulers uses to place Pods in the Cluster.
These conditions, among others, include a constraint on each Node being labeled as having support for the Operator to function on it: the `bottlerocket.amazonaws.com/platform-version` label.
With this label present and the workloads scheduled, the Agent and Controller process will coordinate an update as soon as the Agent annotates its Node (by default only one update will happen at a time).

To use the [suggested deployment](./dogswatch.yaml) or [development deployment](./dev/deployment.yaml) as a base, any customized resources must be updated to use a customized container image to run.
Then with the configured deployment, use `kubelet apply -f $UPDATED_DEPLOYMENT.yaml` to prepare the above resources and schedule the Dogswatch Pods in a Cluster.

## What Makes Up Dogswatch

Dogswatch is made up of two distinct processes, one of which runs on each host.

- `dogswatch -controller`

  The coordinating process responsible for the handling update of Bottlerocket nodes
  cooperatively with the cluster's workloads.

- `dogswatch -agent`

  The on-host process responsible for publishing update metadata and executing
  update activities.

## How It Coordinates

The Dogswatch processes communicate by applying updates to the Kubernetes Node resources' Annotations.
The Annotations are used to communicate the Agent activity (called an `intent`) as determined by the Controller process, the current Agent activity in response to the intent, and the Host's update status
as known by the Agent process.

The Agent and Controller processes listen to an event stream from the Kubernetes cluster in order to quickly and reliably handle communicated `intent` in addition to updated metadata pertinent to updates and the Operator itself.

### Observing Progress and State

Dogwatch's operation can be simply observed by inspecting the labels and annotations on the Node resource.
The state and pending activity are posted as progress is made.

``` sh
# With a configured kubectl and jq available on $PATH
kubectl get nodes -o json \
  | jq -C -S '.items | map(.metadata|{(.name): (.annotations*.labels|to_entries|map(select(.key|startswith("bottlerocket")))|from_entries)}) | add'
```

### Current Limitations

- Pod replication & healthy count is not taken into consideration (#502)
- Nodes update without pause between each Node (#503)
- Single Node cluster degrades into unscheduleable on update (#501)
- Node labels are not automatically applied to allow scheduling (#504)

## How to Contribute and Develop Changes for Dogswatch

Working on Dogswatch requires a fully configured, working Kubernetes cluster.
For the sake of development workflow, we suggest using a cluster that is containerized or virtualized - tools to manage these are available: [`kind`](https://github.com/Kubernetes-sigs/kind) (containerized) and [`minikube`](https://github.com/Kubernetes/minikube) (virtualized).
The `dev/` directory contains several resources that may be used for development and debugging purposes:

- `dashboard.yaml` - A **development environment** set of Kubernetes resources (these use insecure settings and *are not suitable for use in Production*!)
- `deployment.yaml` - A _template_ for Kubernetes resources for Dogswatch that schedule a controller and setup a DaemonSet
- `kind-cluster.yml` - A `kind` Cluster definition that may be used to stand up a local development cluster

Much of the development workflow can be accommodated by the `Makefile` provided alongside the code.
Each of these targets utilize tools and environments they're configured to access - for example: `kubectl`, as configured on a host, will be used.
If `kubectl` is configured to configured with access to production, please ensure take steps to reconfigure `kubectl` to affect only a development cluster.

**General use targets**

- `container` - build a container image used by the Kubernetes resources
- `dashboard` - create or update Kubernetes-dashboard (*not suitable for use in Production*)
- `deploy` - create or update Dogswatch Kubernetes resources
- `rollout` - reload and restart Dogswatch processes in the cluster
- `test` - run `go test` against `dogswatch`

**`kind` development targets**

- `load`
- `cluster`
- `rollout-kind`
