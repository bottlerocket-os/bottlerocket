# Supported hardware for Bottlerocket on bare metal

This document captures the hardware Bottlerocket for bare metal is tested on and known to work with.
It also contains hardware that contributors have reported to be functional.

Bottlerocket's kernel configuration isn't as extensive as some other distros, so it's possible that it is missing drivers for your specific hardware.
If so, please [submit an issue](https://github.com/bottlerocket-os/bottlerocket/issues/new?assignees=&labels=&template=feature.md) and we'll work on integrating the proper kernel modules for your hardware!

## Confirmed support

Bottlerocket is tested on and known to work with the hardware below.

| Server model | CPU | BIOS/UEFI | Network Card | Disk | RAID/Storage controller |
| --- | --- | --- | --- | --- | --- |
| Supermicro SYS-E200-8D | Intel Xeon D-1528 | BIOS & UEFI | Intel i350 1G & 10G | SATA SSD, NVME | N/A |
| Dell R240 | Intel Xeon E2236 | BIOS & UEFI | Broadcom BCM5720 1G | SATA SSD (RAID0) | PERC H730P |
| Dell R620 | Intel Xeon E5-2660 | BIOS | Intel i350 1G | SATA HDD | PERC H710P |
| HP DL20 | Intel Xeon E2234 | BIOS | HPE 361i 1G | SATA SSD | HPE Smart Array S100i |

## Reported Support

Bottlerocket has been reported to work on the hardware below.

| Server model | CPU | BIOS/UEFI | Network Card | Disk | RAID/Storage controller |
| --- | --- | --- | --- | --- | --- |
