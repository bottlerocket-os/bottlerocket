#!/bin/sh

echo "Latest upstream tag for linux-firmware:"
git ls-remote --tags --refs https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git | tail -1
