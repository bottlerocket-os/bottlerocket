package intent

import (
	"fmt"
	"reflect"
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/pkg/errors"
	"gotest.tools/assert"
)

func testIntent() *Intent {
	i := &Intent{
		NodeName: "test",
		Wanted:   marker.NodeActionStabilize,
		Active:   marker.NodeActionStabilize,
		State:    marker.NodeStateReady,
	}
	return i
}

func TestReset(t *testing.T) {
	i := testIntent()
	s := testIntent()

	s.reset()

	// first action after reset
	assert.Equal(t, s.Projected().Wanted, marker.NodeActionStabilize)
	assert.Check(t, i.Active != s.Active)
}

func TestGivenDuplicate(t *testing.T) {
	i := testIntent()
	s := Given(i)
	assert.DeepEqual(t, i, s)
}

func TestClone(t *testing.T) {
	i := testIntent()
	i.State = marker.NodeStateUnknown
	s := i.Clone()
	assert.DeepEqual(t, i, s)
}

func TestIntentTruths(t *testing.T) {
	type pred = string

	testcases := []struct {
		name    string
		intents []Intent
		truthy  []pred
		falsy   []pred
	}{
		{
			name: "empty",
			intents: []Intent{
				{}, // empty
			},
			truthy: []pred{"Stuck"},
			falsy:  []pred{"Errored"},
		},
		{
			name: "reset",
			intents: []Intent{
				func() Intent { i := testIntent(); i.reset(); return *i }(),
			},
			truthy: []pred{"Realized", "Waiting", "Stuck"},
			falsy:  []pred{"Intrusive"},
		},
		{
			name: "working",
			intents: []Intent{
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
			name: "not-stuck-pending",
			intents: []Intent{
				{
					Wanted: marker.NodeActionStabilize,
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateReady,
				},
				{
					// The first step we take in an update, should be coming
					// from a stable place.
					Wanted: ((&Intent{}).SetBeginUpdate().Wanted),
					Active: marker.NodeActionStabilize,
					State:  marker.NodeStateReady,
				},
			},
			truthy: []pred{"Waiting"},
			falsy:  []pred{"Stuck", "Errored", "DegradedPath"},
		},
		{
			name: "stuck",
			intents: []Intent{
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
			intents: []Intent{
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
			intents: []Intent{
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
			intents: []Intent{
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
			intents: []Intent{
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
			intents: []Intent{
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
			intents: []Intent{
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
			name: "terminal",
			intents: []Intent{
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
			intents: []Intent{

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

				//t.Logf("projectActive %#v", intent.projectActive())

				for _, predT := range tc.truthy {
					noOverlap(predT)
					match, err := callCheck(&intent, predT)
					assert.NilError(t, err)
					assert.Check(t, match, "%q expected to be true", predT)
				}

				for _, predF := range tc.falsy {
					noOverlap(predF)
					match, err := callCheck(&intent, predF)
					assert.NilError(t, err)
					assert.Check(t, !match, "%q expected to be false", predF)
				}
			})
		}
	}
}

func callCheck(recv *Intent, methodName string) (bool, error) {
	val := reflect.ValueOf(recv)
	typ := reflect.TypeOf(recv)
	method, ok := typ.MethodByName(methodName)
	if !ok {
		return false, errors.Errorf("no predicate method named %q", methodName)
	}
	res := method.Func.Call([]reflect.Value{val})
	if len(res) != 1 {
		return false, errors.Errorf("expected single return value from predicate method")
	}
	if res[0].Type().Name() != "bool" {
		return false, errors.Errorf("return value from predicate was not a bool")
	}
	return res[0].Bool(), nil
}

func TestProjectionMatches(t *testing.T) {
	i := Intent{
		Wanted: marker.NodeActionPerformUpdate,
		Active: marker.NodeActionStabilize,
		State:  marker.NodeStateReady,
	}
	assert.Equal(t, i.projectActive().Wanted, i.Active)
}
