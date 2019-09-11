package intent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
)

var nextLinear map[marker.NodeAction]marker.NodeAction = map[marker.NodeAction]marker.NodeAction{
	"":                             marker.NodeActionStablize,
	marker.NodeActionUnknown:       marker.NodeActionStablize,
	marker.NodeActionStablize:      "",
	marker.NodeActionReset:         marker.NodeActionStablize,
	marker.NodeActionPrepareUpdate: marker.NodeActionPerformUpdate,
	marker.NodeActionPerformUpdate: marker.NodeActionRebootUpdate,
	marker.NodeActionRebootUpdate:  "TODO-preflight-check",
}

func calculateNext(action marker.NodeAction, _state marker.NodeState) (marker.NodeAction, marker.NodeState) {
	// TODO: resolve next state if applicable
	return nextLinear[action], ""
}
