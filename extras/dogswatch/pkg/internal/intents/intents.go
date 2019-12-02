package intents

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
)

const (
	// NodeName is a suitable reusable NodeName that may be provided by callers.
	NodeName = "intents-node"
)

// ret is a wrapper that provides the extendable interface to canned and
// distinguished Intents for use in tests.
func ret(name string, i intent.Intent, initOpts ...func(i *intent.Intent)) func(opts ...func(i *intent.Intent)) *intent.Intent {
	i.NodeName = name
	for _, opt := range initOpts {
		opt(&i)
	}

	return func(opts ...func(i *intent.Intent)) *intent.Intent {
		c := i.Clone()
		for _, opt := range opts {
			opt(c)
		}
		return c
	}
}

// WithReset resets the provided intent to the initial state.
func WithReset() func(i *intent.Intent) {
	return func(i *intent.Intent) {
		*i = *i.Reset() // replace the pointer's value
	}
}

// WithNodeName sets the intent's NodeName.
func WithNodeName(name string) func(i *intent.Intent) {
	return func(i *intent.Intent) {
		i.NodeName = name
	}
}

// WithBusy marks the intent as busy with the intent.
func WithBusy() func(i *intent.Intent) {
	return func(i *intent.Intent) {
		i.State = marker.NodeStateBusy
	}
}

// WithUpdateAvailable marks the intent with the provided NodeUpdate marker.
func WithUpdateAvailable(up ...marker.NodeUpdate) func(i *intent.Intent) {
	return func(i *intent.Intent) {
		if len(up) == 1 {
			i.UpdateAvailable = up[0]
			return
		}
		i.UpdateAvailable = marker.NodeUpdateAvailable
	}
}

// NormalizeNodeName makes all provided intents' NodeName uniform for comparison
// or otherwise normalized expectations.
func NormalizeNodeName(name string, is ...*intent.Intent) {
	namer := WithNodeName(name)
	for _, in := range is {
		if in != nil {
			namer(in)
		}
	}
}

// Set the intent to be pending the provided NodeAction.
func Pending(wanted marker.NodeAction) func(*intent.Intent) {
	return func(i *intent.Intent) {
		i.Wanted = wanted
	}
}

// Use the provided intent as the targeted next NodeAction.
func NextAs(next *intent.Intent) func(*intent.Intent) {
	return func(in *intent.Intent) {
		if next == nil {
			return
		}
		in.Wanted = next.Wanted
	}
}

var (
	Stabilized = ret("Stabilized", intent.Intent{
		Wanted: marker.NodeActionStabilize,
		Active: marker.NodeActionStabilize,
		State:  marker.NodeStateReady,
	})

	Stabilizing = ret("Stabilizing", intent.Intent{
		Wanted: marker.NodeActionStabilize,
		Active: marker.NodeActionStabilize,
		State:  marker.NodeStateBusy,
	})

	PendingStabilizing = ret("PendingStabilizing", intent.Intent{
		Wanted: marker.NodeActionStabilize,
		Active: marker.NodeActionUnknown,
		State:  marker.NodeStateUnknown,
	})

	BusyRebootUpdate = ret("PendingRebootUpdate", intent.Intent{
		Wanted: marker.NodeActionRebootUpdate,
		Active: marker.NodeActionRebootUpdate,
		State:  marker.NodeStateBusy,
	}, WithUpdateAvailable())

	PendingRebootUpdate = ret("PendingRebootUpdate", intent.Intent{
		Wanted: marker.NodeActionRebootUpdate,
		Active: marker.NodeActionPrepareUpdate,
		State:  marker.NodeStateReady,
	})

	UpdateError = ret("UpdateError", intent.Intent{
		Wanted: marker.NodeActionRebootUpdate,
		Active: marker.NodeActionRebootUpdate,
		State:  marker.NodeStateError,
	}, WithUpdateAvailable())

	UpdateSuccess = ret("UpdateSuccess", intent.Intent{
		Wanted: marker.NodeActionRebootUpdate,
		Active: marker.NodeActionRebootUpdate,
		State:  marker.NodeStateReady,
	}, WithUpdateAvailable(marker.NodeUpdateUnknown))

	UpdatePrepared = ret("UpdatePrepared", intent.Intent{
		Wanted: marker.NodeActionPrepareUpdate,
		Active: marker.NodeActionPrepareUpdate,
		State:  marker.NodeStateReady,
	}, WithUpdateAvailable(marker.NodeUpdateAvailable))

	PendingPrepareUpdate = ret("PendingPrepareUpdate", intent.Intent{
		Wanted: marker.NodeActionPrepareUpdate,
		Active: marker.NodeActionStabilize,
		State:  marker.NodeStateReady,
	})

	UpdatePerformed = ret("UpdatePerformed", intent.Intent{
		Wanted: marker.NodeActionPerformUpdate,
		Active: marker.NodeActionPerformUpdate,
		State:  marker.NodeStateReady,
	}, WithUpdateAvailable(marker.NodeUpdateAvailable))

	PendingUpdate = ret("PendingUpdate", intent.Intent{
		Wanted: marker.NodeActionPerformUpdate,
		Active: marker.NodeActionPrepareUpdate,
		State:  marker.NodeStateReady,
	}, WithUpdateAvailable(marker.NodeUpdateAvailable))

	Unknown = ret("Unknown", intent.Intent{
		Wanted: marker.NodeActionUnknown,
		Active: marker.NodeActionUnknown,
		State:  marker.NodeStateUnknown,
	}, WithUpdateAvailable())

	Reset = ret("Reset", intent.Intent{}, WithReset())
)
