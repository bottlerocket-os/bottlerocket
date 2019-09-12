package intent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	v1 "k8s.io/api/core/v1"
)

type Intent struct {
	// NodeName is an identifier that uniquely addresses the intent target.
	NodeName string

	// CurrentAction is the currently instructed action on the node.
	CurrentAction marker.NodeAction
	// CurrentState is the current state of the node.
	CurrentState marker.NodeState

	// Action is the intended next action to be instructed.
	Action marker.NodeAction
	// State is the state that would be reached if the intent were
	// progressed upon.
	State marker.NodeState
}

func (i *Intent) Active() bool {
	switch i.CurrentAction {
	case marker.NodeActionReset,
		marker.NodeActionPrepareUpdate,
		marker.NodeActionPerformUpdate,
		marker.NodeActionRebootUpdate:
		return true
	}

	switch i.CurrentState {
	case marker.NodeStateRebooting:
		return true
	}

	return false
}

// IsNeeded indicates that the intent is needing progress made on it.
func (i *Intent) Pending() bool {
	// If its active, then its an intent pending action to make it inactive.
	if i.Active() {
		return true
	}
	// If the node has an update, the intent is needing progress.
	switch i.CurrentState {
	case marker.NodeStateUpdateAvailable:
		return true
	}
	// Otherwise, the node can sit tight.
	return false
}

// Next returns the n+1 step Intent to be communicated.
func (i Intent) Next() *Intent {
	i.State, i.Action = calculateNext(i.CurrentState, i.CurrentAction)
	return &i
}

// Given determines the commuincated intent from a Node Resource without
// extrapolating into the next steps.
func Given(node *v1.Node) *Intent {
	annos := node.GetAnnotations()

	intent := &Intent{
		NodeName: node.GetName(),

		CurrentState:  annos[marker.NodeActionActive],
		CurrentAction: annos[marker.NodeActionWanted],

		State:  annos[marker.NodeActionActive],
		Action: annos[marker.NodeActionWanted],
	}

	return intent
}
