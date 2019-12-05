package controller

import (
	"fmt"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/internal/logfields"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/tools/cache"
)

const (
	maxClusterActive = 1
)

type Policy interface {
	// Check determines if the policy permits continuing with an intended
	// action.
	Check(*PolicyCheck) (bool, error)
}

type PolicyCheck struct {
	Intent        *intent.Intent
	ClusterActive int
	ClusterCount  int
}

func newPolicyCheck(in *intent.Intent, resources cache.Store) (*PolicyCheck, error) {
	// TODO: use a workqueue (or other facility) to pull a stable consistent
	// view at each intent.
	ress := resources.List()
	clusterCount := len(ress)
	clusterActive := 0
	for _, res := range ress {
		node, ok := res.(*v1.Node)
		if !ok {
			clusterCount--
			continue
		}
		cin := intent.Given(node)
		if isClusterActive(cin) {
			clusterActive++
		}
	}

	if logging.Debuggable {
		logging.New("policy-check").WithFields(logfields.Intent(in)).WithFields(logrus.Fields{
			"cluster-count":  fmt.Sprintf("%d", clusterCount),
			"cluster-active": fmt.Sprintf("%d", clusterActive),
			"resource-count": fmt.Sprintf("%d", len(ress)),
		}).Debug("collected policy check")
	}

	if clusterCount <= 0 {
		return nil, errors.Errorf("%d resources listed of inappropriate type", len(ress))
	}

	return &PolicyCheck{
		Intent:        in,
		ClusterActive: clusterActive,
		ClusterCount:  clusterCount,
	}, nil
}

// isClusterActive matches intents that the cluster shouldn't run concurrently.
func isClusterActive(i *intent.Intent) bool {
	stabilizing := i.Wanted == marker.NodeActionStabilize
	return !stabilizing && !i.Stuck()
}

type defaultPolicy struct {
	log logging.Logger
}

func (p *defaultPolicy) Check(ck *PolicyCheck) (bool, error) {
	log := p.log.WithFields(logfields.Intent(ck.Intent))

	// policy checks are applied to intended actions, Intents that are next in
	// line to be executed. Projections are made without considering the policy
	// at time of the projection to the next state. So, we have to check when
	// the update process is starting up.
	startingUpdate := ck.Intent.Active == marker.NodeActionStabilize
	if !startingUpdate {
		if ck.Intent.InProgress() {
			if logging.Debuggable {
				log.Debug("permit already in progress")
			}
			return true, nil
		}

		if ck.Intent.Terminal() {
			if logging.Debuggable {
				log.Debug("permit terminal intent")
			}
			return true, nil
		}
	}

	// If there are no other active nodes in the cluster, then go ahead with the
	// intended action.
	if ck.ClusterActive < maxClusterActive {
		log.WithFields(logrus.Fields{
			"cluster-active": fmt.Sprintf("%d", ck.ClusterActive),
			"allowed-active": fmt.Sprintf("%d", maxClusterActive),
		}).Debugf("permit according to active threshold")

		return true, nil
	}

	return false, nil
}
