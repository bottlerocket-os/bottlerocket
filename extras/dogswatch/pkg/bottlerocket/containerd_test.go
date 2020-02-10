package bottlerocket

import (
	"testing"
)

func TestContainerdSystemdPath(t *testing.T) {
	t.Log(containerdDropInDir)
	if containerdDropInDir != "/.bottlerocket/rootfs/run/systemd/system/containerd.service.d" {
		t.Fatal("should have matched")
	}
}
