package nodestream

import (
	"time"

	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
)

const (
	defaultResyncPeriod = time.Minute * 10
)

type Config struct {
	// NodeName limits the nodestream to a single Node resource with the
	// provided name.
	NodeName string
	// ResyncPeriod is the time between complete resynchronization of the cached
	// resource data.
	ResyncPeriod time.Duration
	// PlatformVersion, when specified, limits the nodestream to Nodes that are
	// labeled with the provided PlatformVersion.
	PlatformVersion marker.PlatformVersion
	// OperatorVersion, when specified, limits the nodestream to Nodes that are
	// labeled with the provided OperatorVersion.
	OperatorVersion marker.OperatorVersion
	// LabelSelectorExtra is a free-form selector appended to the calculated
	// selector.
	LabelSelectorExtra string
	// FieldSelectorExtra is a free-form selector appended to the calculated
	// selector.
	FieldSelectorExtra string
}

// TODO: test this func

func (c *Config) selector() func(options *metav1.ListOptions) {
	var (
		fieldSelector string
		labelSelector string
	)
	if c.NodeName != "" {
		// limit the streamed updates to the specified node.
		fieldSelector = "metadata.name=" + c.NodeName
	}

	labelSelector = marker.PlatformVersionKey

	if c.LabelSelectorExtra != "" {
		if labelSelector != "" {
			labelSelector += ","
		}
		labelSelector += c.LabelSelectorExtra
	}

	if c.FieldSelectorExtra != "" {
		if fieldSelector != "" {
			fieldSelector += ","
		}
		fieldSelector += c.FieldSelectorExtra
	}

	return func(options *metav1.ListOptions) {
		options.LabelSelector = labelSelector
		options.FieldSelector = fieldSelector
	}
}

func (c *Config) resyncPeriod() time.Duration {
	if c.ResyncPeriod == 0 {
		return defaultResyncPeriod
	}
	return c.ResyncPeriod
}
