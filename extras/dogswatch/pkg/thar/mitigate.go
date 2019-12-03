package thar

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/pkg/errors"
)

var mitigations = []mitigation{
	&containerdDropIn{},
}

// mitigation applies a change, if needed, to permit dogswatch operation.
type mitigation interface {
	Name() string
	Check() (bool, error)
	Apply() (bool, error)
}

func ApplyMitigations() error {
	log := logging.New("mitigation")
	errored := false
	applied := false

	for _, m := range mitigations {
		mlog := log.WithField("mitigation", m.Name())
		needed, err := m.Check()
		if err != nil {
			errored = true
			mlog.WithError(err).Error("unable to determine need")
			continue
		}
		if !needed {
			errored = true
			mlog.Debug("not needed")
			continue
		}

		applied = true
		mlog.Warn("applying mitigation")
		applied, err := m.Apply()
		if err != nil {
			errored = true
			mlog.WithError(err).Error("unable to apply")

			continue
		}
		if !applied {
			errored = true
			mlog.Error("unsuccessful")
			continue
		}
		mlog.Warn("applied mitigation")
	}

	if errored {
		err := errors.New("errors occurred during mitigation fixes")
		log.WithError(err).Error("see log for mitigation attempts")
		return err
	}

	if applied {
		log.Info("applied all mitigations")
	}

	return nil
}
