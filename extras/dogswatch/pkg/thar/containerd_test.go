package thar

import (
	"testing"
)

func TestContainerdSystemdPath(t *testing.T) {
	t.Log(containerdDropInDir)
	if containerdDropInDir != "/.thar/rootfs/run/systemd/system/containerd.service.d" {
		t.Fatal("should have matched")
	}
}
