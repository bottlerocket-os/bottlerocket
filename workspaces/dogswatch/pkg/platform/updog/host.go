package updog

import "github.com/amazonlinux/thar/dogswatch/pkg/platform"

// Host is the integration for this platform and the host - this is a very light
// mapping between the platform requirements and the implementation backing the
// interaction itself.
type Host interface {
	Status() (*statusResponse, error)
	ListAvailable() (*listAvailableResponse, error)
	PrepareUpdate(id UpdateID) (*prepareUpdateResponse, error)
	ApplyUpdate(id UpdateID) (*applyUpdateResponse, error)
	BootUpdate(id UpdateID, rebootNow bool) (*bootUpdateResponse, error)
}

type hostOption struct {
	Wait bool     `json:"wait"`
	ID   UpdateID `json:"id"`
}

// UpdateID is the type of the opaque Identifier used for this platform.
type UpdateID string

type actionResponse struct {
	// ID is the Update's ID which should match throughout the orchestration of
	// an update.
	ID UpdateID `json:"id"`
	// Status is the presently executing or executed action.
	Status hostStatus `json:"status"`
	// Action is the presently requested action.
	Action hostCommand `json:"action"`
}

var _ platform.Status = (*statusResponse)(nil)

type statusResponse actionResponse

func (sr *statusResponse) OK() bool {
	return sr.Status != ""
}

type applyUpdateResponse actionResponse
type prepareUpdateResponse actionResponse
type bootUpdateResponse actionResponse

var _ platform.Available = (*listAvailableResponse)(nil)

type listAvailableResponse struct {
	Schema          string             `json:"schema"`
	ReportedUpdates []*availableUpdate `json:"updates"`
}

func (l *listAvailableResponse) Updates() []platform.Update {
	us := make([]platform.Update, len(l.ReportedUpdates))
	for i := range l.ReportedUpdates {
		us[i] = l.ReportedUpdates[i]
	}
	return us
}

var _ platform.Update = (*availableUpdate)(nil)

type availableUpdate struct {
	ID         UpdateID `json:"id"`
	Applicable bool     `json:"applicable"`
	Flavor     string   `json:"flavor"`
	Arch       string   `json:"arch"`
	Version    string   `json:"version"`
	Status     string   `json:"status"`
}

func (u *availableUpdate) Identifier() interface{} {
	return u.ID
}
