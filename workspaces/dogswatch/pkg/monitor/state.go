package monitor

import (
	"encoding/json"

	"github.com/amazonlinux/thar/dogswatch/pkg/constants"
	"github.com/pkg/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/types"
	v1 "k8s.io/client-go/kubernetes/typed/core/v1"
)

type State struct {
	NodeStatus      constants.NodeAction
	NodeAction      constants.NodeAction
	UpdateAvailable constants.NodeState
	PlatformVersion constants.PlatformVersion
	OperatorVersion constants.OperatorVersion
}

func (s State) Annotations() map[string]string {
	return map[string]string{
		string(constants.AnnotationUpdateAvailable): string(s.UpdateAvailable),
		string(constants.AnnotationPlatformVersion): string(s.PlatformVersion),
		string(constants.AnnotationOperatorVersion): string(s.OperatorVersion),
	}
}

func (s State) Labels() map[string]string {
	return map[string]string{
		string(constants.LabelPlatformVersion): string(s.PlatformVersion),
	}
}

func (s State) ToObjectMeta() *metav1.ObjectMeta {
	var meta metav1.ObjectMeta
	meta.SetAnnotations(s.Annotations())
	meta.SetLabels(s.Labels())
	return &meta
}

func (s State) PatchNode(nodeClient v1.NodeInterface, nodeName string) error {
	patchData, err := json.Marshal(s.ToObjectMeta())
	_, err := nodeClient.Patch(nodeName, types.JSONPatchType, patchData)
	return errors.WithMessagef(err, "could not update state metadata on %q", nodeName)
}

// statePatch is the structure used for sending a PATCH request for modifying an
// object's state representation. Each time it is submitted, the values should
// be fully representative.
type statePatch struct {
	Metadata struct {
		Annotations map[string]string `json:"annotations,omitempty"`
		Labels      map[string]string `json:"labels,omitempty"`
	} `json:"metadata,omitempty"`
}
