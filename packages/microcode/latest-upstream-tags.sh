#!/bin/sh

echo "Latest upstream tag for Intel ucode (microcode-ctl):"
git ls-remote --tags --refs https://github.com/intel/Intel-Linux-Processor-Microcode-Data-Files.git | tail -1

echo "Latest upstream tag for AMD ucode (linux-firmware):"
git ls-remote --tags --refs https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git | tail -1
