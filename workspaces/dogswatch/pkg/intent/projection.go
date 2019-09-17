package intent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/pkg/errors"
)

var nextLinear map[marker.NodeAction]marker.NodeAction = map[marker.NodeAction]marker.NodeAction{
	"":                        marker.NodeActionStablize,
	marker.NodeActionStablize: marker.NodeActionStablize,
	marker.NodeActionUnknown:  marker.NodeActionStablize,

	marker.NodeActionReset:         marker.NodeActionStablize,
	marker.NodeActionPrepareUpdate: marker.NodeActionPerformUpdate,
	marker.NodeActionPerformUpdate: marker.NodeActionRebootUpdate,
	// FIN. The intendee must know what to do next.
	marker.NodeActionRebootUpdate: marker.NodeActionRebootUpdate,
}

// TODO: add tests for the expected state machine turns.

func calculateNext(action marker.NodeAction) (marker.NodeAction, error) {
	// TODO: resolve next state if applicable
	next, ok := nextLinear[action]
	if !ok {
		return marker.NodeActionUnknown, errors.Errorf("no next action from %q, resolving as unknown", action)
	}
	return next, nil
}
