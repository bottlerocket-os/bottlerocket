package intent

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/intent/internal/callcheck"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/internal/testoutput"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/marker"

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

func TestIntentTruthsInternal(t *testing.T) {
	type pred = string
	testcases := []struct {
		name    string
		intents []Intent
		truthy  []pred
		falsy   []pred
	}{
		{
			name: "reset",
			intents: []Intent{
				func(i *Intent) Intent { i.reset(); return *i }(testIntent()),
			},
			truthy: []pred{"Realized", "Waiting", "Stuck"},
			falsy:  []pred{"Intrusive"},
		},
	}

	for _, tc := range testcases {
		for _, intent := range tc.intents {
			name := fmt.Sprintf("%s(%s)", tc.name, intent.DisplayString())
			t.Run(name, func(t *testing.T) {
				logging.Set(testoutput.Setter(t))
				defer logging.Set(testoutput.Revert())

				intent.NodeName = "state-machine"

				preds := map[pred]struct{}{}
				noOverlap := func(p pred) {
					_, overlappingPredicate := preds[p]
					assert.Assert(t, !overlappingPredicate, "the predicate %q was asserted twice", p)
					preds[p] = struct{}{}
				}

				for _, predT := range tc.truthy {
					noOverlap(predT)
					match, err := callcheck.Predicate(&intent, predT)
					assert.NilError(t, err)
					assert.Check(t, match, "%q expected to be true", predT)
				}

				for _, predF := range tc.falsy {
					noOverlap(predF)
					match, err := callcheck.Predicate(&intent, predF)
					assert.NilError(t, err)
					assert.Check(t, !match, "%q expected to be false", predF)
				}
			})
		}
	}
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

func TestProjectionMatches(t *testing.T) {
	i := Intent{
		Wanted: marker.NodeActionPerformUpdate,
		Active: marker.NodeActionStabilize,
		State:  marker.NodeStateReady,
	}
	assert.Equal(t, i.projectActive().Wanted, i.Active)
}
