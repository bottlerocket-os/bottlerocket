package updog

type hostCommand string

const (
	CommandStatusQuery   hostCommand = "status"
	CommandListAvailable hostCommand = "list-available"
	CommandPrepareUpdate hostCommand = "prepare-update"
	CommandApplyUpdate   hostCommand = "apply-update"
	CommandBootUpdate    hostCommand = "boot-update"
)

func (h hostCommand) String() string {
	return string(h)
}

type hostStatus string

const (
	StatusBootable        hostStatus = "bootable"
	StatusUpToDate                   = "up-to-date"
	StatusUpdateAvailable            = "available"
	StatusUpdateApplied              = "applied"
	StatusUpdatePrepared             = "prepared"
	StatusPendingAction              = "pending"
)

func (h hostStatus) String() string {
	return string(h)
}
