package controller

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/internal/intents"
	"github.com/amazonlinux/thar/dogswatch/pkg/internal/testoutput"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"gotest.tools/assert"
)

type testingPoster struct {
	calledIntents []intent.Intent
	fn            func(i *intent.Intent) error
}

func (p *testingPoster) Post(i *intent.Intent) error {
	p.calledIntents = append(p.calledIntents, *i)
	if p.fn != nil {
		return p.fn(i)
	}
	return nil
}

type testingNodeManager struct {
	CordonFn   func(string) error
	UncordonFn func(string) error
	DrainFn    func(string) error
}

func trackFn(v *bool) func(string) error {
	return func(_ string) error {
		*v = true
		return nil
	}
}

func (nm *testingNodeManager) Cordon(n string) error {
	if nm.CordonFn != nil {
		return nm.CordonFn(n)
	}
	return nil
}

func (nm *testingNodeManager) Uncordon(n string) error {
	if nm.UncordonFn != nil {
		return nm.UncordonFn(n)
	}
	return nil
}

func (nm *testingNodeManager) Drain(n string) error {
	if nm.DrainFn != nil {
		return nm.DrainFn(n)
	}
	return nil
}

type testManagerHooks struct {
	Poster      *testingPoster
	NodeManager *testingNodeManager
}

func testManager(t *testing.T) (*ActionManager, *testManagerHooks) {
	m := newManager(testoutput.Logger(t, logging.New("manager")), nil, "test-node")

	hooks := &testManagerHooks{
		Poster:      &testingPoster{},
		NodeManager: &testingNodeManager{},
	}
	m.poster = hooks.Poster
	m.nodem = hooks.NodeManager
	return m, hooks
}

func TestManagerIntentForSimple(t *testing.T) {
	nils := []*intent.Intent{
		intents.BusyRebootUpdate(),
		intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateUnavailable)),
		intents.PendingUpdate(),
	}
	nonnils := []*intent.Intent{
		intents.UpdateError(),
		intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateAvailable)),
	}

	intents.NormalizeNodeName("inactive", nils...)
	intents.NormalizeNodeName("active", nonnils...)

	for _, in := range nils {
		t.Run(fmt.Sprintf("nil(%s)", in.DisplayString()), func(t *testing.T) {
			m, _ := testManager(t)
			actual := m.intentFor(in)
			assert.Assert(t, actual == nil)
		})
	}
	for _, in := range nonnils {
		t.Run(fmt.Sprintf("non(%s)", in.DisplayString()), func(t *testing.T) {
			m, _ := testManager(t)
			actual := m.intentFor(in)
			assert.Assert(t, actual != nil)
		})
	}
}

func TestManagerIntentForTargeted(t *testing.T) {
	cases := []struct {
		input    *intent.Intent
		expected *intent.Intent
	}{
		{
			input:    intents.UpdateError(),
			expected: intents.Reset(),
		},
		// Update handling is a pass through to handle the "exact" intent.
		{
			input:    intents.UpdateSuccess(),
			expected: intents.UpdateSuccess(),
		},
		{
			input: intents.UpdatePrepared(),
			expected: intents.UpdatePrepared(
				intents.Pending(marker.NodeActionPerformUpdate)),
		},
		{
			input:    intents.PendingStabilizing(),
			expected: nil,
		},
		{
			input:    intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateUnavailable)),
			expected: nil,
		},
	}

	for _, tc := range cases {
		t.Run(fmt.Sprintf("%s--->%s", tc.input.DisplayString(), tc.expected.DisplayString()), func(t *testing.T) {
			intents.NormalizeNodeName(t.Name(), tc.input, tc.expected)
			m, _ := testManager(t)
			actual := m.intentFor(tc.input)
			assert.DeepEqual(t, actual, tc.expected)
		})
	}
}

func TestTakeAction(t *testing.T) {
	t.Run("success", func(t *testing.T) {
		m, hooks := testManager(t)
		var (
			uncordoned = false
		)
		hooks.NodeManager.UncordonFn = trackFn(&uncordoned)
		err := m.takeAction(intents.UpdateSuccess())
		assert.NilError(t, err)
		assert.Check(t, uncordoned)
	})

	t.Run("perform-update", func(t *testing.T) {
		m, hooks := testManager(t)
		var (
			uncordoned = false
			cordoned   = false
			drained    = false
		)
		hooks.NodeManager.DrainFn = trackFn(&drained)
		hooks.NodeManager.CordonFn = trackFn(&cordoned)
		hooks.NodeManager.UncordonFn = trackFn(&uncordoned)
		in := intents.UpdatePerformed()
		t.Logf("controller input intent: %s", in)
		pin := m.intentFor(in)
		t.Logf("controller managed intent: %s", pin.DisplayString())
		err := m.takeAction(pin)
		assert.NilError(t, err)
		assert.Check(t, cordoned == true)
		assert.Check(t, drained == true)
		assert.Check(t, uncordoned != true)
	})

	t.Run("signal-stabilize", func(t *testing.T) {
		m, hooks := testManager(t)
		var (
			uncordoned = false
			cordoned   = false
			drained    = false
		)
		hooks.NodeManager.DrainFn = trackFn(&drained)
		hooks.NodeManager.CordonFn = trackFn(&cordoned)
		hooks.NodeManager.UncordonFn = trackFn(&uncordoned)
		in := intents.Unknown()
		t.Logf("controller input intent: %s", in)
		pin := m.intentFor(in)
		t.Logf("controller managed intent: %s", pin.DisplayString())
		err := m.takeAction(pin)
		assert.NilError(t, err)
		assert.Check(t, cordoned == false)
		assert.Check(t, drained == false)
		assert.Check(t, uncordoned == false)
	})
}

func TestMakePolicyCheck(t *testing.T) {
	m, _ := testManager(t)

	pview, err := m.makePolicyCheck(intents.Stabilized())
	assert.Check(t, pview == nil)
	assert.Check(t, err != nil)
}

func TestMakePolicyCheckUpdatesAvailable(t *testing.T) {
	m, _ := testManager(t)
	pview, err := m.makePolicyCheck(intents.Stabilized(intents.WithUpdateAvailable()))
	assert.Check(t, pview == nil)
	assert.Check(t, err != nil)
}

func TestSuccessfulUpdate(t *testing.T) {
	logging.Set(testoutput.Setter(t))
	defer logging.Set(testoutput.Revert())

	cases := struct {
		truthy []*intent.Intent
		falsy  []*intent.Intent
	}{
		truthy: []*intent.Intent{
			intents.UpdateSuccess(),
		},
		falsy: []*intent.Intent{
			intents.Unknown(),
			intents.Stabilized(),
			intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateUnavailable)),
		},
	}

	for _, tc := range cases.truthy {
		t.Run(tc.DisplayString(), func(t *testing.T) {
			logging.Set(testoutput.Setter(t))
			defer logging.Set(testoutput.Revert())
			assert.Check(t, successfulUpdate(tc) == true)
		})
	}
	for _, tc := range cases.falsy {
		t.Run(tc.DisplayString(), func(t *testing.T) {
			logging.Set(testoutput.Setter(t))
			defer logging.Set(testoutput.Revert())
			assert.Check(t, successfulUpdate(tc) == false)
		})
	}
}
