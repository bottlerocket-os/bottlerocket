# kernel-5.15

This package contains the Bottlerocket Linux kernel of the 5.15 series.


## Testing of Configuration Changes

Bottlerocket kernels are built in multiple flavors (e.g. cloud, bare metal) and for multiple architectures (e.g. aarch64, x86_64).
The kernel configuration for any of those combinations might change independently of the others.
Please use `tools/diff-kernel-config` from the main Bottlerocket repository to ensure the configuration for any of the combinations does not change inadvertently.
Changes that can have an effect on the resulting kernel configuration include:

* explicit kernel configuration changes
* package updates/kernel rebases

Reviewers on a pull request potentially changing the kernel configuration will appreciate having the report produced by `diff-kernel-config` included in the PR description.
