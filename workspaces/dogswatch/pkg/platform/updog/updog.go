package updog

import (
	"os/exec"
)

var (
	updogBin = "updog"
)

// updog implements the binding for the platform to the host's implementation
// for manipulating updates on its behalf.
type updog struct {
	Bin command
}

type command interface {
	CheckUpdate() (bool, error)
	Update() error
	UpdateImage() error
	Reboot() error
	Status() (bool, error)
}

type executable struct{}

func (e *executable) runOk(cmd *exec.Cmd) (bool, error) {
	if err := cmd.Start(); err != nil {
		return false, err
	}
	err := cmd.Wait()
	return err == nil, err
}

func (e *executable) CheckUpdate() (bool, error) {
	return e.runOk(exec.Command(updogBin, "check-update"))
}

func (e *executable) Update() error {
	_, err := e.runOk(exec.Command(updogBin, "update"))
	return err
}

func (e *executable) UpdateImage() error {
	_, err := e.runOk(exec.Command(updogBin, "update-image"))
	return err
}

func (e *executable) Reboot() error {
	// TODO: reboot
	_, err := e.runOk(exec.Command("echo", "reboot"))
	return err
}

func (e *executable) Status() (bool, error) {
	_, err := exec.LookPath(updogBin)
	return err == nil, err
}

func newUpdogHost() Host {
	return &updog{Bin: &executable{}}
}

func (u *updog) Status() (*statusResponse, error) {
	if _, err := u.Bin.Status(); err != nil {
		return nil, err
	}
	return &statusResponse{}, nil
}

func (u *updog) ListAvailable() (*listAvailableResponse, error) {
	if _, err := u.Bin.CheckUpdate(); err != nil {
		return nil, err
	}
	return &listAvailableResponse{}, nil
}

func (u *updog) PrepareUpdate(id UpdateID) (*prepareUpdateResponse, error) {
	// TODO: extend updog for prepare steps
	return &prepareUpdateResponse{}, nil
}

func (u *updog) ApplyUpdate(id UpdateID) (*applyUpdateResponse, error) {
	if err := u.Bin.UpdateImage(); err != nil {
		return nil, err
	}
	return &applyUpdateResponse{}, nil
}

func (u *updog) BootUpdate(id UpdateID, rebootNow bool) (*bootUpdateResponse, error) {
	if err := u.Bin.Update(); err != nil {
		return nil, err
	}
	return &bootUpdateResponse{}, nil
}
