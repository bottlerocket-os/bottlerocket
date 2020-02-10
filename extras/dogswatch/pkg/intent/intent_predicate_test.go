package intent_test

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/intent"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/intent/internal/callcheck"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/internal/intents"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/internal/testoutput"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/marker"

	"gotest.tools/assert"
)

func TestIntentTruths(t *testing.T) {
	type pred = string

	testcases := []struct {
		name    string
		intents []intent.Intent
		truthy  []pred
		falsy   []pred
	}{
		{
			name: "empty",
			intents: []intent.Intent{
				{}, // empty
			},
			truthy: []pred{"Stuck"},
			falsy:  []pred{"Errored"},
		},
		{
			name: "success",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionRebootUpdate,
					Active: marker.NodeActionRebootUpdate,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Waiting", "Terminal", "Realized"},
			falsy:  []pred{"Intrusive", "Stuck", "InProgress"},
		},
		{
			name: "working",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionStabilize,
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateBusy,
				},
			},
			truthy: []pred{"InProgress"},
			falsy:  []pred{"Waiting", "Actionable", "Realized", "Stuck"},
		},
		{
			name: "not-stuck",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionStabilize,
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateReady,
				},
				{
					// The first step we take in an update, should be coming
					// from a stable place.
					Wanted: ((&intent.Intent{}).SetBeginUpdate().Wanted),
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Waiting"},
			falsy:  []pred{"Stuck", "Errored", "DegradedPath"},
		},
		{
			name: "not-stuck-busy",
			intents: []intent.Intent{
				*intents.PreparingUpdate(),
				*intents.PerformingUpdate(),
				*intents.BusyRebootUpdate(),
			},
			truthy: []pred{"InProgress"},
			falsy:  []pred{"Waiting", "Errored", "Stuck"},
		},
		{
			name: "stuck",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionUnknown,
					Active: marker.NodeActionUnknown,
					State:  marker.NodeStateBusy,
				},
				{
					Wanted: marker.NodeActionUnknown,
					Active: marker.NodeActionUnknown,
					State:  marker.NodeStateError,
				},
				{
					Wanted: marker.NodeActionUnknown,
					Active: marker.NodeActionPerformUpdate,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Stuck"},
			falsy:  []pred{"Realized", "Terminal"},
		},
		{
			name: "stuck",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionRebootUpdate,
					Active: marker.NodeActionUnknown,
					State:  marker.NodeStateError,
				},
			},
			truthy: []pred{"DegradedPath"},
		},
		{
			name: "waiting",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionStabilize,
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Waiting", "Realized", "Terminal"},
			falsy:  []pred{"Actionable"},
		},
		{
			name: "waiting",
			intents: []intent.Intent{
				{
					Wanted:          marker.NodeActionStabilize,
					Active:          marker.NodeActionUnknown,
					State:           marker.NodeStateUnknown,
					UpdateAvailable: marker.NodeUpdateAvailable,
				},
			},
			truthy: []pred{"InProgress"},
			falsy:  []pred{"Realized", "Actionable", "Stuck"},
		},
		{
			name: "errored-nominal",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionStabilize,
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateError,
				},
			},
			truthy: []pred{"Errored", "Waiting"},
			falsy:  []pred{"Realized"},
		},
		{
			name: "errored-unusual",
			intents: []intent.Intent{
				{
					Wanted: "arst",
					Active: "neio",
					State:  marker.NodeStateError,
				},
			},
			truthy: []pred{"Errored", "Waiting", "Stuck"},
			falsy:  []pred{"Realized"},
		},
		{
			name: "inprogress",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionRebootUpdate,
					Active: marker.NodeActionRebootUpdate,
					State:  marker.NodeStateBusy,
				},
			},
			truthy: []pred{"InProgress", "Intrusive"},
			falsy:  []pred{"Errored", "Realized", "Stuck", "Waiting"},
		},
		{
			name: "actionable",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionPrepareUpdate,
					Active: marker.NodeActionPrepareUpdate,
					State:  marker.NodeStateReady,
				},
				{
					Wanted: marker.NodeActionPerformUpdate,
					Active: marker.NodeActionPerformUpdate,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Actionable", "Realized", "Waiting"},
			falsy:  []pred{"Errored", "Stuck", "DegradedPath"},
		},
		{
			name: "terminal",
			intents: []intent.Intent{
				{
					Wanted: marker.NodeActionRebootUpdate,
					Active: marker.NodeActionRebootUpdate,
					State:  marker.NodeStateBusy,
				},
			},
			truthy: []pred{"Terminal", "InProgress"},
			falsy:  []pred{"Errored", "Realized", "Stuck", "Actionable", "Waiting"},
		},
		{
			name: "terminal",
			intents: []intent.Intent{

				{
					Wanted: marker.NodeActionRebootUpdate,
					Active: marker.NodeActionRebootUpdate,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Terminal", "Realized", "Waiting"},
			falsy:  []pred{"Errored", "Stuck", "Actionable"},
		},
	}

	for _, tc := range testcases {
		for _, intent := range tc.intents {
			name := fmt.Sprintf("%s(%s)", tc.name, intent.DisplayString())
			t.Run(name, func(t *testing.T) {
				intent.NodeName = "state-machine"

				preds := map[pred]struct{}{}
				noOverlap := func(p pred) {
					_, overlappingPredicate := preds[p]
					assert.Assert(t, !overlappingPredicate, "the predicate %q was asserted twice", p)
					preds[p] = struct{}{}
				}

				for _, predT := range tc.truthy {
					t.Run(predT, func(t *testing.T) {
						logging.Set(testoutput.Setter(t))
						defer logging.Set(testoutput.Revert())

						noOverlap(predT)
						match, err := callcheck.Predicate(&intent, predT)
						assert.NilError(t, err)
						assert.Check(t, match, "%q expected to be true", predT)
					})
				}

				for _, predF := range tc.falsy {
					t.Run(predF, func(t *testing.T) {
						logging.Set(testoutput.Setter(t))
						defer logging.Set(testoutput.Revert())

						noOverlap(predF)
						match, err := callcheck.Predicate(&intent, predF)
						assert.NilError(t, err)
						assert.Check(t, !match, "%q expected to be false", predF)
					})
				}
			})
		}
	}
}
