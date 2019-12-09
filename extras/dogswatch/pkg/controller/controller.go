package controller

import (
	"context"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/nodestream"
	"github.com/amazonlinux/thar/dogswatch/pkg/workgroup"
	"k8s.io/client-go/kubernetes"
)

// Controller is the Dogswatch component that runs coordination for the Thar
// upgrade processes across many hosts, running the Dogswatch Agent, in a
// cluster.
type Controller struct {
	log     logging.Logger
	kube    kubernetes.Interface
	manager *actionManager
}

// New creates a Dogswatch Controller instance.
func New(log logging.Logger, kube kubernetes.Interface, nodeName string) (*Controller, error) {
	return &Controller{
		log:     log,
		kube:    kube,
		manager: newManager(log.WithField("worker", "manager"), kube, nodeName),
	}, nil
}

// Run executes the event loop for the Controller until signaled to exit.
func (c *Controller) Run(ctx context.Context) error {
	worker, cancel := context.WithCancel(ctx)
	defer cancel()

	c.log.Debug("starting workers")

	group := workgroup.WithContext(worker)

	// The nodestream will provide us with resource events that are scoped to
	// Nodes we "should" care about - those are labeled with markers.
	ns := nodestream.New(c.log.WithField("worker", "informer"), c.kube, nodestream.Config{}, c.manager)
	// Couple the informer's reflector in the manager for accessing the cached
	// cluster state.
	c.manager.SetStoreProvider(ns.GetInformer())

	group.Work(ns.Run)
	group.Work(c.manager.Run)

	c.log.Debug("running control loop")
	for {
		select {
		case <-ctx.Done():
			cancel()
			return nil
		}
	}
}
