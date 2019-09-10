package agent

import (
	"k8s.io/apimachinery/pkg/types"
)

func (a *Agent) patchState() error {
	nc := a.kube.CoreV1().Nodes()
	patchJSON, err := a.state.PatchJSON()
	if err != nil {
		return err
	}
	_, err = nc.Patch(a.state.NodeName, types.JSONPatchType, patchJSON)
	return err
}
