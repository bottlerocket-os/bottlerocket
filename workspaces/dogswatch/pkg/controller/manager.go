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
		input:  make(chan *Intent),
	}
}

func (am *ActionManager) Run(ctx context.Context) error {
	am.log.Debug("starting")
	defer am.log.Debug("finished")

	actives := map[string]struct{}{}
	permit := make(chan *Intent)

	// TODO: ensure permit isn't left to block this, without the separated
	// selects there could be a race condition from golang's runtime selection
	// of the channel. So this becomes a busier loop than is needed :(
	for {
		// Handle active intents
		select {
		case <-ctx.Done():
			return
		case pin, ok := <-permit:
			// TODO: handle
		default:
			// carry on
		}

		// Check for new intents
		select {
		case <-ctx.Done():
			return

		case in, ok := <-am.input:
			log.Debug("checking with policy")
			proceed, err := am.policy.Check(&PolicyCheck{Intent: in, ClusterActive(len(actives))})
			if err != nil {
				log.WithError(err).Error("policy check errored")
				continue
			}
			if !proceed {
				log.Debug("policy denied intent")
				return nil
			}
			log.Debug("policy permitted intent")
		default:
			// carry on
		}
		return nil
	}
}

func (am *ActionManager) handle(node *v1.Node) {
	log := am.log.WithField("node", node.GetName())
	log.Debug("handling node event")

	in := am.intentFor(node)
	if in == nil {
		return // no actionable intent signaled
	}

	select {
	case am.input <- in:
		log.Debug("submitted")
	default:
		log.Warn("manager is unable to handle intent at this time")
	}
}

func (am *ActionManager) intentFor(node *v1.Node) *intent.Intent {
	log := am.log.WithField("node", node.GetName())
	in := intent.Given(node).Next()

	if in.Pending() {
		log.Debug("needs action")
		return in
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
