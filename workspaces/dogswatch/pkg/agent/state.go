package agent

import (
	"encoding/json"
	"time"

	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
)

// NOTE: maybe add locked access, unsure what async boundaries this will cross.
type State struct {
	NodeName string

	Status          string
	State           string
	UpdateAvailable string
	PlatformVersion string
	OperatorVersion string
}

// PatchJSON returns marshaled JSON for use with the Patch method to update a
// Node's resource metadata.
func (s *State) PatchJSON() ([]byte, error) {
	return json.Marshal(s.toObjectMeta())
}

func (s *State) toObjectMeta() *metav1.ObjectMeta {
	var meta metav1.ObjectMeta

	meta.SetAnnotations(map[string]string{
		"dev." + marker.Prefix + "/last-patch": time.Now().Format(time.RFC3339),

		marker.NodeStateKey:       marker.NodeStateUnknown,
		marker.NodeActionKey:      marker.NodeActionUnknown,
		marker.UpdateAvailableKey: marker.False,
		marker.PlatformVersionKey: marker.PlatformUnknown,
		marker.OperatorVersionKey: marker.OperatorBuildVersion,
	})

	meta.SetLabels(map[string]string{
		marker.PlatformVersionKey: marker.PlatformUnknown,
		marker.OperatorVersionKey: marker.OperatorBuildVersion,
	})

	return &meta
}
