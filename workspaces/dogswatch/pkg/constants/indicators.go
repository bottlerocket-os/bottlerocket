//go:generate stringer -linecomment=true -type=NodeState,NodeAction,OperatorVersion,PlatformVersion -output indicators.gen.go
package constants

// NodeState indicates the general state of the Node as understood by the Node's
// Agent.
//
// Note: the values placed on the resources do not use the internal enum type -
// they are strings.
type NodeState uint8

const (
	NodeStateUnknown         NodeState = iota // unknown
	NodeStateRefreshing                       // refreshing
	NodeStateUpToDate                         // up-to-date
	NodeStateUpdateAvailable                  // update-available
	NodeStateRebooting                        // rebooting
)

// NodeAction indicates the permitted action to be taken on a Node by the Agent.
//
// Note: the values placed on the resources do not use the internal enum type -
// they are strings.
type NodeAction uint8

const (
	NodeActionUnknown       NodeAction = iota
	NodeActionStablize                 // stablize
	NodeActionReset                    // reset-state
	NodeActionPrepareUpdate            // prepare-update
	NodeActionPerformUpdate            // perform-update
	NodeActionRebootUpdate             // reboot-update
)

// OperatorVersion describes compatibility versioning at the Operator level (the
// Controller and Node Agent).
//
// Note: the values placed on the resources do not use the internal enum type -
// they are strings.
type OperatorVersion uint8

const (
	// OperatorUnknown is incompatible with all versions of the operator, it
	// should normally be unused.
	OperatorUnknown OperatorVersion = iota // 0.0.0-unknown

	OperatorV1Alpha // 1.0.0-alpha

	// OperatorDevelopmentDoNotUseInProduction is compatible with production
	// builds but should not be used in production. If this version is noted,
	// you should consider using a production build instead.
	OperatorDevelopmentDoNotUseInProduction // 1.0.0-zeta
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
type PlatformVersion uint8

const (
	// PlatformUnknown is incompatible with all versions, it should normally be
	// unused.
	PlatformUnknown PlatformVersion = iota // 0.0.0

	// PlatformV0 is the stubbed development mock up of the platform integration.
	PlatformV0 // 0.1.0-zeta

	// PlatformV1Alpha is the initial platform integration with a
	// to-be-stablized interface.
	PlatformV1Alpha // 1.0.0-alpha

	// PlatformV1AlphaNoOp can be used to observe would-be actions in a cluster.
	// This version indicates that the compiled platform integration will not
	// perform any action, guranteeing so by excluding the capability at build
	// time.
	PlatformV1AlphaNoOp // 1.0.0-alpha+noop

	// PlatformDevelopmentDoNotUseInProduction is compatible with production
	// builds but should not be used in production. If this version is noted,
	// you should consider using a production build instead.
	PlatformDevelopmentDoNotUseInProduction // 0.1.0-zeta
)

var (
	PlatformBuildVersion = PlatformDevelopmentDoNotUseInProduction
)
