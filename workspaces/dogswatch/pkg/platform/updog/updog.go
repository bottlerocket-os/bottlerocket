package updog

import (
	"context"
	"encoding/json"
	"os/exec"

	"github.com/pkg/errors"
)

var (
	updogBin = "updog"
)

// updog implements the binding for the platform to the host's implementation
// for manipulating updates on its behalf.
type updog struct {
	cli executer
}

type executer interface {
	execute(ctx context.Context, args []string, data interface{}) (interface{}, error)
}

func newUpdogHost() Host {
	return &updog{cli: &binExecute{}}
}

type binExecute struct{}

// binExecute abstracts the call to the backing `updog` executable.
func (*binExecute) execute(ctx context.Context, args []string, data interface{}) (interface{}, error) {
	cmd := exec.CommandContext(ctx, updogBin, args...)

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, errors.WithMessage(err, "could not get handle to stdout")
	}

	if err := cmd.Start(); err != nil {
		return nil, errors.WithMessagef(err, "failed to execute")
	}
	if err := json.NewDecoder(stdout).Decode(data); err != nil {
		return nil, errors.WithMessagef(err, "could not parse stdout")
	}
	err = cmd.Wait()
	return data, err
}

func (u *updog) Status() (*statusResponse, error) {
	action := CommandStatusQuery
	shape := &statusResponse{Action: action}
	response, err := u.cli.execute(context.TODO(), []string{action}, shape)
	if err != nil {
		return nil, errors.WithMessage(err, "could not get status")
	}
	return response.(*statusResponse), nil
}

func (u *updog) ListAvailable() (*listAvailableResponse, error) {
	action := CommandListAvailable
	shape := &listAvailableResponse{}
	response, err := u.cli.execute(context.TODO(), []string{action}, shape)
	if err != nil {
		return nil, errors.WithMessage(err, "could not get available updates")
	}
	return response.(*listAvailableResponse), nil
}

func (u *updog) PrepareUpdate(id UpdateID) (*prepareUpdateResponse, error) {
	action := CommandPrepareUpdate
	shape := &prepareUpdateResponse{Action: action}
	response, err := u.cli.execute(context.TODO(), []string{action}, shape)
	if err != nil {
		return nil, errors.WithMessage(err, "could not prepare update")
	}
	return response.(*prepareUpdateResponse), nil
}

func (u *updog) ApplyUpdate(id UpdateID) (*applyUpdateResponse, error) {
	action := CommandApplyUpdate
	shape := &applyUpdateResponse{Action: action}
	response, err := u.cli.execute(context.TODO(), []string{action}, shape)
	if err != nil {
		return nil, errors.WithMessage(err, "could not apply update")
	}
	return response.(*applyUpdateResponse), nil
}

func (u *updog) BootUpdate(id UpdateID, rebootNow bool) (*bootUpdateResponse, error) {
	action := CommandBootUpdate
	shape := &bootUpdateResponse{Action: action}
	response, err := u.cli.execute(context.TODO(), []string{action}, shape)
	if err != nil {
		return nil, errors.WithMessagef(err, "could not set update next boot (reboot:%t)", rebootNow)
	}

	return response.(*bootUpdateResponse), nil
}
