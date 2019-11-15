package agent

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/k8sutil"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/amazonlinux/thar/dogswatch/pkg/nodestream"
	"github.com/amazonlinux/thar/dogswatch/pkg/platform"
	"github.com/amazonlinux/thar/dogswatch/pkg/workgroup"
	"github.com/pkg/errors"
	v1 "k8s.io/api/core/v1"
	v1meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	corev1 "k8s.io/client-go/kubernetes/typed/core/v1"
)

const (
	initialPollDelay   = time.Minute * 1
	updatePollInterval = time.Minute * 30
)

var (
	errInvalidProgress = errors.New("intended to make invalid progress")
)

type Agent struct {
	log      logging.Logger
	kube     kubernetes.Interface
	platform platform.Platform
	nodeName string

	poster poster
	proc   proc

	progress progression
}

type poster interface {
	Post(*intent.Intent) error
}

type proc interface {
	KillProcess() error
}

func New(log logging.Logger, kube kubernetes.Interface, plat platform.Platform, nodeName string) (*Agent, error) {
	if nodeName == "" {
		return nil, errors.New("nodeName must be provided for Agent to manage")
	}
	var nodeclient corev1.NodeInterface
	if kube != nil {
		nodeclient = kube.CoreV1().Nodes()
	}
	return &Agent{
		log:      log,
		kube:     kube,
		platform: plat,
		poster:   &k8sPoster{log, nodeclient},
		proc:     &osProc{},
		nodeName: nodeName,
	}, nil
}

func (a *Agent) checkProviders() error {
	switch {
	case a.kube == nil:
		return errors.New("kubernetes client is nil")
	case a.platform == nil:
		return errors.New("supporting platform is nil")
	}
	return nil
}

// TODO: add regular update checks

func (a *Agent) Run(ctx context.Context) error {
	if err := a.checkProviders(); err != nil {
		return errors.WithMessage(err, "misconfigured")
	}
	a.log.Debug("starting")
	defer a.log.Debug("finished")
	group := workgroup.WithContext(ctx)

	ns := nodestream.New(a.log.WithField("worker", "informer"), a.kube, nodestream.Config{
		NodeName: a.nodeName,
	}, a.handler())

	group.Work(ns.Run)
	group.Work(a.periodicUpdateChecker)

	err := a.checkNodePreflight()
	if err != nil {
		return err
	}

	select {
	case <-ctx.Done():
		a.log.Info("waiting on workers to finish")
		return group.Wait()
	}
}

func (a *Agent) periodicUpdateChecker(ctx context.Context) error {
	timer := time.NewTimer(initialPollDelay)
	defer timer.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-timer.C:
			// TODO: update this when we have richer data plumbed
			_, err := a.platform.ListAvailable()
			avail := err == nil
			a.setUpdateAvailable(avail)
		}
		timer.Reset(updatePollInterval)
	}
}

func (a *Agent) setUpdateAvailable(available bool) error {
	// TODO: handle brief race condition internally - this needs to be improved,
	// though the kubernetes control plane will reject out of order updates by
	// way of resource versioning C-A-S operations.
	node, err := a.kube.CoreV1().Nodes().Get(a.nodeName, v1meta.GetOptions{})
	if err != nil {
		return errors.WithMessage(err, "unable to get node")
	}
	in := intent.Given(node)
	switch available {
	case true:
		in.UpdateAvailable = marker.NodeUpdateAvailable
	case false:
		in.UpdateAvailable = marker.NodeUpdateUnavailable
	}
	return a.poster.Post(in)
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
	if activeIntent(in) {
		a.log.Debug("active intent received")
		if err := a.realize(in); err != nil {
			a.log.WithError(err).Error("could not handle intent")
		}
		return
	}
	a.log.Debug("inactive intent received")
}

func activeIntent(i *intent.Intent) bool {
	wanted := i.InProgress() && !i.DegradedPath()
	empty := i.Wanted == "" || i.Active == "" || i.State == ""
	unknown := i.Wanted == marker.NodeActionUnknown
	return wanted && !empty && !unknown
}

func (a *Agent) realize(in *intent.Intent) error {
	a.log.WithField("intent", fmt.Sprintf("%#v", in)).Debug("realizing intent")

	var err error

	// TODO: Run a quick check of the Nodes posted progress before proceeding

	// ACK the wanted action.
	in.Active = in.Wanted
	in.State = marker.NodeStateBusy
	if err = a.poster.Post(in); err != nil {
		return err
	}

	// TODO: Propagate status from realization and periodically
	switch in.Wanted {
	case marker.NodeActionReset:
		a.progress.Reset()

	case marker.NodeActionPrepareUpdate:
		var ups platform.Available
		ups, err = a.platform.ListAvailable()
		if err != nil {
			break
		}
		if len(ups.Updates()) == 0 {
			err = errInvalidProgress
			break
		}
		a.progress.SetTarget(ups.Updates()[0])
		a.log.Debug("preparing update")
		err = a.platform.Prepare(a.progress.GetTarget())

	case marker.NodeActionPerformUpdate:
		if !a.progress.Valid() {
			err = errInvalidProgress
			break
		}
		a.log.Debug("updating")
		err = a.platform.Update(a.progress.GetTarget())

	case marker.NodeActionUnknown, marker.NodeActionStabilize:
		a.log.Debug("sitrep")
		_, err = a.platform.Status()

	case marker.NodeActionRebootUpdate:
		if !a.progress.Valid() {
			err = errInvalidProgress
			break
		}
		a.log.Debug("rebooting")
		a.log.Info("Rebooting Node to complete update")
		// TODO: ensure Node is setup to be validated on boot (ie: kubelet will
		// run agent again before we let other Pods get scheduled)
		err = a.platform.BootUpdate(a.progress.GetTarget(), true)
		// Shortcircuit to terminate.

		// TODO: actually handle shutdown.
		// die("goodbye");
		if err == nil {
			if a.proc != nil {
				defer a.proc.KillProcess()
			}
			return err
		}
	}

	if err != nil {
		a.log.WithError(err).Error("could not realize intent")
		in.State = marker.NodeStateError
	} else {
		a.log.Debug("realized intent")
		in.State = marker.NodeStateReady
	}

	a.poster.Post(in)

	return err
}

func (a *Agent) checkNodePreflight() error {
	// TODO: Run a check of the Node Resource and reset appropriately

	// TODO: Inform controller for taint removal

	n, err := a.kube.CoreV1().Nodes().Get(a.nodeName, v1meta.GetOptions{})
	if err != nil {
		return errors.WithMessage(err, "unable to retrieve Node for preflight check")
	}

	// Update our state to be "ready" for action, this shouldn't actually do so
	// unless its really done.
	in := intent.Given(n)
	// TODO: check that we're properly reseting, for now its not needed to mark
	// our work "done"
	switch {
	case in.Terminal(): // we're at a terminating point where there's no progress to make.
		in.State = marker.NodeStateReady
	case in.Waiting():
		// already in a holding pattern, no need to re-prime ourselves in
		// preflight.
	case in.Wanted == "" || in.Active == "":
		in = in.Reset()
	default:
		// there's not a good way to re-prime ourselves in the prior state.
		in = in.Reset()
	}
	a.poster.Post(in)

	return nil
}

type osProc struct{}

func (*osProc) KillProcess() error {
	p, _ := os.FindProcess(os.Getpid())
	go p.Kill()
	return nil
}

type k8sPoster struct {
	log        logging.Logger
	nodeclient corev1.NodeInterface
}

func (k *k8sPoster) Post(i *intent.Intent) error {
	nodeName := i.GetName()
	defer k.log.WithField("node", nodeName).Debugf("posted intent %s", i.DisplayString())
	return k8sutil.PostMetadata(k.nodeclient, nodeName, i)
}
