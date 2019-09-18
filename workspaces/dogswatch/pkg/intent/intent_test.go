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
		Wanted:   marker.NodeActionStablize,
		Active:   marker.NodeActionStablize,
		State:    marker.NodeStateReady,
	}
	return i
}

func TestReset(t *testing.T) {
	i := testIntent()
	s := testIntent()

	s.reset()

	// first action after reset
	assert.Check(t, s.Projected().Wanted == marker.NodeActionStablize)
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

func TestResetStates(t *testing.T) {
	s := testIntent()
	i := Intent{NodeName: s.NodeName}
	s.reset()
	assert.Check(t, i.Waiting())
	assert.Check(t, i.Needed())
	assert.Check(t, i.WantProgress())
	assert.Check(t, !i.Intrusive())
}

func TestIntentTruths(t *testing.T) {
	type pred = string

	testcases := []struct {
		name   string
		intent Intent
		truthy []pred
		falsy  []pred
	}{
		{
			name:   "empty",
			intent: Intent{},
			falsy:  []pred{"Needed"},
		},
		{
			name:   "reset",
			intent: func() Intent { i := testIntent(); i.reset(); return *i }(),
			truthy: []pred{"Needed", "WantProgress"},
			falsy:  []pred{"Intrusive"},
		},
		{
			name: "working state",
			intent: Intent{
				Wanted: marker.NodeActionStablize,
				Active: marker.NodeActionStablize,
				State:  marker.NodeStateBusy,
			},
			truthy: []pred{"InProgress"},
			falsy:  []pred{"Waiting"},
		},
		{
			name: "errored state",
			intent: Intent{
				Wanted: marker.NodeActionStablize,
				Active: marker.NodeActionStablize,
				State:  marker.NodeStateError,
			},
			truthy: []pred{"Errored", "Waiting"},
		},
	}

	for _, tc := range testcases {
		name := fmt.Sprintf("%s(%s-%s-%s)", tc.name, tc.intent.Wanted, tc.intent.Active, tc.intent.State)
		t.Run(name, func(t *testing.T) {
			tc.intent.NodeName = "state-machine"

			for _, predT := range tc.truthy {
				res, err := callCheck(&tc.intent, predT)
				assert.NilError(t, err)
				assert.Check(t, res == true, "%q expected to be true", predT)
			}
			for _, predF := range tc.falsy {
				res, err := callCheck(&tc.intent, predF)
				assert.NilError(t, err)
				assert.Check(t, res == false, "%q expected to be false", predF)
			}
		})
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
