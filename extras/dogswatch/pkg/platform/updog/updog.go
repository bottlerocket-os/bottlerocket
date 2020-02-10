package updog

import (
	"bufio"
	"bytes"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/bottlerocket"
	"github.com/pkg/errors"
)

var (
	updogBin = filepath.Join(bottlerocket.PlatformBin, "updog")
)

const (
	// updateIdentifier is a stand-in update identifier for Updog sourced updates.
	updateIdentifier = "latest"
)

// updog implements the binding for the platform to the host's implementation
// for manipulating updates on its behalf.
type updog struct {
	Bin command

	log logging.Logger
}

type command interface {
	CheckUpdate() (bool, error)
	Update() error
	UpdateImage() error
	Reboot() error
	Status() (bool, error)
}

type executable struct {
	log logging.SubLogger
}

func (e *executable) runOk(cmd *exec.Cmd) (bool, error) {
	cmd.SysProcAttr = bottlerocket.ProcessAttrs()

	var buf bytes.Buffer
	writer := bufio.NewWriter(&buf)
	cmd.Stdout = writer
	cmd.Stderr = writer

	log := e.log.WithField("cmd", cmd.String())
	log.Debug("running command")
	if err := cmd.Start(); err != nil {
		log.WithError(err).Error("failed to start command")
		if logging.Debuggable {
			log.WithField("output", buf.String()).Debugf("command output")
		}
		return false, err
	}
	err := cmd.Wait()
	if err != nil {
		log.WithError(err).Error("error during command run")
		if logging.Debuggable {
			log.WithField("output", buf.String()).Debug("command output")
		}
		return false, err
	}
	log.Debug("command completed successfully")
	if logging.Debuggable {
		log.WithField("output", buf.String()).Debug("command output")
	}
	// Boolean currently only used by ListUpdate. Returns true if the
	// command yielded output, which indicates an update is available.
	// TODO: Update this when an interface is defined between updog
	// and dogswatch.
	updateEmitted := len(buf.String()) > 0
	return updateEmitted, err
}

func (e *executable) CheckUpdate() (bool, error) {
	return e.runOk(exec.Command(updogBin, "check-update"))
}

func (e *executable) Update() error {
	_, err := e.runOk(exec.Command(updogBin, "update-apply", "-r"))
	return err
}

func (e *executable) UpdateImage() error {
	_, err := e.runOk(exec.Command(updogBin, "update-image"))
	return err
}

func (e *executable) Reboot() error {
	// TODO: reboot
	_, err := e.runOk(exec.Command("reboot"))
	return err
}

func (e *executable) Status() (bool, error) {
	_, err := os.Stat(bottlerocket.RootFS + updogBin)
	if err != nil {
		return false, errors.Wrap(err, "updog not found in bottlerocket container mount "+bottlerocket.RootFS)
	}
	// TODO: add support for an updog usability check
	return true, err
}

func newUpdogHost() Host {
	log := logging.New("updog")

	return &updog{
		Bin: &executable{log: log.WithField(logging.SubComponentField, "host-bin")},
		log: log,
	}
}

func (u *updog) Status() (*statusResponse, error) {
	if _, err := u.Bin.Status(); err != nil {
		return nil, err
	}
	return &statusResponse{}, nil
}

func (u *updog) ListAvailable() (*listAvailableResponse, error) {
	avail, err := u.Bin.CheckUpdate()
	if err != nil {
		return nil, errors.Wrap(err, "unable to check for updates")
	}
	if avail {
		return &listAvailableResponse{
			// TODO: deserialize output from updog and plumb version IDs
			ReportedUpdates: []*availableUpdate{&availableUpdate{ID: updateIdentifier}},
		}, nil
	}
	return &listAvailableResponse{}, nil
}

func (u *updog) PrepareUpdate(id UpdateID) (*prepareUpdateResponse, error) {
	// TODO: extend updog for prepare steps.
	return &prepareUpdateResponse{
		ID: updateIdentifier,
	}, nil
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
