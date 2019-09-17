package intent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
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

func calculateNext(action marker.NodeAction) marker.NodeAction {
	// TODO: resolve next state if applicable
	return nextLinear[action]
}
