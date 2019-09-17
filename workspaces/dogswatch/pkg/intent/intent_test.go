package intent

import (
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
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
	assert.Check(t, s.Wanted == marker.NodeActionStablize)
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
