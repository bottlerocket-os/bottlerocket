package controller

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
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

func testManager(t *testing.T) *ActionManager {
	l := logging.New("manager").WithFields(logrus.Fields{})
	l.Logger.SetOutput(&testingOutput{t})
	l.Logger.SetLevel(logrus.DebugLevel)
	m := newManager(l, nil, "test-node")
	return m
}

func TestManagerIntentForSimple(t *testing.T) {
	nils := []intent.Intent{
		{
			Wanted:          marker.NodeActionRebootUpdate,
			Active:          marker.NodeActionRebootUpdate,
			State:           marker.NodeStateBusy,
			UpdateAvailable: marker.NodeUpdateAvailable,
		},
	}
	nonnils := []intent.Intent{
		{
			Wanted:          marker.NodeActionRebootUpdate,
			Active:          marker.NodeActionRebootUpdate,
			State:           marker.NodeStateError,
			UpdateAvailable: marker.NodeUpdateAvailable,
		},
	}
	for _, in := range nils {
		in.NodeName = "test-node"
		t.Run(fmt.Sprintf("nil(%s)", in.DisplayString()), func(t *testing.T) {
			m := testManager(t)
			actual := m.intentFor(&in)
			assert.Assert(t, actual == nil)
		})
	}
	for _, in := range nonnils {
		in.NodeName = "test-node"
		t.Run(fmt.Sprintf("non(%s)", in.DisplayString()), func(t *testing.T) {
			m := testManager(t)
			actual := m.intentFor(&in)
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
			input: &intent.Intent{
				Wanted:          marker.NodeActionRebootUpdate,
				Active:          marker.NodeActionRebootUpdate,
				State:           marker.NodeStateError,
				UpdateAvailable: marker.NodeUpdateAvailable,
			},
			expected: (&intent.Intent{}).Reset(),
		},
		{
			input: &intent.Intent{
				Wanted:          marker.NodeActionStabilize,
				Active:          marker.NodeActionUnknown,
				State:           marker.NodeStateUnknown,
				UpdateAvailable: marker.NodeUpdateAvailable,
			},
			expected: nil,
		},
	}

	for _, tc := range cases {
		tc.input.NodeName = "test-node"
		if tc.expected != nil {
			tc.expected.NodeName = tc.input.NodeName
		}
		t.Run(fmt.Sprintf("%s--->%s", tc.input.DisplayString(), tc.expected.DisplayString()), func(t *testing.T) {
			m := testManager(t)
			actual := m.intentFor(tc.input)
			assert.DeepEqual(t, actual, tc.expected)
		})
	}
}
