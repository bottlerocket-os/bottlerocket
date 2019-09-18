package marker

type Key = string

const (
	Prefix = "thar.amazonaws.com"

	// UpdateAvailableKey is used to identify a Node as having an update
	// available. The value itself is not checked at this time but may be used
	// to communicate a version at a later time.
	UpdateAvailableKey Key = Prefix + "/update-available"

	// TODO: Use this and post "statically" at kubelet launch
	//
	// PlatformVersionKey is where the compatibility version is posted for the
	// given Node.
	PlatformVersionKey Key = Prefix + "/platform-version"
	// OperatorVersionKey is where the compatibility version is posted for the
	// given Node. This version describes the understood "protocol" between
	// Operating Controller and the managed Nodes.
	OperatorVersionKey Key = Prefix + "/operator-version"

	// TODO: name these better.. they need to communicate the status of the
	// node, the node's current state and the desired state for the node to
	// reach.

	// NodeActionWanted provides the Node with the Controller's wanted action to
	// make update progress.
	NodeActionWanted Key = Prefix + "/action-wanted"
	// NodeActionActiveStatus provides progress information on a
	NodeActionActiveState Key = Prefix + "/action-state"
	// NodeActionActive provides the acknowledged and acted-upon action that was
	// wanted of a Node.
	NodeActionActive Key = Prefix + "/action-active"

	ChaoticKey Key = Prefix + "/chaotic"

	NodeSelectorLabel = PlatformVersionKey
	PodSelectorLabel  = PlatformVersionKey
)
