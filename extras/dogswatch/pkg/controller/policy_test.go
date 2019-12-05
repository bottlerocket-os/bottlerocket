package controller

import (
	"fmt"
	"testing"

	"github.com/amazonlinux/thar/dogswatch/pkg/internal/intents"
	"github.com/amazonlinux/thar/dogswatch/pkg/internal/testoutput"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"

	"gotest.tools/assert"
)

func TestPolicyCheck(t *testing.T) {
	cases := []struct {
		Name         string
		PolicyCheck  *PolicyCheck
		ShouldPermit bool
		ShouldError  bool
	}{
		// should not update when threshold would be exceeded
		{
			Name:         "update-available-maxactive",
			ShouldPermit: false,
			PolicyCheck: &PolicyCheck{
				Intent:        intents.Stabilized(intents.WithUpdateAvailable(marker.NodeUpdateUnavailable)),
				ClusterActive: maxClusterActive,
				ClusterCount:  maxClusterActive + 1,
			},
		},
		// stabilize should always be permitted
		{
			Name:         "stabilize-new",
			ShouldPermit: true,
			PolicyCheck: &PolicyCheck{
				Intent:        intents.PendingStabilizing(),
				ClusterActive: maxClusterActive,
				ClusterCount:  maxClusterActive + 1,
			},
		},
		{
			Name:         "stabilize-new",
			ShouldPermit: true,
			PolicyCheck: &PolicyCheck{
				Intent:        intents.PendingStabilizing(),
				ClusterActive: 0,
				ClusterCount:  maxClusterActive + 1,
			},
		},

		{
			Name:         "perform-maxactive",
			ShouldPermit: false,
			PolicyCheck: &PolicyCheck{
				Intent:        intents.PendingPrepareUpdate(),
				ClusterActive: maxClusterActive,
				ClusterCount:  maxClusterActive + 1,
			},
		},
	}

	for _, tc := range cases {

		check := tc.PolicyCheck
		t.Run(fmt.Sprintf("%s(%s) %d/%d", tc.Name, check.Intent.DisplayString(), check.ClusterActive, check.ClusterCount),
			func(t *testing.T) {
				policy := defaultPolicy{
					log: testoutput.Logger(t, logging.New("policy-check")),
				}

				permit, err := policy.Check(check)
				assert.Equal(t, tc.ShouldPermit, permit)
				if tc.ShouldError {
					assert.Error(t, err, "")
				} else {
					assert.NilError(t, err)
				}
			})
	}
}
