package marker

type Key = string

const (
	Prefix = "thar.amazonaws.com"

	UpdateAvailableKey Key = Prefix + "/update-available"
	PlatformVersionKey Key = Prefix + "/platform-version"
	OperatorVersionKey Key = Prefix + "/operator-version"

	// TODO: name these better.. they need to communicate the status of the
	// node, the node's current state and the desired state for the node to
	// reach.
	NodeStatusKey Key = Prefix + "/node-status"
	NodeActionKey Key = Prefix + "/desired-state"
	NodeStateKey  Key = Prefix + "/node-state"

	LabelPrefix              = Prefix
	PlatformVersionLabel Key = Prefix + "/platform-version"
	ChaoticLabel         Key = Prefix + "/chaotic"
)
