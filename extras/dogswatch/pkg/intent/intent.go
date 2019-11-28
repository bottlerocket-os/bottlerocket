package intent

import (
	"fmt"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/sirupsen/logrus"
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
// commanded progress towards completing an update. This doesn't however
// indicate whether or not the intent reached a waiting state successfully or
// not. Utilize other checks to assert a combinational check.
func (i *Intent) Waiting() bool {
	var done bool
	switch i.State {
	case marker.NodeStateReady:
		// Ready for action, probably waiting for next command.
		done = true
	case "", marker.NodeStateUnknown:
		// Node is an unknown state or doesn't yet have a state. This is likely
		// because there hasn't been a requested action or because it hasn't yet
		// done something to warrant action.
		done = true
	case marker.NodeStateError:
		// Node errored and is waiting on a next step.
		done = true
	default:
		// The state is unclear, other predicates may better inform the caller
		// of the scenario its handling.
		done = false
	}
	return done
}

// Intrusive indicates that the intention will be intrusive if realized.
func (i *Intent) Intrusive() bool {
	rebooting := i.Wanted == marker.NodeActionRebootUpdate && !i.Realized()
	return rebooting
}

// Errored indicates that the intention was not realized and failed in attempt
// to do so.
func (i *Intent) Errored() bool {
	errored := i.State == marker.NodeStateError
	return errored
}

// Stuck intents are those that cannot be realized or are terminal in their
// current state and should be unstuck. If terminal handling is needed, the
// caller should use Intent.Terminal() to detect this case.
func (i *Intent) Stuck() bool {
	// The end of the state machine has been reached.
	exhausted := i.Terminal() && i.Realized()
	// A step failed during and may not be able to be tried without taking
	// intervening action.
	failure := i.Errored()
	// The actions reached a static position, but are unable to be driven by the
	// state machine's steps.
	degradedStatic := !i.Waiting() && (exhausted || failure)
	// The actions were transitioned to unknown handling and waiting for
	// instructions.
	stuckUnknown := i.inUnknownState() && !i.InProgress()
	wantingUnknown := i.Wanted == marker.NodeActionUnknown && i.Waiting()
	degradedUnknown := stuckUnknown || wantingUnknown
	// The action's step was out of line and resulted in an taking an unknown
	// action.
	degradedPath := i.DegradedPath()
	// The action was not one of progress and yet was acted upon.
	degradedBusy := !i.isInProgress(i.Wanted) && i.Wanted == i.Active && i.State == marker.NodeStateBusy

	if logging.Debuggable {
		logging.New("intent").WithFields(logrus.Fields{
			"intent":          i.DisplayString(),
			"degradedStatic":  degradedStatic,
			"degradedUnknown": degradedUnknown,
			"degradedPath":    degradedPath,
			"degradedBusy":    degradedBusy,
		}).Debug("Stuck")
	}

	return degradedStatic || degradedUnknown || degradedPath || degradedBusy
}

// DegradedPaths indicates that the intent will derail and step into an unknown
// step if it has not already.
func (i *Intent) DegradedPath() bool {
	anticipated := i.projectActive()
	// path is misaligned because we're starting anew.
	starting := i.SetBeginUpdate().Wanted == i.Wanted
	untargeted := anticipated.Wanted == marker.NodeActionUnknown
	inconsistent := !i.Realized() && anticipated.Wanted != i.Wanted

	if logging.Debuggable {
		logging.New("intent").WithFields(logrus.Fields{
			"intent":       i.DisplayString(),
			"anticipated":  anticipated.DisplayString(),
			"starting":     starting,
			"untargeted":   untargeted,
			"inconsistent": inconsistent,
		}).Debug("DegradedPath")
	}
	return (!starting || i.Terminal()) && (untargeted || inconsistent)
}

// Realized indicates that the Intent reached the intended state.
func (i *Intent) Realized() bool {
	return !i.InProgress() && !i.Errored()
}

// InProgess reports true when the Intent is for a node that is actively making
// progress towards completing an update.
func (i *Intent) InProgress() bool {
	// waiting for handling of intent
	pendingNode := i.Wanted != i.Active && i.Waiting() && !i.Errored()
	// waiting on handler to complete its intent handling
	pendingFinish := i.Wanted == i.Active && !i.Waiting()

	if logging.Debuggable {
		logging.New("intent").WithFields(logrus.Fields{
			"intent":        i.DisplayString(),
			"pendingNode":   pendingNode,
			"pendingFinish": pendingFinish,
		}).Debug("InProgress")
	}

	return pendingNode || pendingFinish
}

// isInProgress indicates that the field provided is an action that may be able
// to make progress towards another state.
func (i *Intent) isInProgress(field marker.NodeAction) bool {
	switch field {
	case marker.NodeActionReset,
		marker.NodeActionStabilize,
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

func (i *Intent) SetBeginUpdate() *Intent {
	u := i.Clone()
	u.Wanted = marker.NodeActionPrepareUpdate
	return u
}

// Needed indicates that the intent is needing progress made on it.
func (i *Intent) Actionable() bool {
	needsAction := (i.Waiting() || i.Realized()) && !i.Terminal()
	return needsAction && !i.Stuck() && !i.InProgress()
}

// Projected returns the n+1 step projection of a would-be Intent. It does not
// check whether or not the next step is correct given the current intent (ie:
// this will not error if the node has not actually completed a step).
func (i *Intent) Projected() *Intent {
	p := i.Clone()
	if p.inUnknownState() {
		p.reset()
	}
	p.Wanted, _ = calculateNext(p.Wanted)
	return p
}

func (i *Intent) projectActive() *Intent {
	prior := i.Clone()
	prior.Wanted = i.Active
	return prior.Projected()
}

func (i *Intent) inUnknownState() bool {
	return i.State == "" ||
		i.State == marker.NodeStateUnknown
}

// Terminal indicates that the intent has reached a terminal point in the
// progression, the intent will not make progress in anyway without outside
// state action.
func (i *Intent) Terminal() bool {
	next, err := calculateNext(i.Wanted)
	if err != nil {
		return false
	}
	// The next turn in the state machine is the same as the realized Wanted and
	// Active states, therefore we've reached a terminal point.
	atTerminal := next == i.Wanted && i.Wanted == i.Active
	if logging.Debuggable {
		logging.New("intent").WithFields(logrus.Fields{
			"atTerminal": atTerminal,
		}).Debug("Targeted")
	}
	return atTerminal
}

// Reset brings the intent back to the start of the progression where the intent
// may be able to resolve issues and fall into a valid state.
func (i *Intent) Reset() *Intent {
	p := i.Clone()
	p.reset()
	return p.Projected()
}

// reset reverts the Intent to its Origin point from which an Intent should be
// able to be driven to a Terminal point.
func (i *Intent) reset() {
	i.Wanted = marker.NodeActionUnknown
	i.Active = marker.NodeActionUnknown
	i.State = marker.NodeStateUnknown
	i.UpdateAvailable = marker.NodeUpdateUnknown
}

// SetUpdateAvailable modifies the intent to reflect the provided available
// state.
func (i *Intent) SetUpdateAvailable(available bool) *Intent {
	switch available {
	case true:
		i.UpdateAvailable = marker.NodeUpdateAvailable
	case false:
		i.UpdateAvailable = marker.NodeUpdateUnavailable
	}

	return i
}

func (i *Intent) DisplayString() string {
	if i == nil {
		return fmt.Sprintf(",,")
	}
	return fmt.Sprintf("%s,%s,%s", i.Wanted, i.Active, i.State)
}

// Clone returns a copy of the Intent to mutate independently of the source
// instance.
func (i Intent) Clone() *Intent {
	return Given(&i)
}

// Equivalent compares intentional state to determine equivalency.
func Equivalent(i, j *Intent) bool {
	return i.Wanted == j.Wanted &&
		i.Active == i.Active &&
		i.State == i.State
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
	// GetName returns the Input's addressable Name.
	GetName() string
}
