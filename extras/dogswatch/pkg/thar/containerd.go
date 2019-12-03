package thar

import (
	"io"
	"os"
	"path/filepath"
	"strconv"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	systemd "github.com/coreos/go-systemd/v22/dbus"
	"github.com/coreos/go-systemd/v22/unit"
	dbus "github.com/godbus/dbus/v5"
	"github.com/pkg/errors"
)

var (
	systemdUnitTransient = filepath.Join(RootFS, "/run/systemd/system")
	systemdSocket        = filepath.Join(RootFS, "/run/systemd/private")

	containerdUnit      = "containerd.service"
	containerdDropInDir = filepath.Join(systemdUnitTransient, containerdUnit+".d")

	containerdKillMode = "mixed"
)

type containerdDropIn struct{}

func (*containerdDropIn) Name() string {
	return "containerd-killmode"
}

func (c *containerdDropIn) Apply() (bool, error) {
	err := c.writeUnit()
	if err != nil {
		return false, err
	}
	err = c.reloadUnit()
	if err != nil {
		return false, err
	}
	return true, nil
}

func (c *containerdDropIn) Check() (bool, error) {
	if !c.runEnvironment() {
		return false, nil
	}

	conn, err := c.connect()
	if err != nil {
		return false, err
	}
	defer conn.Close()

	prop, err := conn.GetUnitTypeProperty(containerdUnit, "Service", "KillMode")
	if err != nil {
		return false, errors.Wrap(err, "unable to query service unit")
	}
	variant := prop.Value
	if mode, ok := variant.Value().(string); ok {
		log.WithField("KillMode", mode).Debugf("identified %s KillMode", containerdUnit)
		if mode == containerdKillMode {
			log.Debug("mitigation not required")
			return false, nil
		}
	} else {
		// KillMode property wasn't a string, but it should be.
		log.Debugf("failed to reflect string for property %q", "KillMode")
		log.Debugf("property object %#v", prop)
		return false, errors.Errorf("unable to handle queried property: %q", prop)
	}
	return true, nil
}

func (*containerdDropIn) runEnvironment() bool {
	// This doesn't apply without having root.
	if os.Geteuid() != 0 {
		return false
	}

	// And needs systemd access
	stat, err := os.Stat(systemdSocket)
	if err != nil {
		return false
	}
	isSocket := stat.Mode()&os.ModeSocket == os.ModeSocket
	if !isSocket {
		return false
	}
	return true
}

func (*containerdDropIn) writeUnit() error {
	// Drop-In Unit
	options := []*unit.UnitOption{
		unit.NewUnitOption("Service", "KillMode", containerdKillMode),
	}

	err := os.MkdirAll(containerdDropInDir, 0750)
	if err != nil {
		return errors.Wrap(err, "unable to create transient unit dir")
	}

	f, err := os.Create(filepath.Join(containerdDropInDir, "99-killmode-workaround.conf"))
	if err != nil {
		return errors.Wrap(err, "unable to create drop in unit")
	}
	_, err = io.Copy(f, unit.Serialize(options))
	if err != nil {
		f.Close()
		os.Remove(f.Name())
		return errors.Wrap(err, "unable to write drop in unit")
	}
	f.Close()
	return nil
}

func (c *containerdDropIn) reloadUnit() error {
	sd, err := c.connect()
	if err != nil {
		return errors.Wrap(err, "unable to connect to systemd")
	}
	defer sd.Close()

	err = sd.Reload()
	if err != nil {
		return errors.Wrap(err, "unable to execute daemon-reload")
	}
	// For now, this is all that's needed.
	return nil
}

func (c *containerdDropIn) connect() (*systemd.Conn, error) {
	dialer := func() (*dbus.Conn, error) {
		// Connect to the thar systemd socket
		conn, err := dbus.Dial("unix:path=" + systemdSocket)
		if err != nil {
			return nil, errors.Wrap(err, "unable to connect to thar systemd socket")
		}
		// Authenticate with the user's authority.
		methods := []dbus.Auth{dbus.AuthExternal(strconv.Itoa(os.Getuid()))}
		err = conn.Auth(methods)
		if err != nil {
			conn.Close()
			return nil, errors.Wrap(err, "unable to authenticate with thar systemd")
		}
		return conn, nil
	}
	return systemd.NewConnection(dialer)
}
