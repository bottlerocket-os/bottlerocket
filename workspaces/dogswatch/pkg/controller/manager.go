package controller

import (
	"errors"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/sirupsen/logrus"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/kubernetes"
)

// ActionManager handles node changes according to policy and runs a node update
// flow to completion as allowed by policy.
type ActionManager struct {
	log    logging.Logger
	kube   kubernetes.Interface
	policy Policy
}

func newManager(log logging.Logger, kube kubernetes.Interface) *ActionManager {
	return &ActionManager{
		log:    log,
		kube:   kube,
		policy: &defaultPolicy{},
	}
}

func (am *ActionManager) HandleNode(node *v1.Node) error {
	log := am.log.WithField("node", node.GetName())
	log.Debug("handling event")
	in := am.intentFor(node)

	if in == nil {
		log.Debug("no actionable intent")
		return nil
	}

	proceed, err := am.policy.Check(in)
	if err != nil {
		log.WithError(err).Error("policy check errored")
		return err
	}
	if !proceed {
		log.Debug("cannot proceed with intent")
		return nil
	}

	// TODO: write progress manager
	return errors.New("unimplemented")
}

func (am *ActionManager) intentFor(node *v1.Node) *Intent {
	log := am.log.WithField("node", node.GetName())

	in := &Intent{
		Intent:        intent.Given(node).Next(),
		ClusterActive: 0,
		ClusterCount:  0,
	}

	// Tack on debug info if its available, otherwise don't.
	if log.Logger.IsLevelEnabled(logrus.DebugLevel) {
		log = log.WithField("intent", in)
	}

	if in.Pending() {
		log.Debug("needs action")
		return in
	}
	log.Debug("no action needed")
	return nil
}
