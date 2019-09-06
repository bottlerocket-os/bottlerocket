package constants

type AnnotationName string

const (
	aprefix                                  = "thar.amazonaws.com"
	AnnotationPrefix                         = aprefix
	AnnotationUpdateAvailable AnnotationName = aprefix + "/update-available"
	AnnotationPlatformVersion AnnotationName = aprefix + "/platform-version"
	AnnotationOperatorVersion AnnotationName = aprefix + "/operator-version"

	// TODO: name these better.. they need to communicate the status of the
	// node, the node's current state and the desired state for the node to
	// reach.
	AnnotationNodeStatus  AnnotationName = aprefix + "/node-status"
	AnnotationNodeAction  AnnotationName = aprefix + "/desired-state"
	AnnotationNodeDesired AnnotationName = aprefix + "/node-state"
)

type LabelName string

const (
	lprefix              = "thar.amazonaws.com"
	LabelPrefix          = lprefix
	LabelPlatformVersion = lprefix + "/platform-version"
	LabelChaotic         = lprefix + "/chaotic"
)
