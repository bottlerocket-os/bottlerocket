# Dogswatch: Update Operator

Dogswatch is a [Kubernetes operator](https://Kubernetes.io/docs/concepts/extend-Kubernetes/operator/) that coordinates update activities on Thar hosts in a Kubernetes cluster. 

## How to Run on Kubernetes


To run the Dogswatch Operator in your Kubernetes cluster, the following are required resources and configuration (examples given in the [./dev/deployment.yaml](./dev/deployment.yaml) template):

- **`dogswatch` Container Image**
  
  Holding the Dogswatch binaries and its supporting environment.

- **Controller Deployment**

  Scheduling a stop-restart-tolerant Controller process on available Nodes.

- **Agent DaemonSet**
  
  Scheduling Agent on Thar hosts

- **Thar Namespace**

  Grouping Thar related resources and roles.

- **Service Account for the Agent**

  Configured for authenticating the Agent process on Kubernetes APIs.

- **Cluster privileged credentials with read-write access to Nodes for Agent**
  
  Applied to Agent Service Account to update annotations on the Node resource that the Agent is running under.
  
- **Service Account for the Controller**

  Configured for authenticating the Controller process on Kubernetes APIs.
  
- **Cluster privileged credentials with access to Pods and Nodes for Controller**

  Applied to the Controller Service Account for manipulating annotations on Node resources as well as cordon & uncordoning for updates.
  The Controller also must be able to un-schedule (`delete`) Pods running on Nodes that will be updated.

In the [./dev/deployment.yaml example](./dev/deployment.yaml), the resource specifies the conditions that the Kubernetes Schedulers will place them in the Cluster.
These conditions include the Node being labeled as having the required level of support for the Operator to function on it: the `thar.amazonaws.com/platform-version` label.
With this label present and the workloads scheduled, the Agent and Controller process will coordinate an update as soon as the Agent annotates its Node (by default only one update will happen at a time).

To use the example [./dev/deployment.yaml](./dev/deployment.yaml) as a base, you must modify the resources to use the appropriate container image that is available to your kubelets (a common image is forthcoming, see #505).
Then with a appropriately configured deployment yaml, you may call `kubelet apply -f ./my-deployment.yaml` to prepare the above resources and schedule the Dogswatch Pods in your Cluster.

## What Makes Up Dogswatch

Dogswatch is made up of two distinct processes, one of which runs on each host.

- `dogswatch -controller`
    
  The coordinating process responsible for the handling update of Thar nodes
  cooperatively with the cluster's workloads.
  
- `dogswatch -agent`
  
  The on-host process responsible for publishing update metadata and executing
  update activities.
  
## How It Coordinates

The Dogswatch processes communicate by applying updates to the Kubernetes Node resources' Annotations. 
The Annotations are used to communicate the Agent activity (called an `intent`) as determined by the Controller process, the current Agent activity in response to the intent, and the Host's update status
as known by the Agent process.

The Agent and Controller processes listen to an event stream from the Kubernetes cluster in order to quickly and reliably handle communicated `intent` in addition to updated metadata pertinent to updates and the Operator itself.

### Current Limitations

- Pod replication & healthy count is not taken into consideration (#502)
- Nodes update without pause between each (#503)
- Single Node cluster degrades into unscheduleable on update (#501)
- Node labels are not automatically applied to allow scheduling (#504)

## How to Contribute and Develop Changes for Dogswatch

Working on Dogswatch requires a fully functioning Kubernetes cluster. 
For the sake of development workflow, you may easily run this within a container or VM as with [`kind`](https://github.com/Kubernetes-sigs/kind) or [`minikube`](https://github.com/Kubernetes/minikube). 
The `dev/` directory contains several resources that may be used for development and debugging purposes:

- `dashboard.yaml` - A **development environment** set of Kubernetes resources (these use insecure settings and *are not suitable for use in Production*!)
- `deployment.yaml` - A _template_ for Kubernetes resources for Dogswatch that schedule a controller and setup a DaemonSet
- `kind-cluster.yml` - A `kind` Cluster definition that may be used to stand up a local development cluster

Much of the development workflow can be accommodated by the `Makefile` providedalongside the code. 
Each of these targets utilize your existing environment and tools - for example: your `kubectl` as configured will be used. 
If you have locally configured access to production, please ensure you've taken steps to reconfigure or otherwise cause `kubectl` to affect only your development cluster.

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
