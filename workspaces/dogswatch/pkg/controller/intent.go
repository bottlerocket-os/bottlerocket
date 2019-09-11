package controller

import "github.com/amazonlinux/thar/dogswatch/pkg/intent"

type Intent struct {
	*intent.Intent
	// ClusterActive is the number of nodes that are actively making progress in
	// the cluster.
	ClusterActive int
	// ClusterCount is the number of nodes that are operated in the cluster.
	ClusterCount int
}
