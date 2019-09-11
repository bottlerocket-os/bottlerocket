package controller

import (
	"context"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/workgroup"
	"k8s.io/client-go/kubernetes"
)

type Controller struct {
	log     logging.Logger
	kube    kubernetes.Interface
	manager *ActionManager
}

func New(log logging.Logger, kube kubernetes.Interface) (*Controller, error) {
	return &Controller{
		log:     log,
		kube:    kube,
		manager: newManager(log.WithField("worker", "manager"), kube),
	}, nil
}

func (c *Controller) Run(ctx context.Context) error {
	worker, cancel := context.WithCancel(ctx)
	defer cancel()

	c.log.Debug("starting workers")

	group := workgroup.WithContext(worker)

	group.Work(c.informer)
	group.Work(c.streamer)

	c.log.Debug("running control loop")
	for {
		select {
		case <-ctx.Done():
			cancel()
			return nil
		}
	}
}

func (c *Controller) informer(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return nil
	}
}

func (c *Controller) streamer(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return nil
	}
}
