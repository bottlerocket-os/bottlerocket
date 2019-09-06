package agent

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/monitor"
	"github.com/amazonlinux/thar/dogswatch/pkg/platform"
)

type Agent struct {
	platform platform.Platform
	monitor  monitor.Node
}
