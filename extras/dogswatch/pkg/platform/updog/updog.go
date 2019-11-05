package updog

import (
	"bufio"
	"bytes"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/thar"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
)

var (
	updogBin = filepath.Join(thar.PlatformBin, "updog")
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
	cmd.SysProcAttr = thar.ProcessAttrs()

	var buf bytes.Buffer
	writer := bufio.NewWriter(&buf)
	cmd.Stdout = writer
	cmd.Stderr = writer

	if logging.Debuggable {
		logging.New("updog").WithFields(logrus.Fields{
			"cmd": cmd.String(),
		}).Debug("Executing")
	}

	if err := cmd.Start(); err != nil {
		if logging.Debuggable {
			logging.New("updog").WithFields(logrus.Fields{
				"cmd":    cmd.String(),
				"output": buf.String(),
			}).WithError(err).Error("Failed to start command")
		}
		return false, err
	}
	err := cmd.Wait()
	if err != nil {
		if logging.Debuggable {
			logging.New("updog").WithFields(logrus.Fields{
				"cmd":    cmd.String(),
				"output": buf.String(),
			}).WithError(err).Error("Command errored durring run")
		}
		return false, err
	}
	if logging.Debuggable {
		logging.New("updog").WithFields(logrus.Fields{
			"cmd":    cmd.String(),
			"output": buf.String(),
		}).Debug("Command completed successfully")
	}
	// Boolean currently only used by ListUpdate. Returns true if the
	// command yielded output, which indicates an update is available.
	// TODO: Update this when an interface is defined between updog
	// and dogswatch.
	return len(buf.String()) > 0, err
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
	_, err := os.Stat(thar.RootFS + updogBin)
	if err != nil {
		return false, errors.Wrap(err, "updog not found in thar container mount "+thar.RootFS)
	}
	// TODO: add support for an updog usability check
	return true, err
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
	if avail, err := u.Bin.CheckUpdate(); err != nil {
		return nil, err
	} else {
		if avail {
			return &listAvailableResponse{
				// TODO: deserialize output from updog and plumb version IDs
				ReportedUpdates: []*availableUpdate{&availableUpdate{ID: "POSITIVE_STUB_INDICATOR"}},
			}, nil
		} else {
			return &listAvailableResponse{}, nil
		}
	}
}

func (u *updog) PrepareUpdate(id UpdateID) (*prepareUpdateResponse, error) {
	// TODO: extend updog for prepare steps.
	return &prepareUpdateResponse{
		ID: "POSITIVE_STUB_INDICATOR",
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
