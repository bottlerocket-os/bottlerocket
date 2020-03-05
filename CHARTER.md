# Bottlerocket Charter

## Tenets (unless you know better ones)

These tenets guide Bottlerocket's development.
They let you know what we value and what we're working toward, even if not every feature is ready yet.

### Secure

Bottlerocket is **secure** so it can become a quiet piece of a platform you trust.
It uses a variety of mechanisms to provide defense-in-depth, and enables automatic updates by default.
It protects itself from persistent threats.
It enables kernel features that allow users to assert their own policies for locking down workloads.

### Open

Bottlerocket is **open** because the best OS can only be built through collaboration.
It is developed in full view of the world using open source tools and public infrastructure services.
It is not a Kubernetes distro, nor an Amazon distro.
We obsess over shared components like the kernel, and work within the community to support new orchestrators and platforms.

### Small

Bottlerocket is **small** because a few big ideas scale better than many small ones.
It includes only the core set of components needed for development and for use at runtime.
Anything we ship, we must be prepared to fix, so our goal is to ship as little as possible while staying useful.

### Simple

Bottlerocket is **simple** because simple lasts.
Users can pick the image they want, tweak a handful of settings, and then forget about it.
We favor settings that convey high-level intent over those that provide low-level control over specific details, because it is easier to preserve intent across months and years of automatic updates.
