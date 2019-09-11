package controller

const (
	allowedClusterActive = 1
)

type Policy interface {
	// Check determines if the policy permits continuing with an intended
	// action.
	Check(*Intent) (bool, error)
}

type defaultPolicy struct{}

func (p *defaultPolicy) Check(in *Intent) (bool, error) {
	// If already active, continue to handle it.
	if in.Active() {
		return true, nil
	}
	// If there are no other active nodes in the cluster, then go ahead with the
	// intended action.
	if in.ClusterActive == 0 {
		return true, nil
	}
	return false, nil
}
