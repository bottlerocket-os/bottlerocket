package agent

import (
	"context"
	"errors"
	"sync"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/platform"
	"k8s.io/client-go/kubernetes"
)

type Agent struct {
	logger   logging.Logger
	kube     kubernetes.Interface
	platform platform.Platform

	state *nodeState

	// component is the shared context root for workers and goroutines.
	component context.Context

	//
	waitgroup *sync.WaitGroup
}

func New(logger logging.Logger, kube kubernetes.Interface, plat platform.Platform) *Agent {
	return &Agent{
		logger:   logger,
		kube:     kube,
		platform: plat,
		state:    initialState(),
	}
}

func (a *Agent) Run(ctx context.Context) error {
	return errors.New("unimplemented")
}
