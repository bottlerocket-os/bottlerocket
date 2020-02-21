package marker

// NodeState indicates the general state of the Node as understood by the Node's
// Agent.
type NodeState = string

// TODO: timestamps should accompany the posted state for staleness checking and
// backoff operations.

const (
	NodeStateUnknown         NodeState = "unknown"
	NodeStateRefreshing      NodeState = "refreshing"
	NodeStateUpToDate        NodeState = "up-to-date"
	NodeStateUpdateAvailable NodeState = "update-available"
	NodeStateRebooting       NodeState = "rebooting"
	NodeStateReady           NodeState = "ready"
	NodeStateBusy            NodeState = "busy"
	NodeStateError           NodeState = "error"
)

// NodeAction indicates the permitted action to be taken on a Node by the Agent.
type NodeAction = string

const (
	NodeActionUnknown       NodeAction = "unknown"
	NodeActionStabilize     NodeAction = "stabilize"
	NodeActionReset         NodeAction = "reset-state"
	NodeActionPrepareUpdate NodeAction = "prepare-update"
	NodeActionPerformUpdate NodeAction = "perform-update"
	NodeActionRebootUpdate  NodeAction = "reboot-update"
)

// OperatorVersion describes compatibility versioning at the Operator level (the
// Controller and Node Agent).
type OperatorVersion = string

const (
	// OperatorUnknown is incompatible with all versions of the operator, it
	// should normally be unused.
	OperatorUnknown OperatorVersion = "0.0.0-unknown"

	OperatorV1Alpha OperatorVersion = "1.0.0-alpha"

	// OperatorDevelopmentDoNotUseInProduction is compatible with production
	// builds but should not be used in production. If this version is noted,
	// you should consider using a production build instead.
	OperatorDevelopmentDoNotUseInProduction OperatorVersion = "1.0.0-zeta+dev"
)

var (
	// OperatorBuildVersion is the version of the operator at compile time.
	OperatorBuildVersion = OperatorDevelopmentDoNotUseInProduction
)

// OperatorVersion describes compatibility versioning at the Platform level (the
// host component integrations).
//
// Note: the values placed on the resources do not use the internal enum type -
// they are strings.
type PlatformVersion = string

const (
	// PlatformUnknown is incompatible with all versions, it should normally be
	// unused.
	PlatformVersionUnknown PlatformVersion = "0.0.0"

	// PlatformV0 is the stubbed development mock up of the platform integration.
	PlatformV0 PlatformVersion = "0.1.0-zeta"

	// PlatformV1Alpha is the initial platform integration with a
	// to-be-stabilized interface.
	PlatformV1Alpha PlatformVersion = "1.0.0-alpha"

	// PlatformV1AlphaNoOp can be used to observe would-be actions in a cluster.
	// This version indicates that the compiled platform integration will not
	// perform any action, guaranteeing so by excluding the capability at build
	// time.
	PlatformV1AlphaNoOp PlatformVersion = "1.0.0-alpha+noop"

	// PlatformDevelopmentDoNotUseInProduction is compatible with production
	// builds but should not be used in production. If this version is noted,
	// you should consider using a production build instead.
	PlatformDevelopmentDoNotUseInProduction PlatformVersion = "0.1.0-zeta"
)

var (
	PlatformVersionBuild = PlatformDevelopmentDoNotUseInProduction
)

type NodeUpdate = string

const (
	NodeUpdateAvailable   NodeUpdate = "true"
	NodeUpdateUnavailable NodeUpdate = "false"
	NodeUpdateUnknown     NodeUpdate = "unknown"
)
