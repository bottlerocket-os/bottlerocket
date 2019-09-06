package updog

import (
	"context"
	"errors"
	"testing"

	"gotest.tools/assert"
)

type testExecuter struct {
	t *testing.T

	expectedArgs []string
	data         interface{}
	returnErr    error
}

func (fake *testExecuter) execute(_ context.Context, args []string, data interface{}) (interface{}, error) {
	assert.DeepEqual(fake.t, fake.expectedArgs, args)
	return fake.data, fake.returnErr
}

func testUpdog(t *testing.T) (*updog, *testExecuter) {
	executer := &testExecuter{
		t: t,
	}
	return &updog{
		cli: executer,
	}, executer
}

func TestUpdogErrors(t *testing.T) {
	u, fake := testUpdog(t)
	fake.expectedArgs = []string{"status"}
	fake.returnErr = errors.New("fake")
	_, err := u.Status()
	assert.Assert(t, err != nil)
}

func TestUpdogStatus(t *testing.T) {
	var id UpdateID = "fake"
	u, fake := testUpdog(t)
	fake.expectedArgs = []string{"status"}
	fake.data = &statusResponse{
		Action: CommandStatusQuery,
		Status: StatusUpToDate,
		ID:     id,
	}
	status, err := u.Status()
	assert.NilError(t, err)
	assert.Equal(t, status.ID, id)
	assert.Assert(t, status.OK())
}
