package agent

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"gotest.tools/assert"
)

func TestActiveIntent(t *testing.T) {
	active := []intent.Intent{
		{
			Wanted: marker.NodeActionStabilize,
			Active: marker.NodeActionUnknown,
			State:  marker.NodeStateUnknown,
		},
	}

	inactive := []intent.Intent{
		{
			Wanted: marker.NodeActionRebootUpdate,
			Active: marker.NodeActionRebootUpdate,
			State:  marker.NodeStateError,
		},
		{
			Wanted: marker.NodeActionStabilize,
			Active: "",
			State:  "arst",
		},
		{
			Wanted: marker.NodeActionPerformUpdate,
			Active: marker.NodeActionPerformUpdate,
			State:  marker.NodeStateReady,
		},
		{
			Wanted: marker.NodeActionPerformUpdate,
			Active: marker.NodeActionPerformUpdate,
			State:  marker.NodeStateError,
		},
		{
			Wanted: marker.NodeActionPerformUpdate,
			Active: marker.NodeActionPerformUpdate,
			State:  marker.NodeStateUnknown,
		},
		{
			Wanted: "",
			Active: marker.NodeActionPerformUpdate,
			State:  marker.NodeStateUnknown,
		},
	}

	for _, in := range active {
		t.Run(fmt.Sprintf("active(%s)", in.DisplayString()), func(t *testing.T) {
			assert.Check(t, activeIntent(&in) == true)
		})
	}

	for _, in := range inactive {
		t.Run(fmt.Sprintf("inactive(%s)", in.DisplayString()), func(t *testing.T) {
			assert.Check(t, activeIntent(&in) == false)
		})
	}
}
