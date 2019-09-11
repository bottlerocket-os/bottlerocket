package controller

import (
	"errors"

	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
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
	intent := am.intentFor(node)

	if intent == nil {
		log.Debug("no actionable intent")
		return nil
	}

	proceed, err := am.policy.Check(intent)
	if err != nil {
		log.WithError(err).Error("policy check errored")
		return err
	}
	if !proceed {
		log.Debug("cannot proceed with intent")
		return nil
	}

	return errors.New("unimplemented")
}

func (am *ActionManager) intentFor(node *v1.Node) *actionIntent {
	log := am.log.WithField("node", node.GetName())
	// TODO: get the next states for intent.
	nextAction := ""
	nextState := ""

	annos := node.GetAnnotations()

	intent := &actionIntent{
		ID: node.GetName(),

		CurrentState:  annos[marker.NodeStateKey],
		CurrentAction: annos[marker.NodeActionKey],

		IntentState:  nextState,
		IntentAction: nextAction,

		ClusterActive: 0,
	}

	// Tack on debug info if its available, otherwise don't.
	if log.Logger.IsLevelEnabled(logrus.DebugLevel) {
		log = log.WithField("intent", intent)
	}

	if !intent.IsNeeded() {
		log.Debug("intent determined to be unneeded")
		return nil
	}
	log.Debug("intent needs action")
	return intent
}

type actionIntent struct {
	// ID is an identifier that uniquely addresses the intent target.
	ID string

	// CurrentAction is the currently instructed action on the node.
	CurrentAction marker.NodeAction
	// CurrentState is the current state of the node.
	CurrentState marker.NodeState

	// IntentAction is the intended next action to be instructed.
	IntentAction marker.NodeAction
	// IntentState is the state that would be reached if the intent were
	// progressed upon.
	IntentState marker.NodeState

	// ClusterActive is the number of nodes that are actively making progress in
	// the cluster.
	ClusterActive int

	// ClusterCount is the number of nodes that are operated in the cluster.
	ClusterCount int
}

func (i *actionIntent) IsActive() bool {
	switch i.CurrentAction {
	case marker.NodeActionReset,
		marker.NodeActionPrepareUpdate,
		marker.NodeActionPerformUpdate,
		marker.NodeActionRebootUpdate:
		return true
	}

	switch i.CurrentState {
	case marker.NodeStateRebooting:
		return true
	}

	return false
}

// IsNeeded indicates that the intent is needing progress made on it.
func (i *actionIntent) IsNeeded() bool {
	// If its active, then its a needed intent.
	if i.IsActive() {
		return true
	}
	// If the node has an update, the intent is needing progress.
	switch i.CurrentState {
	case marker.NodeStateUpdateAvailable:
		return true
	}
	// Otherwise, the node can sit tight.
	return false
}
