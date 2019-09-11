package agent

import (
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
)

// NOTE: maybe add locked access, unsure what async boundaries this will cross.
type nodeState struct {
	mu *sync.RWMutex

	name            string
	action          string
	status          string
	state           string
	updateAvailable bool
	platformVersion string
	operatorVersion string

	staticAnnotations map[string]string
	staticLabels      map[string]string
}

func initialState() *nodeState {
	return &nodeState{
		mu: &sync.RWMutex{},

		name: "unset",
		//		Status:          marker.NodeStatusUnknown,
		action:          marker.NodeActionUnknown,
		state:           marker.NodeStateUnknown,
		updateAvailable: false,
		platformVersion: marker.PlatformVersionUnknown,
		operatorVersion: marker.OperatorBuildVersion,
	}
}

// Update permits a caller to make protected mutations to the nodeState.
func (s *nodeState) update(updateFn func(*nodeState) error) error {
	s.mu.Lock()
	err := updateFn(s)
	s.mu.Unlock()
	return err
}

// ResourceName is the name to be used
func (s *nodeState) resourceName() string {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.name
}

// patchJSON returns marshaled JSON for use with the Patch method to update a
// Node's resource metadata.
func (s *nodeState) patchJSON() ([]byte, error) {
	return json.Marshal(s.toObjectMeta())
}

func (s *nodeState) toObjectMeta() *metav1.ObjectMeta {
	s.mu.RLock()
	var meta metav1.ObjectMeta

	annos := map[string]string{
		"dev." + marker.Prefix + "/last-patch": time.Now().Format(time.RFC3339),
		marker.NodeStateKey:                    s.state,
		marker.NodeActionKey:                   s.action,
		marker.UpdateAvailableKey:              fmt.Sprintf("%t", s.updateAvailable),
		marker.PlatformVersionKey:              s.platformVersion,
		marker.OperatorVersionKey:              s.operatorVersion,
	}
	extendMap(annos, s.staticAnnotations)

	labels := map[string]string{
		marker.PlatformVersionKey: s.platformVersion,
		marker.OperatorVersionKey: s.operatorVersion,
	}
	extendMap(labels, s.staticLabels)

	meta.SetAnnotations(annos)
	meta.SetLabels(labels)
	s.mu.RUnlock()
	return &meta
}

// extendMap copies kv's to `into` from `from` without concern for `into`'s
// keyspace.
func extendMap(into map[string]string, from map[string]string) {
	if into == nil || from == nil {
		return
	}
	for k, _ := range from {
		into[k] = from[k]
	}
}
