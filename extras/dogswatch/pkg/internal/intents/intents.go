package intents

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
)

const (
	NodeName = "intents-library"
)

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

func WithReset() func(i *intent.Intent) {
	return func(i *intent.Intent) {
		*i = *i.Reset() // replace the pointer's value
	}
}

func WithNodeName(name string) func(i *intent.Intent) {
	return func(i *intent.Intent) {
		i.NodeName = name
	}
}

func WithBusy() func(i *intent.Intent) {
	return func(i *intent.Intent) {
		i.State = marker.NodeStateBusy
	}
}

func WithUpdateAvailable(up ...marker.NodeUpdate) func(i *intent.Intent) {
	return func(i *intent.Intent) {
		if len(up) == 1 {
			i.UpdateAvailable = up[0]
			return
		}
		i.UpdateAvailable = marker.NodeUpdateAvailable
	}
}

func NormalizeNodeName(name string, is ...*intent.Intent) {
	namer := WithNodeName(name)
	for _, in := range is {
		if in != nil {
			namer(in)
		}
	}
}

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

	Unknown = ret("Unknown", intent.Intent{
		Wanted: marker.NodeActionUnknown,
		Active: marker.NodeActionUnknown,
		State:  marker.NodeStateUnknown,
	}, WithUpdateAvailable())

	Reset = ret("Reset", intent.Intent{}, WithReset())
)
