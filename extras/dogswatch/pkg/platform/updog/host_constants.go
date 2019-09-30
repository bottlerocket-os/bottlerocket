package updog

type hostCommand = string

const (
	CommandStatusQuery   hostCommand = "status"
	CommandListAvailable hostCommand = "list-available"
	CommandPrepareUpdate hostCommand = "prepare-update"
	CommandApplyUpdate   hostCommand = "apply-update"
	CommandBootUpdate    hostCommand = "boot-update"
)

type hostStatus = string

const (
	StatusBootable        hostStatus = "bootable"
	StatusUpToDate        hostStatus = "up-to-date"
	StatusUpdateAvailable hostStatus = "available"
	StatusUpdateApplied   hostStatus = "applied"
	StatusUpdatePrepared  hostStatus = "prepared"
	StatusPendingAction   hostStatus = "pending"
)
