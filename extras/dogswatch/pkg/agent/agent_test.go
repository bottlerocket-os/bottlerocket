package agent

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/intent"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/internal/intents"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/internal/testoutput"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/marker"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/platform"
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

		*intents.Stabilized(intents.WithUpdateAvailable("")),
		*intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateUnavailable)),
		*intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateUnknown)),
	}

	for _, in := range active {
		t.Run(fmt.Sprintf("active(%s)", in.DisplayString()), func(t *testing.T) {
			logging.Set(testoutput.Setter(t))
			defer logging.Set(testoutput.Revert())
			assert.Check(t, activeIntent(&in) == true)
		})
	}

	for _, in := range inactive {
		t.Run(fmt.Sprintf("inactive(%s)", in.DisplayString()), func(t *testing.T) {
			logging.Set(testoutput.Setter(t))
			defer logging.Set(testoutput.Revert())
			assert.Check(t, activeIntent(&in) == false)
		})
	}
}

func testAgent(t *testing.T) (*Agent, *testHooks) {
	hooks := &testHooks{
		Poster:   &testPoster{},
		Platform: &testPlatform{},
		Proc:     &testProc{},
	}
	a, err := New(testoutput.Logger(t, logging.New("agent")), nil, hooks.Platform, intents.NodeName)
	if err != nil {
		panic(err)
	}
	a.poster = hooks.Poster
	a.proc = hooks.Proc
	return a, hooks
}

type testHooks struct {
	Poster   *testPoster
	Proc     *testProc
	Platform *testPlatform
}

type testPoster struct {
	calledIntents []intent.Intent
	fn            func(i *intent.Intent) error
}

func (p *testPoster) Post(i *intent.Intent) error {
	p.calledIntents = append(p.calledIntents, *i)
	if p.fn != nil {
		return p.fn(i)
	}
	return nil
}

type testProc struct {
	Killed bool
}

func (p *testProc) KillProcess() error {
	p.Killed = true
	return nil
}

type testPlatform struct {
	StatusFn        func() (platform.Status, error)
	ListAvailableFn func() (platform.Available, error)
	PrepareFn       func(target platform.Update) error
	UpdateFn        func(target platform.Update) error
	BootUpdateFn    func(target platform.Update, rebootNow bool) error
}

// Status reports the underlying platform's health and metadata.
func (p *testPlatform) Status() (platform.Status, error) {
	if p.StatusFn != nil {
		return p.StatusFn()
	}
	status := testStatus(true)
	return &status, nil
}

type testStatus bool

func (s *testStatus) OK() bool {
	return *s == true
}

// ListAvailable provides the list of updates that a platform is offering
// for use. The list MUST be ordered in preference as well as recency.
func (p *testPlatform) ListAvailable() (platform.Available, error) {
	if p.ListAvailableFn != nil {
		return p.ListAvailable()
	}

	return &testListAvailable{}, nil
}

type testListAvailable struct {
	updates []platform.Update
}

func (l *testListAvailable) Updates() []platform.Update {
	update := testUpdate("test")
	return []platform.Update{
		&update,
	}
}

type testUpdate string

func (s *testUpdate) Identifier() interface{} {
	return *s
}

// Prepare causes the platform to take steps towards an update without
// committing to it. For example, a platform may require steps to preform
// pre-flight checkss or initialization migrations prior to executing an
// update.
func (p *testPlatform) Prepare(target platform.Update) error {
	if p.PrepareFn != nil {
		return p.PrepareFn(target)
	}
	return nil
}

// Update causes the platform to commit to an update taking potentially
// irreversible steps to do so.
func (p *testPlatform) Update(target platform.Update) error {
	if p.UpdateFn != nil {
		return p.UpdateFn(target)
	}
	return nil
}

// BootUpdate causes the platform to configure itself to use the update on
// next boot. Optionally, the caller may indicate that the update should be
// immediately rebooted to use.
func (p *testPlatform) BootUpdate(target platform.Update, rebootNow bool) error {
	if p.BootUpdateFn != nil {
		return p.BootUpdateFn(target, rebootNow)
	}
	return nil
}

func TestAgentRealize(t *testing.T) {
	t.Run("stabilize", func(t *testing.T) {
		a, hooks := testAgent(t)

		var (
			platformStatus = false
		)

		hooks.Platform.StatusFn = func() (platform.Status, error) {
			platformStatus = true
			status := testStatus(true)
			return &status, nil
		}
		a.realize(intents.PendingStabilizing())
		assert.Check(t, platformStatus == true)
	})

	t.Run("in-order", func(t *testing.T) {
		a, hooks := testAgent(t)

		var (
			platformPrepare = false
			platformUpdate  = false
			platformBoot    = false
		)
		hooks.Platform.PrepareFn = func(_ platform.Update) error {
			platformPrepare = true
			return nil
		}
		hooks.Platform.UpdateFn = func(_ platform.Update) error {
			platformUpdate = true
			return nil
		}
		hooks.Platform.BootUpdateFn = func(_ platform.Update, reboot bool) error {
			platformBoot = true
			return nil
		}

		// Call with prepare-update to kick off.
		{
			err := a.realize(intents.PendingPrepareUpdate())
			assert.Check(t, err == nil)
			assert.Check(t, platformPrepare == true)
		}
		// Then perform-update to apply the update
		{
			err := a.realize(intents.PendingUpdate())
			assert.Check(t, err == nil)
			assert.Check(t, platformUpdate == true)
		}
		// Then reboot-update to boot into the update
		{
			err := a.realize(intents.PendingRebootUpdate())
			assert.Check(t, err == nil)
			assert.Check(t, platformBoot == true)
		}
		// The process should have died to do this all.
		assert.Check(t, hooks.Proc.Killed == true)
	})

	t.Run("out-of-order", func(t *testing.T) {
		a, hooks := testAgent(t)

		var (
			platformUpdate = false
		)
		hooks.Platform.UpdateFn = func(_ platform.Update) error {
			platformUpdate = true
			return nil
		}
		err := a.realize(intents.PendingUpdate())
		assert.Check(t, err != nil)
		assert.Check(t, platformUpdate == false)
	})
}
