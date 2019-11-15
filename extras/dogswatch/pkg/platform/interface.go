package platform

import "github.com/pkg/errors"

// Platform is implemented by owners of progress makers.
type Platform interface {
	// Status reports the underlying platform's health and metadata.
	Status() (Status, error)
	// ListAvailable provides the list of updates that a platform is offering
	// for use. The list MUST be ordered in preference as well as recency.
	ListAvailable() (Available, error)
	// Prepare causes the platform to take steps towards an update without
	// committing to it. For example, a platform may require steps to preform
	// pre-flight checks or initialization migrations prior to executing an
	// update.
	Prepare(target Update) error
	// Update causes the platform to commit to an update taking potentially
	// irreversible steps to do so.
	Update(target Update) error
	// BootUpdate causes the platform to configure itself to use the update on
	// next boot. Optionally, the caller may indicate that the update should be
	// immediately rebooted to use.
	BootUpdate(target Update, rebootNow bool) error
}

// Status reports the readiness of the underlying platform.
type Status interface {
	// OK will return true when the platform is able to assert its status
	// response is accurately reporting from the underlying components.
	OK() bool
}

// Available is a listing of available Updates offered by the platform.
type Available interface {
	// Updates returns a list of Updates that may be applied.
	Updates() []Update
}

// Update is a distinct update that may be applied.
type Update interface {
	// Identifier is an opaque identifier used by the platform to coordinate its
	// updates internally. Callers should not introspect this value and instead
	// should pass this along through related update control methods.
	Identifier() interface{}
}

// Ping the platform to verify its liveliness and general usability based on its
// status. Platform consumers should utilize this method to consistently
// validate the platform before use.
func Ping(p Platform) error {
	status, err := p.Status()
	if err != nil {
		return errors.WithMessage(err, "could not retrieve platform status")
	}
	if !status.OK() {
		return errors.New("platform did not report OK status")
	}
	return nil
}
