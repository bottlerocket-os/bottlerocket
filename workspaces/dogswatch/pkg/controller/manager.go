package controller

import (
	"context"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/nodestream"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/kubernetes"
)

var _ nodestream.Handler = (*ActionManager)(nil)

// ActionManager handles node changes according to policy and runs a node update
// flow to completion as allowed by policy.
type ActionManager struct {
	log    logging.Logger
	kube   kubernetes.Interface
	policy Policy
	input  chan *intent.Intent
}

func newManager(log logging.Logger, kube kubernetes.Interface) *ActionManager {
	return &ActionManager{
		log:    log,
		kube:   kube,
		policy: &defaultPolicy{},
		input:  make(chan *intent.Intent, 1),
	}
}

const maxQueuedIntents = 10

func (am *ActionManager) Run(ctx context.Context) error {
	am.log.Debug("starting")
	defer am.log.Debug("finished")

	actives := map[string]struct{}{}
	permit := make(chan *intent.Intent, maxQueuedIntents)

	// TODO: split out accepted intent handler - it should handle its
	// prioritization as needed to ensure that active nodes' events reach it.

	for {
		// Handle active intents
		select {
		case <-ctx.Done():
			return nil

		case pin, ok := <-permit:
			am.log.Debug("handling permitted event")
			if !ok {
				break
			}
			var err error
			if pin.Intrusive() {
				if err = am.cordonNode(pin.NodeName); err == nil {
					err = am.drainWorkload(pin.NodeName)
				}
				if err != nil {
					am.log.WithError(err).Error("could not drain the node")
				}
			}
			err = am.postIntent(pin)
			if err != nil {
				am.log.WithError(err).Error("could not post intent")
			}

		case in, ok := <-am.input:
			if !ok {
				break
			}
			am.log.Debug("checking with policy")
			// TODO: make policy checking and consideration richer
			proceed, err := am.policy.Check(&PolicyCheck{Intent: in, ClusterActive: len(actives)})
			if err != nil {
				am.log.WithError(err).Error("policy check errored")
				continue
			}
			if !proceed {
				am.log.Debug("policy denied intent")
				return nil
			}
			am.log.Debug("policy permitted intent")
			if len(permit) < maxQueuedIntents {
				permit <- in
			} else {
				// TODO: handle backpressure with scheduling
				am.log.Warn("backpressure blocking permitted intents")
			}
		}
	}
}

func (am *ActionManager) handle(node *v1.Node) {
	log := am.log.WithField("node", node.GetName())
	log.Debug("handling event")

	in := am.intentFor(node)
	if in == nil {
		return // no actionable intent signaled
	}

	select {
	case am.input <- in:
		log.Debug("submitted intent")
	default:
		log.Warn("unable to submit intent")
	}
}

// intentFor interprets the intention given the Node's annotations.
func (am *ActionManager) intentFor(node intent.Input) *intent.Intent {
	log := am.log.WithField("node", node.GetName())
	in := intent.Given(node)

	if in.Stuck() {
		log.Warn("resetting to stabilize stuck/invalid intent state")
		return in.Reset()
	}

	next := in.Projected()
	if in.HasUpdateAvailable() || next.Actionable() {
		log.Debug("needs action towards next step")
		return next
	}
	log.Debug("no action needed")
	return nil
}

// OnAdd is a Handler implementation for nodestream
func (am *ActionManager) OnAdd(node *v1.Node) {
	am.handle(node)
}

// OnDelete is a Handler implementation for nodestream
func (am *ActionManager) OnDelete(node *v1.Node) {
	am.handle(node)
}

// OnUpdate is a Handler implementation for nodestream
func (am *ActionManager) OnUpdate(_ *v1.Node, node *v1.Node) {
	am.handle(node)
}
