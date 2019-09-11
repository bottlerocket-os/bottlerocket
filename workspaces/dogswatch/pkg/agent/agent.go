package agent

import (
	"context"
	"fmt"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/amazonlinux/thar/dogswatch/pkg/nodestream"
	"github.com/amazonlinux/thar/dogswatch/pkg/platform"
	"github.com/amazonlinux/thar/dogswatch/pkg/workgroup"
	"github.com/pkg/errors"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/kubernetes"
)

type Agent struct {
	log      logging.Logger
	kube     kubernetes.Interface
	platform platform.Platform

	state    *nodeState
	progress progression
}

func New(log logging.Logger, kube kubernetes.Interface, plat platform.Platform) *Agent {
	return &Agent{
		log:      log,
		kube:     kube,
		platform: plat,
		state:    initialState(),
	}
}

func (a *Agent) check() error {
	switch {
	case a.kube == nil:
		return errors.New("kubernetes client is nil")
	case a.platform == nil:
		return errors.New("supporting platform is nil")
	}
	return nil
}

func (a *Agent) Run(ctx context.Context) error {
	if err := a.check(); err != nil {
		return errors.WithMessage(err, "misconfigured")
	}
	a.log.Debug("starting")
	defer a.log.Debug("finished")
	group := workgroup.WithContext(ctx)

	ns := nodestream.New(a.log.WithField("worker", "informer"), a.kube, nodestream.Config{
		NodeName: "minikube",
	}, a.handler())

	group.Work(ns.Run)

	select {
	case <-ctx.Done():
		a.log.Info("waiting on workers to finish")
		return group.Wait()
	}
}

func (a *Agent) handler() nodestream.Handler {
	return &nodestream.HandlerFuncs{
		OnAddFunc: a.handleEvent,
		// we don't mind the diff between old and new, so handle the new
		// resource.
		OnUpdateFunc: func(_, n *v1.Node) {
			a.handleEvent(n)
		},
		OnDeleteFunc: func(_ *v1.Node) {
			panic("we were deleted, panic. everyone panic. 😱")
		},
	}
}

func (a *Agent) handleEvent(node *v1.Node) {
	in := intent.Given(node)
	if in.Active() {
		a.log.Debug("active intent received")
		a.realize(in)
		return
	}
	a.log.Debug("inactive intent received")
}

func (a *Agent) realize(in *intent.Intent) {
	a.log.WithField("intent", fmt.Sprintf("%#v", in)).Debug("realizing intent")
	var err error

	// TODO: Sanity check progression before proceeding

	// TODO: Propagate status from realization and periodically
	switch in.Action {
	case marker.NodeActionReset:
		a.progress.Reset()
		return
	case marker.NodeActionPrepareUpdate:
		var ups platform.Available
		ups, err = a.platform.ListAvailable()
		if err != nil {
			break
		}
		if len(ups.Updates()) == 0 {
			a.log.Warn("no update to make progress on")
			break
		}
		a.progress.SetTarget(ups.Updates()[0])
		a.log.Debug("preparing update")
		err = a.platform.Prepare(a.progress.GetTarget())
	case marker.NodeActionPerformUpdate:
		if !a.progress.Valid() {
			a.log.Warn("cannot realize intent with invalid progress")
			break
		}
		a.log.Debug("updating")
		err = a.platform.Update(a.progress.GetTarget())
	case marker.NodeActionUnknown, marker.NodeActionStablize:
		if !a.progress.Valid() {
			a.log.Warn("cannot realize intent with invalid progress")
			break
		}
		a.log.Debug("sitrep")
		_, err = a.platform.Status()
	case marker.NodeActionRebootUpdate:
		if !a.progress.Valid() {
			a.log.Warn("cannot realize intent with invalid progress")
			break
		}
		a.log.Debug("rebooting")
		a.log.Info("Rebooting Node to complete update")
		// TODO: ensure Node is setup to be validated on boot (ie: kubelet will
		// run agent again before we let other Pods get scheduled)
		err = a.platform.BootUpdate(a.progress.GetTarget(), true)
	}

	if err != nil {
		a.log.WithError(err).Error("failed to realize intent")
	} else {
		a.log.Debug("action taken to realize intent")
	}
}
