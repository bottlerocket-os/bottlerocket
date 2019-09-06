package updog

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/platform"
	"github.com/pkg/errors"
)

// Assert Updog as a platform implementor.
var _ platform.Platform = (*Platform)(nil)

type Platform struct {
	host Host
}

func New() (*Platform, error) {
	return &Platform{host: &updog{}}, nil
}

// Status reports the underlying platform's health and metadata.
func (p *Platform) Status() (platform.Status, error) {
	return p.host.Status()
}

// ListAvailable provides the list of updates that a platform is offering
// for use. The list MUST be ordered in preference as well as recency.
func (p *Platform) ListAvailable() (platform.Available, error) {
	return p.host.ListAvailable()
}

// Prepare causes the platform to take steps towards an update without
// committing to it. For example, a platform may require steps to preform
// pre-flight checks or initialization migrations prior to executing an
// update.
func (p *Platform) Prepare(target platform.Update) error {
	id, err := targetID(target)
	if err != nil {
		return err
	}
	_, err = p.host.PrepareUpdate(id)
	return err
}

// Update causes the platform to commit to an update taking potentially
// irreversible steps to do so.
func (p *Platform) Update(target platform.Update) error {
	id, err := targetID(target)
	if err != nil {
		return err
	}
	_, err = p.host.ApplyUpdate(id)
	return err
}

// BootUpdate causes the platform to configure itself to use the update on
// next boot. Optionally, the caller may indicate that the update should be
// immediately rebooted to use.
func (p *Platform) BootUpdate(target platform.Update, rebootNow bool) error {
	id, err := targetID(target)
	if err != nil {
		return err
	}
	_, err = p.host.BootUpdate(id, rebootNow)
	return err
}

func targetID(target platform.Update) (UpdateID, error) {
	id, ok := target.Identifier().(UpdateID)
	if !ok {
		return "", errors.Errorf("provided incompatible target identifier %v", target.Identifier())
	}
	return id, nil
}
