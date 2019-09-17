package intent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	v1 "k8s.io/api/core/v1"
)

var _ marker.Container = (*Intent)(nil)

type Intent struct {
	// NodeName is the Resource name that addresses it.
	NodeName string
	// CurrentAction is the currently instructed action on the node.
	Wanted marker.NodeAction
	// CurrentState is the current action taken on the Node.
	Active marker.NodeAction
	// State is the current status or state of the Node reported, generally for
	// the Active action.
	State marker.NodeState
	// UpdateAvailable is the Node's status of having an update ready to apply.
	UpdateAvailable marker.NodeUpdate
}

func (i *Intent) GetAnnotations() map[string]string {
	return map[string]string{
		marker.NodeActionWanted:      i.Wanted,
		marker.NodeActionActive:      i.Active,
		marker.NodeActionActiveState: i.State,
		marker.UpdateAvailableKey:    i.UpdateAvailable,
	}
}

func (i *Intent) GetLabels() map[string]string {
	return map[string]string{}
}

// Waiting reports true when a Node is prepared and waiting to make further
// commanded progress towards completing an update.
func (i *Intent) Waiting() bool {
	var isReady bool
	switch i.State {
	case marker.NodeStateReady:
		isReady = true
	case "", marker.NodeStateUnknown:
		// Node may need to be commanded to become intentional.
		isReady = true
	case marker.NodeStateError:
		// error state indicates that the node is ready to be error handled
		isReady = true
	}
	isProgressed := i.Active == i.Wanted
	return isReady && isProgressed
}

func (i *Intent) Intrusive() bool {
	switch i.Wanted {
	case marker.NodeActionPrepareUpdate,
		marker.NodeActionPerformUpdate,
		marker.NodeActionRebootUpdate:
		return true
	}
	return false
}

func (i *Intent) Errored() bool {
	sameState := i.Active == i.Wanted
	isError := i.State == marker.NodeStateError
	return isError && sameState
}

func (i *Intent) WantProgress() bool {
	notCurrent := i.Wanted != i.Active
	return notCurrent && i.isInProgress(i.Wanted)
}

// InProgess reports true when the Intent is for a node that is making progress
// towards completing an update.
func (i *Intent) InProgress() bool {
	return i.isInProgress(i.Active)
}

func (i *Intent) isInProgress(field marker.NodeAction) bool {
	switch field {
	case marker.NodeActionReset,
		marker.NodeActionStablize,
		marker.NodeActionPrepareUpdate,
		marker.NodeActionPerformUpdate,
		marker.NodeActionRebootUpdate:
		return true
	}
	return false
}

func (i *Intent) HasUpdateAvailable() bool {
	return i.UpdateAvailable == marker.NodeUpdateAvailable
}

// Needed indicates that the intent is needing progress made on it.
func (i *Intent) Needed() bool {
	// If the node has an update, the intent is needing progress.
	readyToMakeProgress := i.Waiting() && i.InProgress()
	return readyToMakeProgress
}

// Projected returns the n+1 step projection of a would-be Intent. It does not
// check whether or not the next step is *sane* given the current intent (ie:
// this will not error if the node has not actually completed a step).
func (i Intent) Projected() *Intent {
	i.Wanted = calculateNext(i.Wanted)
	return &i
}

func (i Intent) Clone() *Intent {
	return &i
}

// Given determines the commuincated intent from a Node without projecting into
// its next steps.
func Given(node *v1.Node) *Intent {
	annos := node.GetAnnotations()

	intent := &Intent{
		NodeName:        node.GetName(),
		Active:          annos[marker.NodeActionActive],
		Wanted:          annos[marker.NodeActionWanted],
		State:           annos[marker.NodeActionActiveState],
		UpdateAvailable: annos[marker.UpdateAvailableKey],
	}

	return intent
}
