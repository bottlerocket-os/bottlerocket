package controller

import (
	"context"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/nodestream"
	"github.com/pkg/errors"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/kubernetes"
	corev1 "k8s.io/client-go/kubernetes/typed/core/v1"
	"k8s.io/client-go/tools/cache"
)

const maxQueuedIntents = 10

var _ nodestream.Handler = (*ActionManager)(nil)

// ActionManager handles node changes according to policy and runs a node update
// flow to completion as allowed by policy.
type ActionManager struct {
	log      logging.Logger
	kube     kubernetes.Interface
	policy   Policy
	input    chan *intent.Intent
	storer   storer
	poster   poster
	nodeName string
	nodem    nodeManager
}

// poster is the implementation of the intent poster that publishes the provided
// intent.
type poster interface {
	Post(*intent.Intent) error
}

// nodeManager is the implementation that interfaces the interactions with nodes
// to accomplish tasks.
type nodeManager interface {
	Cordon(string) error
	Uncordon(string) error
	Drain(string) error
}

type storer interface {
	GetStore() cache.Store
}

func newManager(log logging.Logger, kube kubernetes.Interface, nodeName string) *ActionManager {
	var nodeclient corev1.NodeInterface
	if kube != nil {
		nodeclient = kube.CoreV1().Nodes()
	}

	return &ActionManager{
		log:    log,
		kube:   kube,
		policy: &defaultPolicy{},
		input:  make(chan *intent.Intent, 1),
		poster: &k8sPoster{log, nodeclient},
		nodem:  &k8sNodeManager{kube},
	}
}

func (am *ActionManager) Run(ctx context.Context) error {
	am.log.Debug("starting")
	defer am.log.Debug("finished")

	permit := make(chan *intent.Intent, maxQueuedIntents)

	// TODO: split out accepted intent handler - it should handle its
	// prioritization as needed to ensure that active nodes' events reach it.

	for {
		// Handle active intents
		select {
		case <-ctx.Done():
			return nil

		case pin, ok := <-permit:
			if !ok {
				break
			}
			am.log.Debug("handling permitted event")
			am.takeAction(pin)

		case in, ok := <-am.input:
			if !ok {
				break
			}
			am.log.Debug("checking with policy")

			// TODO: make policy checking and consideration richer
			pview, err := am.makePolicyCheck(in)
			if err != nil {
				am.log.WithError(err).Error("policy unenforceable")
				continue
			}
			proceed, err := am.policy.Check(pview)
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

func (am *ActionManager) takeAction(pin *intent.Intent) error {
	log := am.log.WithField("node", pin.GetName())
	successCheckRun := successfulUpdate(pin)
	if successCheckRun {
		log.Debug("handling successful update")
	}

	if pin.Intrusive() && !successCheckRun {
		err := am.nodem.Cordon(pin.NodeName)
		if err != nil {
			log.WithError(err).Error("could not cordon")
			return err
		}
		err = am.nodem.Drain(pin.NodeName)
		if err != nil {
			log.WithError(err).Error("could not drain")
			// TODO: make workload check/ignore configurable
			log.Warn("proceeding anyway")
		}
	}

	// Handle successful node reconnection.
	if successCheckRun {
		// Reset the state to begin its stabilization.
		pin = pin.Reset()

		err := am.checkNode(pin.NodeName)
		if err != nil {
			log.WithError(err).Error("unable to perform success-check")
			// TODO: make success checks configurable
			log.Warn("proceeding anyway")
		}
		err = am.nodem.Uncordon(pin.NodeName)
		if err != nil {
			log.WithError(err).Error("could not uncordon")
			// TODO: make policy consider failed success handle scenarios,
			// otherwise we could make a starved cluster.
			log.Warn("workload will not return")
			return err
		}
	}

	err := am.poster.Post(pin)
	if err != nil {
		log.WithError(err).Error("could not post intent")
	}
	return err
}

func (am *ActionManager) makePolicyCheck(in *intent.Intent) (*PolicyCheck, error) {
	if am.storer == nil {
		return nil, errors.Errorf("manager has no store to access, needed for policy check")
	}
	return newPolicyCheck(in, am.storer.GetStore())
}

func (am *ActionManager) SetStoreProvider(storer storer) {
	am.storer = storer
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
		log.Debug("intent is stuck")
		log.Warn("resetting to stabilize stuck intent state")
		in = in.Reset()
		return in
	}
	// TODO: add per-node bucketed backoff for error handling and retries.
	if in.Errored() {
		log.Debug("intent errored")
		log.Warn("action errored on node, resetting to stabilize")
		in = in.Reset()
		return in.Projected()
	}
	next := in.Projected()
	if (in.Actionable() || next.Actionable()) && in.Realized() && !in.InProgress() {
		log.Debug("intent needs action")
		log.Debug("needs action towards next step")
		return next
	}
	if !in.Realized() {
		log.Debug("intent is not yet realized")
		return nil
	}

	if successfulUpdate(in) {
		return in
	}

	if in.HasUpdateAvailable() && in.Waiting() && !in.Errored() {
		log.Debug("intent starts update")
		return in.SetBeginUpdate()
	}

	log.Debug("no action needed")
	return nil
}

func successfulUpdate(in *intent.Intent) bool {
	atFinalTerm := intent.FallbackNodeAction != in.Wanted && !in.Stuck()
	return atFinalTerm && in.Waiting() && in.Terminal() && in.Realized()
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
