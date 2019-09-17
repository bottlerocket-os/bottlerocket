package controller

import "github.com/amazonlinux/thar/dogswatch/pkg/intent"

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
