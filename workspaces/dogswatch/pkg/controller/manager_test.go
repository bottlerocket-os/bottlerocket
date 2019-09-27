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
	m := newManager(l, nil, "test-node")
	return m
}

func TestManagerIntentFor(t *testing.T) {
	nils := []intent.Intent{
		{
			Wanted:          marker.NodeActionRebootUpdate,
			Active:          marker.NodeActionRebootUpdate,
			State:           marker.NodeStateBusy,
			UpdateAvailable: marker.NodeUpdateAvailable,
		},
		{
			Wanted: marker.NodeActionRebootUpdate,
			Active: marker.NodeActionPerformUpdate,
			State: marker.NodeStateError,
			UpdateAvailable: marker.NodeUpdateAvailable,
		}
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
