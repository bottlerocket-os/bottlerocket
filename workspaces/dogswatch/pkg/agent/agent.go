package agent

import (
	"context"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/platform"
	"github.com/pkg/errors"
	"k8s.io/client-go/kubernetes"
)

type Agent struct {
	logger   logging.Logger
	kube     kubernetes.Interface
	platform platform.Platform

	state State

	// component is the shared context root for workers and goroutines.
	component context.Context
}

func (a *Agent) updateState(state *State) error {
	patchData, err := state.PatchJSON()
	if err != nil {
		return errors.WithMessage(err, "could not prepare state update")
	}

}
