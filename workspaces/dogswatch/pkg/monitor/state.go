package monitor

import (
	"fmt"

	"github.com/amazonlinux/thar/dogswatch/pkg/constants"
)

type State struct {
	NodeStatus      constants.NodeAction
	NodeAction      constants.NodeAction
	UpdateAvailable constants.NodeState
	PlatformVersion constants.PlatformVersion
	OperatorVersion constants.OperatorVersion
}

func (s State) Annotations() map[constants.AnnotationName]string {
	str := func(c interface{}) string {
		return fmt.Sprintf("%s", c)
	}
	return map[constants.AnnotationName]string{
		constants.AnnotationUpdateAvailable: str(s.UpdateAvailable),
		constants.AnnotationPlatformVersion: str(s.PlatformVersion),
		constants.AnnotationOperatorVersion: str(s.OperatorVersion),
	}
}
