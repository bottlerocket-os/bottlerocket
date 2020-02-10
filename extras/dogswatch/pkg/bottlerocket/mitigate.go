package bottlerocket

import (
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/pkg/errors"
)

var mitigations = []mitigation{
	&containerdDropIn{},
}

// mitigation applies a change, if needed, to permit dogswatch operation.
type mitigation interface {
	Name() string
	Check(logging.SubLogger) (bool, error)
	Apply(logging.SubLogger) (bool, error)
}

func ApplyMitigations() error {
	log := logging.New("mitigation")
	errored := false
	applied := false

	for _, m := range mitigations {
		mlog := log.WithField("mitigation", m.Name())
		needed, err := m.Check(log)
		if err != nil {
			errored = true
			mlog.WithError(err).Error("unable to determine need")
			continue
		}
		if !needed {
			mlog.Debug("not needed")
			continue
		}

		applied = true
		mlog.Warn("applying mitigation")
		applied, err := m.Apply(log)
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
		log.Info("applied mitigations")
	}

	return nil
}
