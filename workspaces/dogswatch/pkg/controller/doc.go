// Controller manages the state transitions of the node's agent which itself
// integrates with the Thar platform. The controller takes cluster state into
// account and manages the nodes at arms length by encoding agreed upon state
// transitions into a combination of labels and annotations stored as metadata
// on the Kubernetes Resource Object for each Node.
//
// Currently, this controller is capable of:
//
// - coordinating host updates such that no two nodes are performing an update
//   simultaneously
//
// The controller IS NOT capable of (yet!):
//
// - ensuring that workloads are sufficiently replicated or distributed to avoid
//   service outtages.
//
// - executing transitions during a set maintenance period
//
// - executing transitions with a bake time
//
// - configuring cluster wide update settings (normally, and currently, set via
//   user data)
//
package controller
