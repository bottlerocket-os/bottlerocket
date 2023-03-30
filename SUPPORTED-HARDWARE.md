# Supported hardware for Bottlerocket on bare metal

The Bottlerocket bare metal variant is intended to run Bottlerocket on targets outside of AWS or vmware clusters.
The vast diversity of available hardware poses a challenge.
The need to be compatible with as much hardware as possible out-of-the-box conflicts with Bottlerocket's core principles of keeping it small and simple.
To strike the right balance, the initial offering focuses on compatibility with common x86_64 server hardware.
The Bottlerocket kernel for metal is configured to include drivers for a wider spread of 10G+ Ethernet NICs in their base configuration (no model-specific FPGA offloading support and similar) as well as common RAID controllers.

Beyond that, the number of drivers included in the Bottlerocket kernels has been reduced substantially comparing to common general purpose Linux distributions.
The aim is to keep Bottlerocket images as lean as possible, while trying to maintain a good out-of-the-box coverage.

It is possible that Bottlerocket is missing drivers for your specific hardware.
Please [submit an issue](https://github.com/bottlerocket-os/bottlerocket/issues/new?assignees=&labels=&template=metal_driver.md) to open a discussion on inclusion of additional drivers.

## Limitations of hardware support to be added

Adding drivers that are part of the upstream Linux source tree is an easy fix for certain target platforms.
However, there are limitations of what to add to the Bottlerocket metal variant in order to accommodate Bottlerocket's core principles of keeping it small and simple.
If you want to create a custom variant that for example contains specific drivers, the current work towards out-of-tree builds will help you achieve that.
Work for that is currently underway and can be tracked in [issue #2669](https://github.com/bottlerocket-os/bottlerocket/issues/2669).
Until out-of-tree builds land the following limitations apply to the available Bottlerocket variants:

* There is no plan to add out-of-tree drivers to Bottlerocket images.
* There is no plan to add additional CPU architectures.
* There is no plan to add drivers for embedded devices in the core images.

If you have questions about these limitations or want to debate them, feel free to open an issue or start a discussion.

## Testing

The AWS Bottlerocket team does basic functional testing on a limited set of server configurations they have available (See [Hardware configurations confirmed to work](#hardware-configurations-confirmed-to-work)).
"Functional testing" means that machines are provisioned and base functionality of storage and network hardware is proven by a properly functioning distribution.

With the vast diversity of hardware available community involvement in confirming hardware configurations work is key.
We are interested to learn about your success stories running Bottlerocket on other hardware platforms.
Feel free to report a working configuration below by opening a PR with the information.

### Hardware configurations confirmed to work

Bottlerocket is tested on and known to work with the hardware below.

| Server model | CPU | BIOS/UEFI | Network Card | Disk | RAID/Storage controller | Entity confirming |
| --- | --- | --- | --- | --- | --- | --- |
| Supermicro SYS-E200-8D | Intel Xeon D-1528 | BIOS & UEFI | Intel i350 1G & 10G | SATA SSD, NVME | N/A | AWS Bottlerocket team |
| Dell R240 | Intel Xeon E2236 | BIOS & UEFI | Broadcom BCM5720 1G | SATA SSD (RAID0) | PERC H730P | AWS Bottlerocket team |
| Dell R620 | Intel Xeon E5-2660 | BIOS | Intel i350 1G | SATA HDD | PERC H710P | AWS Bottlerocket team |
| HP DL20 | Intel Xeon E2234 | BIOS | HPE 361i 1G | SATA SSD | HPE Smart Array S100i | AWS Bottlerocket team |

