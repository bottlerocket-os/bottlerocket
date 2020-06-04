package main

import (
	// We need to apply a containerd patch (c134a9b), and the best time to
	// do that is during the build, after we've cached the Go modules we
	// need. However, the patch adds an import of the SELinux module, so
	// we also need to cache it, or the build will fail. The unused import
	// lets us cache the module we know we'll need.

	// TODO: drop this when we move to containerd v1.4.0
	_ "github.com/opencontainers/selinux/go-selinux/label"
)
