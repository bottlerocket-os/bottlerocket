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
	// NodeName can be configured to scope the nodestream to a single node.
	NodeName string
	// ResyncPeriod is the time between complete resynchronization of the cached
	// resource data.
	ResyncPeriod time.Duration
}

func (c *Config) selector() func(options *metav1.ListOptions) {
	var (
		fieldSelector string
		labelSelector string
	)
	if c.NodeName != "" {
		// limit the streamed updates to the specified node.
		fieldSelector = "metadata.name=" + c.NodeName
	} else {
		labelSelector = marker.PlatformVersionKey
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
