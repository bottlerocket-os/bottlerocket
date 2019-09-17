package intent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
)

// TODO: encapsulate state machine-y handling. Callers should not have to
// reference marker to compare the needed state nor set the necessary response.

// Intent is a pseudo-Container of Labels and Annotations.
var _ marker.Container = (*Intent)(nil)

// Intent is the sole communicator of state progression and desired intentions
// for an Agent to act upon and to communicate its progress.
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

// GetName returns the name of the Intent's target.
func (i *Intent) GetName() string {
	return i.NodeName
}

// GetAnnotations transposes the Intent into a map of Annotations suitable for
// adding to a Resource.
func (i *Intent) GetAnnotations() map[string]string {
	return map[string]string{
		marker.NodeActionWanted:      i.Wanted,
		marker.NodeActionActive:      i.Active,
		marker.NodeActionActiveState: i.State,
		marker.UpdateAvailableKey:    i.UpdateAvailable,
	}
}

// GetLabels transposes the Intent into a map of Labels suitable for adding to a
// Resource.
func (i *Intent) GetLabels() map[string]string {
	return map[string]string{
		marker.UpdateAvailableKey: i.UpdateAvailable,
	}
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

// Intrusive indicates that the intention will be intrusive if realized.
func (i *Intent) Intrusive() bool {
	switch i.Wanted {
	case marker.NodeActionPrepareUpdate,
		marker.NodeActionPerformUpdate,
		marker.NodeActionRebootUpdate:
		return true
	}
	return false
}

// Errored indicates that the intention was not realized and failed in attempt
// to do so.
func (i *Intent) Errored() bool {
	sameState := i.Active == i.Wanted
	isError := i.State == marker.NodeStateError
	return isError && sameState
}

// WantProgress indicates that the intention is wanted but not actively being
// progressed towards.
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

// HasUpdateAvailable indicates the Node has flagged itself as needing an
// update.
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
func (i *Intent) Projected() *Intent {
	p := i.Clone()
	if p.inUnknownState() {
		p.reset()
	} else {
		p.Wanted, _ = calculateNext(p.Wanted)
	}
	return p
}

// reset reverts the Intent to its Origin point from which an Intent should be
// able to be driven to a Terminal point.
func (i *Intent) reset() {
	i.Wanted, _ = calculateNext(marker.NodeActionUnknown)
	i.Active = marker.NodeActionUnknown
	i.State = marker.NodeStateUnknown
}

func (i *Intent) inUnknownState() bool {
	return i.State == "" ||
		i.State == marker.NodeStateUnknown
}

func (i Intent) Terminal() bool {
	next, err := calculateNext(i.Wanted)
	if err != nil {
		return false
	}
	// The next turn in the state machine is the same as the realized Wanted and
	// Active states, therefore we've reached a terminal point.
	matchesTerminal := next == i.Wanted && i.Wanted == i.Active
	return matchesTerminal
}

// Clone returns a copy of the Intent to mutate independently of the source
// instance.
func (i Intent) Clone() *Intent {
	return &i
}

// Given determines the commuincated intent from a Node without projecting into
// its next steps.
func Given(input Input) *Intent {
	annos := input.GetAnnotations()

	intent := &Intent{
		NodeName:        input.GetName(),
		Active:          annos[marker.NodeActionActive],
		Wanted:          annos[marker.NodeActionWanted],
		State:           annos[marker.NodeActionActiveState],
		UpdateAvailable: annos[marker.UpdateAvailableKey],
	}

	return intent
}

// Input is a suitable container of data for interpreting an Intent from. This
// effectively is a subset of the kubernetes' v1.Node accessors, but is more
// succinct and scoped to the used accessors.
type Input interface {
	marker.Container
	GetName() string
}
