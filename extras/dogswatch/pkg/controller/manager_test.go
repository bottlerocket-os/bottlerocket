package controller

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/internal/intents"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/sirupsen/logrus"
	"gotest.tools/assert"
)

type testingOutput struct {
	t *testing.T
}

func (l *testingOutput) Write(p []byte) (n int, err error) {
	l.t.Logf("%s", p)
	return len(p), nil
}

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

func testManager(t *testing.T) *ActionManager {
	l := logging.New("manager").WithFields(logrus.Fields{})
	l.Logger.SetOutput(&testingOutput{t})
	l.Logger.SetLevel(logrus.DebugLevel)
	m := newManager(l, nil, "test-node")
	m.poster = &testingPoster{}
	return m
}

func TestManagerIntentForSimple(t *testing.T) {
	nils := []*intent.Intent{
		intents.BusyRebootUpdate(),
	}
	nonnils := []*intent.Intent{
		intents.UpdateError(),
	}

	for _, in := range nils {
		in.NodeName = "test-node"
		t.Run(fmt.Sprintf("nil(%s)", in.DisplayString()), func(t *testing.T) {
			m := testManager(t)
			actual := m.intentFor(in)
			assert.Assert(t, actual == nil)
		})
	}
	for _, in := range nonnils {
		in.NodeName = "test-node"
		t.Run(fmt.Sprintf("non(%s)", in.DisplayString()), func(t *testing.T) {
			m := testManager(t)
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
			input:    intents.PendingStabilizing(),
			expected: nil,
		},
	}

	for _, tc := range cases {
		t.Run(fmt.Sprintf("%s--->%s", tc.input.DisplayString(), tc.expected.DisplayString()), func(t *testing.T) {
			intents.NormalizeNodeName(t.Name(), tc.input, tc.expected)
			m := testManager(t)
			actual := m.intentFor(tc.input)
			assert.DeepEqual(t, actual, tc.expected)
		})
	}
}

func TestMakePolicyCheck(t *testing.T) {
	m := testManager(t)

	pview, err := m.makePolicyCheck(intents.Stabilized())
	assert.Check(t, pview == nil)
	assert.Check(t, err != nil)
}

func TestMakePolicyCheckUpdatesAvailable(t *testing.T) {
	m := testManager(t)
	pview, err := m.makePolicyCheck(intents.Stabilized(intents.WithUpdateAvailable()))
	assert.Check(t, pview == nil)
	assert.Check(t, err != nil)
}
