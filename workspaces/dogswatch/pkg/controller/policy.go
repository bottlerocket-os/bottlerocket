package controller

const (
	allowedClusterActive = 1
)

type Policy interface {
	// Check determines if the policy permits continuing with an intended
	// action.
	Check(*actionIntent) (bool, error)
}

type defaultPolicy struct{}

func (p *defaultPolicy) Check(intent *actionIntent) (bool, error) {
	// If already active, continue to handle it.
	if intent.IsActive() {
		return true, nil
	}
	// If there are no other active nodes in the cluster, then go ahead with the
	// intended action.
	if intent.ClusterActive == 0 {
		return true, nil
	}
	return false, nil
}
