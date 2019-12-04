package controller

import (
	"fmt"

	"github.com/amazonlinux/thar/dogswatch/pkg/intent"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/tools/cache"
)

const (
	allowedClusterActive = 1
)

type Policy interface {
	// Check determines if the policy permits continuing with an intended
	// action.
	Check(*PolicyCheck) (bool, error)
}

type defaultPolicy struct{}

func (p *defaultPolicy) Check(ck *PolicyCheck) (bool, error) {
	// If already active, continue to handle it.
	if ck.Intent.InProgress() {
		return true, nil
	}
	// If there are no other active nodes in the cluster, then go ahead with the
	// intended action.
	if ck.ClusterActive == 0 {
		return true, nil
	}
	return false, nil
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
		if !cin.Terminal() {
			clusterActive++
		}
	}

	if logging.Debuggable {
		logging.New("policy-check").WithFields(logrus.Fields{
			"intent":         in.DisplayString(),
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
