// SPDX-License-Identifier: Apache-2.0

package main

import (
	"context"

	"github.com/containerd/containerd/containers"
	"github.com/containerd/containerd/oci"
	spec "github.com/opencontainers/runtime-spec/specs-go"
)

// withAllDevicesAllowed permits all access on all devices nodes for the container
// Taken from https://github.com/containerd/containerd/blob/25947db049b058fcbce291ef883b8b512e3ea440/oci/spec_opts.go#L1010
// which is not available in containerd v1.3.7
// See https://github.com/bottlerocket-os/bottlerocket/issues/1038
func withAllDevicesAllowed(_ context.Context, _ oci.Client, _ *containers.Container, s *spec.Spec) error {
	// This check-and-set is originally done by `setLinux` from https://github.com/containerd/containerd/blob/25947db049b058fcbce291ef883b8b512e3ea440/oci/spec_opts.go#L74
	if s.Linux == nil {
		s.Linux = &spec.Linux{}
	}

	if s.Linux.Resources == nil {
		s.Linux.Resources = &spec.LinuxResources{}
	}
	s.Linux.Resources.Devices = []spec.LinuxDeviceCgroup{
		{
			Allow:  true,
			Access: "rwm",
		},
	}
	return nil
}
