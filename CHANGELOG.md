# v1.5.0 (2021-12-17)

## Security Enhancements
* Add the ability to hotpatch log4j for CVE-2021-44228 in running containers ([#1872], [#1871], [#1869])

## OS Changes
* Enable configuration for OCI hooks in the container lifecycle ([#1868])
* Retry all failed requests to IMDS ([#1841])
* Enable node feature discovery for Kubernetes device plugins ([#1863])
* Add `apiclient get` subcommand for simple API retrieval ([#1836])
* Add support for CPU microcode updates ([#1827])
* Consistently support API prefix queries ([#1835])

## Build Changes
* Add support for custom image sizes ([#1826])
* Add support for unifying the OS and data partitions on a single disk ([#1870])

## Documentation Changes
* Fixed typo in the README ([#1847] thanks, PascalBourdier!)

[#1826]:https://github.com/bottlerocket-os/bottlerocket/pull/1826
[#1827]:https://github.com/bottlerocket-os/bottlerocket/pull/1827
[#1835]:https://github.com/bottlerocket-os/bottlerocket/pull/1835
[#1836]:https://github.com/bottlerocket-os/bottlerocket/pull/1836
[#1841]:https://github.com/bottlerocket-os/bottlerocket/pull/1841
[#1847]:https://github.com/bottlerocket-os/bottlerocket/pull/1847
[#1863]:https://github.com/bottlerocket-os/bottlerocket/pull/1863
[#1868]:https://github.com/bottlerocket-os/bottlerocket/pull/1868
[#1869]:https://github.com/bottlerocket-os/bottlerocket/pull/1869
[#1870]:https://github.com/bottlerocket-os/bottlerocket/pull/1870
[#1871]:https://github.com/bottlerocket-os/bottlerocket/pull/1871
[#1872]:https://github.com/bottlerocket-os/bottlerocket/pull/1872

# v1.4.2 (2021-12-02)

## Security Fixes

* Update default [admin](https://github.com/bottlerocket-os/bottlerocket-admin-container/releases/tag/v0.7.3) and [control](https://github.com/bottlerocket-os/bottlerocket-control-container/releases/tag/v0.5.3) host containers to address CVE-2021-43527 ([#1852])
* Update kernel-5.4 and kernel-5.10 to include recent security fixes. ([#1851])

## Build Changes

* Update containerd (to v1.5.8) and Docker (to v20.10.11) ([#1851])

[#1851]: https://github.com/bottlerocket-os/bottlerocket/pull/1851
[#1852]: https://github.com/bottlerocket-os/bottlerocket/pull/1852

# v1.4.1 (2021-11-18)

## Security Fixes

* Apply patches to docker and containerd for CVE-2021-41190 ([#1832], [#1833])

## Build Changes

* Update Bottlerocket SDK to 0.23.1 ([#1831])

[#1831]: https://github.com/bottlerocket-os/bottlerocket/pull/1831
[#1832]: https://github.com/bottlerocket-os/bottlerocket/pull/1832
[#1833]: https://github.com/bottlerocket-os/bottlerocket/pull/1833


# v1.4.0 (2021-11-12)

## OS Changes

* Add 'apiclient exec' for running commands in host containers ([#1802], [#1790])
* Improve boot performance ([#1809])
* Add support for wildcard container registry mirrors ([#1791], [#1818])
* Wait up to 300s for a DHCP lease at boot ([#1800])
* Retry if fetching the IMDS session token fails ([#1801])
* Add ECR account IDs for pulling host containers in GovCloud ([#1793])
* Filter sensitive API settings from `logdog` dump ([#1777])
* Fix kubelet standalone mode ([#1783])

## Build Changes

* Remove aws-k8s-1.17 variant ([#1807])
* Update Bottlerocket SDK to 0.23 ([#1779])
* Update third-party packages ([#1816])
* Update Rust dependencies ([#1810])
* Update Go dependencies of `host-ctr` ([#1775], [#1774])
* Prevent spurious rebuilds of the model package ([#1808])
* Add disk image files to TUF repo ([#1787])
* Vendor wicked service units ([#1798])
* Add CI check for Rust code formatting ([#1782])
* Allow overriding the AMI data file suffix ([#1784])

## Documentation Changes

* Update cargo-make commands to work with newest cargo-make ([#1797])

[#1774]: https://github.com/bottlerocket-os/bottlerocket/pull/1774
[#1775]: https://github.com/bottlerocket-os/bottlerocket/pull/1775
[#1777]: https://github.com/bottlerocket-os/bottlerocket/pull/1777
[#1779]: https://github.com/bottlerocket-os/bottlerocket/pull/1779
[#1782]: https://github.com/bottlerocket-os/bottlerocket/pull/1782
[#1783]: https://github.com/bottlerocket-os/bottlerocket/pull/1783
[#1784]: https://github.com/bottlerocket-os/bottlerocket/pull/1784
[#1787]: https://github.com/bottlerocket-os/bottlerocket/pull/1787
[#1790]: https://github.com/bottlerocket-os/bottlerocket/pull/1790
[#1791]: https://github.com/bottlerocket-os/bottlerocket/pull/1791
[#1793]: https://github.com/bottlerocket-os/bottlerocket/pull/1793
[#1797]: https://github.com/bottlerocket-os/bottlerocket/pull/1797
[#1798]: https://github.com/bottlerocket-os/bottlerocket/pull/1798
[#1800]: https://github.com/bottlerocket-os/bottlerocket/pull/1800
[#1801]: https://github.com/bottlerocket-os/bottlerocket/pull/1801
[#1802]: https://github.com/bottlerocket-os/bottlerocket/pull/1802
[#1807]: https://github.com/bottlerocket-os/bottlerocket/pull/1807
[#1808]: https://github.com/bottlerocket-os/bottlerocket/pull/1808
[#1809]: https://github.com/bottlerocket-os/bottlerocket/pull/1809
[#1810]: https://github.com/bottlerocket-os/bottlerocket/pull/1810
[#1816]: https://github.com/bottlerocket-os/bottlerocket/pull/1816
[#1818]: https://github.com/bottlerocket-os/bottlerocket/pull/1818

# v1.3.0 (2021-10-06)

## Deprecation Notice

The Kubernetes 1.17 variant, `aws-k8s-1.17`, will lose support in November, 2021.
Kubernetes 1.17 is no longer receiving support upstream.
We recommend replacing `aws-k8s-1.17` nodes with a later variant, preferably `aws-k8s-1.21` if your cluster supports it.
See [this issue](https://github.com/bottlerocket-os/bottlerocket/issues/1772) for more details.

## Security Fixes

* Apply patches to docker and containerd for CVE-2021-41089, CVE-2021-41091, CVE-2021-41092, and CVE-2021-41103 ([#1769])

## OS Changes

* Add MCS constraints to the SELinux policy ([#1733])
* Support IPv6 in kubelet and pluto ([#1710])
* Add region flag to aws-iam-authenticator command ([#1762])
* Restart modified host containers ([#1722])
* Add more detail to /etc/os-release ([#1749])
* Add an entry to `/etc/hosts` for the current hostname ([#1713], [#1746])
* Update default control container to v0.5.2 ([#1730])
* Fix various SELinux policy issues ([#1729])
* Update eni-max-pods with new instance types ([#1724], thanks @samjo-nyang!)
* Add cilium device filters to open-vm-tools ([#1718])
* Implement hybrid boot support for x86_64 ([#1701])
* Include `/var/log/kdump` in logdog tarballs ([#1695])
* Use runtime.slice and system.slice cgroup settings in k8s variants ([#1684], thanks @cyrus-mc!)

## Build Changes

* Update third-party packages ([#1701], [#1716], [#1732], [#1755], [#1763], [#1767])
* Update Rust dependencies ([#1707], [#1750], [#1751])
* Add wave definition for slow deployment ([#1734])
* Add 'infrasys' for creating TUF infra in AWS ([#1723])
* Make OVF file first in the OVA bundle ([#1719])
* Raise pubsys messages to 'warn' if AMI exists or repo doesn't ([#1708])
* Add constants crate ([#1709])
* Add release URLs to package definitions ([#1748])
* Add *.src.rpm to packages/.gitignore ([#1768])
* Archive old migrations ([#1699])

## Documentation Changes

* Mention static pods in the security guidance around API access ([#1766])
* Fix link to issue labels ([#1764], thanks @andrewhsu!)
* Fix broken link for TLS bootstrapping ([#1758])
* Update hash for v3 root.json ([#1757])
* Update example version to v1.2.0 in QUICKSTART-VMWARE ([#1741], thanks @yuvalk!)
* Clarify default kernel lockdown settings per variant ([#1704])

[#1684]: https://github.com/bottlerocket-os/bottlerocket/pull/1684
[#1695]: https://github.com/bottlerocket-os/bottlerocket/pull/1695
[#1699]: https://github.com/bottlerocket-os/bottlerocket/pull/1699
[#1701]: https://github.com/bottlerocket-os/bottlerocket/pull/1701
[#1701]: https://github.com/bottlerocket-os/bottlerocket/pull/1701
[#1704]: https://github.com/bottlerocket-os/bottlerocket/pull/1704
[#1707]: https://github.com/bottlerocket-os/bottlerocket/pull/1707
[#1708]: https://github.com/bottlerocket-os/bottlerocket/pull/1708
[#1709]: https://github.com/bottlerocket-os/bottlerocket/pull/1709
[#1710]: https://github.com/bottlerocket-os/bottlerocket/pull/1710
[#1713]: https://github.com/bottlerocket-os/bottlerocket/pull/1713
[#1716]: https://github.com/bottlerocket-os/bottlerocket/pull/1716
[#1718]: https://github.com/bottlerocket-os/bottlerocket/pull/1718
[#1719]: https://github.com/bottlerocket-os/bottlerocket/pull/1719
[#1722]: https://github.com/bottlerocket-os/bottlerocket/pull/1722
[#1723]: https://github.com/bottlerocket-os/bottlerocket/pull/1723
[#1724]: https://github.com/bottlerocket-os/bottlerocket/pull/1724
[#1729]: https://github.com/bottlerocket-os/bottlerocket/pull/1729
[#1730]: https://github.com/bottlerocket-os/bottlerocket/pull/1730
[#1732]: https://github.com/bottlerocket-os/bottlerocket/pull/1732
[#1733]: https://github.com/bottlerocket-os/bottlerocket/pull/1733
[#1734]: https://github.com/bottlerocket-os/bottlerocket/pull/1734
[#1741]: https://github.com/bottlerocket-os/bottlerocket/pull/1741
[#1746]: https://github.com/bottlerocket-os/bottlerocket/pull/1746
[#1748]: https://github.com/bottlerocket-os/bottlerocket/pull/1748
[#1749]: https://github.com/bottlerocket-os/bottlerocket/pull/1749
[#1750]: https://github.com/bottlerocket-os/bottlerocket/pull/1750
[#1751]: https://github.com/bottlerocket-os/bottlerocket/pull/1751
[#1755]: https://github.com/bottlerocket-os/bottlerocket/pull/1755
[#1757]: https://github.com/bottlerocket-os/bottlerocket/pull/1757
[#1758]: https://github.com/bottlerocket-os/bottlerocket/pull/1758
[#1762]: https://github.com/bottlerocket-os/bottlerocket/pull/1762
[#1763]: https://github.com/bottlerocket-os/bottlerocket/pull/1763
[#1764]: https://github.com/bottlerocket-os/bottlerocket/pull/1764
[#1766]: https://github.com/bottlerocket-os/bottlerocket/pull/1766
[#1767]: https://github.com/bottlerocket-os/bottlerocket/pull/1767
[#1768]: https://github.com/bottlerocket-os/bottlerocket/pull/1768
[#1769]: https://github.com/bottlerocket-os/bottlerocket/pull/1769

# v1.2.1 (2021-09-16)

## Security fixes

* Update Kubernetes for CVE-2021-25741 ([#1753])

[#1753]: https://github.com/bottlerocket-os/bottlerocket/pull/1753

# v1.2.0 (2021-08-06)

## OS Changes

* Add settings for kubelet topologyManagerPolicy and topologyManagerScope ([#1659])
* Add support for container image registry mirrors ([#1629])
* Add support for custom CA certificates ([#1654])
* Add a setting for configuring hostname ([#1664], [#1680], [#1693])
* Avoid wildcard for applying rp_filter to interfaces ([#1677])
* Update default admin container to v0.7.2 ([#1685])

## Build Changes

* Add support for zstd compressed kernel ([#1668], [#1689])
* Add support for uploading OVAs to VMware ([#1622])
* Update default built variant to aws-k8s-1.21 ([#1686])
* Remove aws-k8s-1.16 variant ([#1658])
* Move migrations from v1.1.5 to v1.2.0 ([#1682])
* Update third-party packages ([#1676])
* Update host-ctr dependencies ([#1669])
* Update Rust dependencies ([#1655], [#1683], [#1687])

## Documentation Changes

* Fix typo in README ([#1652], **thanks @faultymonk!**)

[#1622]: https://github.com/bottlerocket-os/bottlerocket/pull/1622
[#1629]: https://github.com/bottlerocket-os/bottlerocket/pull/1629
[#1652]: https://github.com/bottlerocket-os/bottlerocket/pull/1652
[#1654]: https://github.com/bottlerocket-os/bottlerocket/pull/1654
[#1655]: https://github.com/bottlerocket-os/bottlerocket/pull/1655
[#1658]: https://github.com/bottlerocket-os/bottlerocket/pull/1658
[#1659]: https://github.com/bottlerocket-os/bottlerocket/pull/1659
[#1664]: https://github.com/bottlerocket-os/bottlerocket/pull/1664
[#1668]: https://github.com/bottlerocket-os/bottlerocket/pull/1668
[#1669]: https://github.com/bottlerocket-os/bottlerocket/pull/1669
[#1676]: https://github.com/bottlerocket-os/bottlerocket/pull/1676
[#1677]: https://github.com/bottlerocket-os/bottlerocket/pull/1677
[#1680]: https://github.com/bottlerocket-os/bottlerocket/pull/1680
[#1682]: https://github.com/bottlerocket-os/bottlerocket/pull/1682
[#1683]: https://github.com/bottlerocket-os/bottlerocket/pull/1683
[#1685]: https://github.com/bottlerocket-os/bottlerocket/pull/1685
[#1686]: https://github.com/bottlerocket-os/bottlerocket/pull/1686
[#1687]: https://github.com/bottlerocket-os/bottlerocket/pull/1687
[#1689]: https://github.com/bottlerocket-os/bottlerocket/pull/1689
[#1693]: https://github.com/bottlerocket-os/bottlerocket/pull/1693

# v1.1.4 (2021-07-23)

## Security fixes

* Update containerd to 1.4.8 ([#1661])
* Update systemd to 247.8 ([#1662])
* Update 5.4 and 5.10 kernels ([#1665])
* Set permissions to root-only for /var/lib/systemd/random-seed ([#1656])

[#1656]: https://github.com/bottlerocket-os/bottlerocket/pull/1656
[#1661]: https://github.com/bottlerocket-os/bottlerocket/pull/1661
[#1662]: https://github.com/bottlerocket-os/bottlerocket/pull/1662
[#1665]: https://github.com/bottlerocket-os/bottlerocket/pull/1665

# v1.1.3 (2021-07-12)

Note: in the Bottlerocket v1.0.8 release, for the aws-k8s-1.20 and aws-k8s-1.21 variants, we set the default Kubernetes CPU manager policy to "static".
We heard from several users that this breaks usage of the Fluent Bit log processor.
In Bottlerocket v1.1.3, we've changed the default back to "none", but have added a setting so you can use the "static" policy if desired.
To do so, set `settings.kubernetes.cpu-manager-policy` to "static".
To do this in user data, for example, pass the following:

```toml
[settings.kubernetes]
cpu-manager-policy = "static"
```

## OS Changes

* Fix parsing of lists of values in domain name search field of DHCP option sets ([#1646], **thanks @hypnoce!**)
* Add setting for configuring Kubernetes CPU manager policy and reconcile policy  ([#1638])

## Build Changes

* Update SDK to 0.22.0 ([#1640])
* Store build artifacts per architecture ([#1630])

## Documentation Changes

* Update references to the ECS variant for GA release ([#1637])

[#1630]: https://github.com/bottlerocket-os/bottlerocket/pull/1630
[#1637]: https://github.com/bottlerocket-os/bottlerocket/pull/1637
[#1638]: https://github.com/bottlerocket-os/bottlerocket/pull/1638
[#1640]: https://github.com/bottlerocket-os/bottlerocket/pull/1640
[#1646]: https://github.com/bottlerocket-os/bottlerocket/pull/1646

# v1.1.2 (2021-06-25)

With this release, the aws-ecs-1 variant has graduated from preview status and is now generally available.
It's been updated to include Docker 20.10.
The new [Bottlerocket ECS Updater](https://github.com/bottlerocket-os/bottlerocket-ecs-updater/) is available to help provide automated updates.
:tada:

## OS Changes

* Add aws-k8s-1.21 variant with Kubernetes 1.21 support ([#1612])
* Add settings for configuring kubelet containerLogMaxFiles and containerLogMaxSize ([#1589]) (Thanks, @samjo-nyang!)
* Add settings for configuring kubelet systemReserved ([#1606])
* Add kdump support, enabled by default in VMware variants ([#1596])
* In host containers, allow mount propagations from privileged containers ([#1601])
* Mark ipv6 lease as optional for eth0 ([#1602])
* Add recommended device filters to open-vm-tools ([#1603])
* In host container definitions, default "enabled" and "superpowered" to false ([#1580])
* Allow pubsys refresh-repo to use default key path ([#1575])
* Update default host containers ([#1609])

## Build Changes

* Add grep package to all variants ([#1562])
* Update Rust dependencies ([#1623], [#1574])
* Update third-party packages ([#1619], [#1616], [#1625])
* In GitHub Actions, pin rust toolchain to match version in SDK ([#1621])
* Add imdsclient library for querying IMDS ([#1372], [#1598], [#1610])
* Remove reqwest proxy workaround in metricdog and updog ([#1592])
* Simplify conditional compilation in early-boot-config ([#1576])
* Only build shibaken for aws variants ([#1591])
* Silence tokio mut warning in thar-be-settings ([#1593])
* Refactor package and variant dependencies ([#1549])
* Add derive attributes at start of list in model-derive ([#1572])
* Limit threads during pubsys validate-repo ([#1564])

## Documentation Changes

* Document the deprecation of the aws-k8s-1.16 variant ([#1600])
* Update README for VMware and add a QUICKSTART-VMWARE ([#1559])
* Add ap-northeast-3 to supported region list ([#1566])
* Add details about the two default Bottlerocket volumes to README ([#1588])
* Document webpki-roots version in webpki-roots-shim ([#1565])

[#1372]: https://github.com/bottlerocket-os/bottlerocket/pull/1372
[#1549]: https://github.com/bottlerocket-os/bottlerocket/pull/1549
[#1559]: https://github.com/bottlerocket-os/bottlerocket/pull/1559
[#1562]: https://github.com/bottlerocket-os/bottlerocket/pull/1562
[#1564]: https://github.com/bottlerocket-os/bottlerocket/pull/1564
[#1565]: https://github.com/bottlerocket-os/bottlerocket/pull/1565
[#1566]: https://github.com/bottlerocket-os/bottlerocket/pull/1566
[#1572]: https://github.com/bottlerocket-os/bottlerocket/pull/1572
[#1574]: https://github.com/bottlerocket-os/bottlerocket/pull/1574
[#1575]: https://github.com/bottlerocket-os/bottlerocket/pull/1575
[#1576]: https://github.com/bottlerocket-os/bottlerocket/pull/1576
[#1580]: https://github.com/bottlerocket-os/bottlerocket/pull/1580
[#1588]: https://github.com/bottlerocket-os/bottlerocket/pull/1588
[#1589]: https://github.com/bottlerocket-os/bottlerocket/pull/1589
[#1591]: https://github.com/bottlerocket-os/bottlerocket/pull/1591
[#1592]: https://github.com/bottlerocket-os/bottlerocket/pull/1592
[#1593]: https://github.com/bottlerocket-os/bottlerocket/pull/1593
[#1596]: https://github.com/bottlerocket-os/bottlerocket/pull/1596
[#1598]: https://github.com/bottlerocket-os/bottlerocket/pull/1598
[#1600]: https://github.com/bottlerocket-os/bottlerocket/pull/1600
[#1601]: https://github.com/bottlerocket-os/bottlerocket/pull/1601
[#1602]: https://github.com/bottlerocket-os/bottlerocket/pull/1602
[#1603]: https://github.com/bottlerocket-os/bottlerocket/pull/1603
[#1606]: https://github.com/bottlerocket-os/bottlerocket/pull/1606
[#1609]: https://github.com/bottlerocket-os/bottlerocket/pull/1609
[#1610]: https://github.com/bottlerocket-os/bottlerocket/pull/1610
[#1612]: https://github.com/bottlerocket-os/bottlerocket/pull/1612
[#1616]: https://github.com/bottlerocket-os/bottlerocket/pull/1616
[#1619]: https://github.com/bottlerocket-os/bottlerocket/pull/1619
[#1621]: https://github.com/bottlerocket-os/bottlerocket/pull/1621
[#1623]: https://github.com/bottlerocket-os/bottlerocket/pull/1623
[#1625]: https://github.com/bottlerocket-os/bottlerocket/pull/1625

# v1.1.1 (2021-05-19)

## Security fixes

* Patch runc for CVE-2021-30465 ([232c5741ecec][232c5741ecec])

[232c5741ecec]: https://github.com/bottlerocket-os/bottlerocket/commit/232c5741ecec1b903df3e56922bda03eecb2c02a

# v1.1.0 (2021-05-07)

## Deprecation Notice

The Kubernetes 1.16 variant, `aws-k8s-1.16`, will lose support in July, 2021.
Kubernetes 1.16 is no longer receiving support upstream.
We recommend replacing `aws-k8s-1.16` nodes with a later variant, preferably `aws-k8s-1.19` if your cluster supports it.
See [this issue](https://github.com/bottlerocket-os/bottlerocket/issues/1552) for more details.

## Important Notes

### New variants with new defaults

This release introduces two new variants, `aws-k8s-1.20` and `vmware-k8s-1.20`.
We plan for all new variants, including these, to contain the following changes:
* The kernel is Linux 5.10 rather than 5.4.
* The kernel lockdown mode is set to "integrity" rather than "none".

The ECS preview variant, `aws-ecs-1`, has also been updated with these changes.

Existing `aws-k8s` variants will not receive these changes as they could affect existing workloads.

### ECS task networking

The `aws-ecs-1` variant now supports the `awsvpc` mode of [ECS task networking](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-networking.html).
This allocates an elastic network interface and private IP address to each task.

## OS Changes

* Add Linux kernel 5.10 for use in new variants ([#1526])
* Add aws-k8s-1.20 variant with Kubernetes 1.20 support ([#1437], [#1533])
* Add vmware-k8s-1.20 variant with Kubernetes 1.20 for VMware ([#1511], [#1529], [#1523], [#1502], [#1554])
* Remove aws-k8s-1.15 variant ([#1487], [#1492])
* Constrain ephemeral port range ([#1560])
* Support awsvpc networking mode in ECS ([#1246])
* Add settings for QPS and burst limits of Kubernetes registry pulls, event records, and API ([#1527], [#1532], [#1541])
* Add setting to allow configuration of Kubernetes TLS bootstrap ([#1485])
* Add setting for configuring Kubernetes cloudProvider to allow usage outside AWS ([#1494])
* Make Kubernetes cluster-dns-ip optional to support usage outside of AWS ([#1482])
* Change parameters to support healthy CIS scan ([#1295]) (Thanks, @felipeac!)
* Generate stable machine IDs for VMware and ARM KVM guests ([#1506], [#1537])
* Enable "integrity" kernel lockdown mode for aws-ecs-1 preview variant ([#1530])
* Remove override for default service start timeout ([#1483])
* Restrict access to bootstrap container user data with SELinux ([#1496])
* Split SELinux policy rules for trusted subjects ([#1558])
* Add symlink to allow usage of secrets store CSI drivers ([#1544])
* Prevent bootstrap containers from restarting ([#1508])
* Add udev rules to mount CD-ROM only when media is present ([#1516])
* Add resize2fs binary to sbin ([#1519]) (Thanks, @samjo-nyang!)
* Only restart a host container if affected by settings change ([#1480])
* Support file patterns when specifying log files in logdog ([#1509])
* Daemonize thar-be-settings to avoid zombie processes ([#1507])
* Add support for AWS region ap-northeast-3: Osaka ([#1504])
* Generate pause container URI with standard template variables ([#1551])
* Get cluster DNS IP from cluster when available ([#1547])

## Build Changes

* Use kernel 5.10 in aws-ecs-1 variant ([#1555])
* Build only the packages needed for the current variant ([#1408], [#1520])
* Use a friendly name for VMware OVA files in build outputs ([#1535])
* Update SDK to 0.21.0 ([#1497], [#1529])
* Allow variants to specify extra kernel parameters ([#1491])
* Move kernel console settings to variant definitions ([#1513])
* Update vmw_backdoor dependency ([#1498]) (Thanks, @lucab!)
* Archive old migrations ([#1540])
* Refactor default settings and containerd configs to shared files ([#1538], [#1542])
* Check cargo version at start of build so we have a clear error when it's too low ([#1503])
* Fix concurrency issue in validate-repo that led to hangs ([#1521])
* Update third-party package dependencies ([#1543], [#1556])
* Update Rust dependencies in the tools/ workspace ([#1548])
* Update tokio-related Rust dependencies in the sources/ workspace ([#1479])
* Add upstream runc patches addressing container scheduling failure ([#1546])
* Retry builds on known BuildKit internal errors ([#1557], [#1561])

## Documentation Changes

* Document the deprecation of the aws-k8s-1.15 variant ([#1476])
* Document the need to quote most Kubernetes labels/taints ([#1550]) (Thanks, @ellistarn!)
* Fix VMware spelling and document user data sources ([#1534])

[#1246]: https://github.com/bottlerocket-os/bottlerocket/pull/1246
[#1295]: https://github.com/bottlerocket-os/bottlerocket/pull/1295
[#1408]: https://github.com/bottlerocket-os/bottlerocket/pull/1408
[#1437]: https://github.com/bottlerocket-os/bottlerocket/pull/1437
[#1476]: https://github.com/bottlerocket-os/bottlerocket/pull/1476
[#1477]: https://github.com/bottlerocket-os/bottlerocket/pull/1477
[#1479]: https://github.com/bottlerocket-os/bottlerocket/pull/1479
[#1480]: https://github.com/bottlerocket-os/bottlerocket/pull/1480
[#1482]: https://github.com/bottlerocket-os/bottlerocket/pull/1482
[#1483]: https://github.com/bottlerocket-os/bottlerocket/pull/1483
[#1485]: https://github.com/bottlerocket-os/bottlerocket/pull/1485
[#1486]: https://github.com/bottlerocket-os/bottlerocket/pull/1486
[#1487]: https://github.com/bottlerocket-os/bottlerocket/pull/1487
[#1491]: https://github.com/bottlerocket-os/bottlerocket/pull/1491
[#1492]: https://github.com/bottlerocket-os/bottlerocket/pull/1492
[#1494]: https://github.com/bottlerocket-os/bottlerocket/pull/1494
[#1496]: https://github.com/bottlerocket-os/bottlerocket/pull/1496
[#1497]: https://github.com/bottlerocket-os/bottlerocket/pull/1497
[#1498]: https://github.com/bottlerocket-os/bottlerocket/pull/1498
[#1502]: https://github.com/bottlerocket-os/bottlerocket/pull/1502
[#1503]: https://github.com/bottlerocket-os/bottlerocket/pull/1503
[#1504]: https://github.com/bottlerocket-os/bottlerocket/pull/1504
[#1506]: https://github.com/bottlerocket-os/bottlerocket/pull/1506
[#1507]: https://github.com/bottlerocket-os/bottlerocket/pull/1507
[#1508]: https://github.com/bottlerocket-os/bottlerocket/pull/1508
[#1509]: https://github.com/bottlerocket-os/bottlerocket/pull/1509
[#1511]: https://github.com/bottlerocket-os/bottlerocket/pull/1511
[#1513]: https://github.com/bottlerocket-os/bottlerocket/pull/1513
[#1516]: https://github.com/bottlerocket-os/bottlerocket/pull/1516
[#1519]: https://github.com/bottlerocket-os/bottlerocket/pull/1519
[#1520]: https://github.com/bottlerocket-os/bottlerocket/pull/1520
[#1521]: https://github.com/bottlerocket-os/bottlerocket/pull/1521
[#1523]: https://github.com/bottlerocket-os/bottlerocket/pull/1523
[#1526]: https://github.com/bottlerocket-os/bottlerocket/pull/1526
[#1527]: https://github.com/bottlerocket-os/bottlerocket/pull/1527
[#1529]: https://github.com/bottlerocket-os/bottlerocket/pull/1529
[#1530]: https://github.com/bottlerocket-os/bottlerocket/pull/1530
[#1532]: https://github.com/bottlerocket-os/bottlerocket/pull/1532
[#1533]: https://github.com/bottlerocket-os/bottlerocket/pull/1533
[#1534]: https://github.com/bottlerocket-os/bottlerocket/pull/1534
[#1535]: https://github.com/bottlerocket-os/bottlerocket/pull/1535
[#1537]: https://github.com/bottlerocket-os/bottlerocket/pull/1537
[#1538]: https://github.com/bottlerocket-os/bottlerocket/pull/1538
[#1540]: https://github.com/bottlerocket-os/bottlerocket/pull/1540
[#1541]: https://github.com/bottlerocket-os/bottlerocket/pull/1541
[#1542]: https://github.com/bottlerocket-os/bottlerocket/pull/1542
[#1543]: https://github.com/bottlerocket-os/bottlerocket/pull/1543
[#1544]: https://github.com/bottlerocket-os/bottlerocket/pull/1544
[#1546]: https://github.com/bottlerocket-os/bottlerocket/pull/1546
[#1547]: https://github.com/bottlerocket-os/bottlerocket/pull/1547
[#1548]: https://github.com/bottlerocket-os/bottlerocket/pull/1548
[#1550]: https://github.com/bottlerocket-os/bottlerocket/pull/1550
[#1551]: https://github.com/bottlerocket-os/bottlerocket/pull/1551
[#1554]: https://github.com/bottlerocket-os/bottlerocket/pull/1554
[#1555]: https://github.com/bottlerocket-os/bottlerocket/pull/1555
[#1556]: https://github.com/bottlerocket-os/bottlerocket/pull/1556
[#1557]: https://github.com/bottlerocket-os/bottlerocket/pull/1557
[#1558]: https://github.com/bottlerocket-os/bottlerocket/pull/1558
[#1560]: https://github.com/bottlerocket-os/bottlerocket/pull/1560
[#1561]: https://github.com/bottlerocket-os/bottlerocket/pull/1561

# v1.0.8 (2021-04-12)

## Deprecation Notice

Bottlerocket 1.0.8 is the last release where we plan to support the Kubernetes 1.15 variant, `aws-k8s-1.15`.
Kubernetes 1.15 is no longer receiving support upstream.
We recommend replacing `aws-k8s-1.15` nodes with a later variant, preferably `aws-k8s-1.19` if your cluster supports it.
See [this issue](https://github.com/bottlerocket-os/bottlerocket/issues/1478) for more details.

## OS Changes

* Support additional kubelet arguments: kube-reserved, eviction-hard, cpu-manager-policy, and allow-unsafe-sysctls ([#1388], [#1472], [#1465])
* Expand file and process restrictions in the SELinux policy ([#1464])
* Add support for bootstrap containers ([#1387], [#1423])
* Make host containers inherit proxy env vars ([#1432])
* Allow gzip compression of user data ([#1366])
* Add 'apply' mode to apiclient for applying settings from URIs ([#1391])
* Add compat symlink for kubelet volume plugins ([#1417])
* Remove bottlerocket.version attribute from ECS agent settings ([#1395])
* Make Kubernetes taint values optional ([#1406])
* Add guestinfo to available VMWare user data retrieval methods ([#1393])
* Include source of invalid base64 data in error messages ([#1469])
* Update eni-max-pods data file ([#1468])
* Update default host container versions ([#1443], [#1441], [#1466])
* Fix avc denial for dbus-broker ([#1434])
* Fix case of outputted JSON keys in host container user data ([#1439])
* Set mode of host container persistent storage directory after creation ([#1463])
* Add "current" persistent storage location for host containers ([#1416])
* Write static-pods manifest to tempfile before persisting it ([#1409])

## Build Changes

* Update default variant to aws-k8s-1.19 ([#1394])
* Update third-party packages ([#1460])
* Update Rust dependencies ([#1461], [#1462])
* Update dependencies of host-ctr ([#1371])
* Add support for specifying a variant's supported architectures ([#1431])
* Build OVA packages and include them in repos ([#1428])
* Add support for qcow2 as an image format ([#1425]) (Thanks, @mikalstill!)
* Prevent unneeded artifacts from being copied through build process ([#1426])
* Change image format for vmware-dev variant to vmdk ([#1397])
* Remove tough dependency from update_metadata ([#1390])
* Remove generate_constants logic from build.rs of parse-datetime ([#1376])
* In the tools workspace, update to tokio v1, reqwest v0.11, and tough v0.11 ([#1370])
* Run static and non-static Rust builds in parallel ([#1368])
* Disable CMDLINE_EXTEND kernel configuration ([#1473])

## Documentation Changes

* Document metrics settings in README ([#1449])
* Fix broken links for symlinked files in models README ([#1444])
* Document `apiclient update` as primary CLI update method ([#1421])
* Use `apiclient set` in introductory documentation, explain raw mode separately ([#1418])
* Prefer resolve:ssm: parameters for simplicity in QUICKSTART ([#1363])
* Update quickstart guides to have arm64 examples ([#1360])
* Document the deprecation of the aws-k8s-1.15 variant ([#1476])

[#1360]: https://github.com/bottlerocket-os/bottlerocket/pull/1360
[#1363]: https://github.com/bottlerocket-os/bottlerocket/pull/1363
[#1366]: https://github.com/bottlerocket-os/bottlerocket/pull/1366
[#1368]: https://github.com/bottlerocket-os/bottlerocket/pull/1368
[#1370]: https://github.com/bottlerocket-os/bottlerocket/pull/1370
[#1371]: https://github.com/bottlerocket-os/bottlerocket/pull/1371
[#1376]: https://github.com/bottlerocket-os/bottlerocket/pull/1376
[#1387]: https://github.com/bottlerocket-os/bottlerocket/pull/1387
[#1388]: https://github.com/bottlerocket-os/bottlerocket/pull/1388
[#1390]: https://github.com/bottlerocket-os/bottlerocket/pull/1390
[#1391]: https://github.com/bottlerocket-os/bottlerocket/pull/1391
[#1393]: https://github.com/bottlerocket-os/bottlerocket/pull/1393
[#1394]: https://github.com/bottlerocket-os/bottlerocket/pull/1394
[#1395]: https://github.com/bottlerocket-os/bottlerocket/pull/1395
[#1397]: https://github.com/bottlerocket-os/bottlerocket/pull/1397
[#1406]: https://github.com/bottlerocket-os/bottlerocket/pull/1406
[#1409]: https://github.com/bottlerocket-os/bottlerocket/pull/1409
[#1416]: https://github.com/bottlerocket-os/bottlerocket/pull/1416
[#1417]: https://github.com/bottlerocket-os/bottlerocket/pull/1417
[#1418]: https://github.com/bottlerocket-os/bottlerocket/pull/1418
[#1421]: https://github.com/bottlerocket-os/bottlerocket/pull/1421
[#1423]: https://github.com/bottlerocket-os/bottlerocket/pull/1423
[#1425]: https://github.com/bottlerocket-os/bottlerocket/pull/1425
[#1426]: https://github.com/bottlerocket-os/bottlerocket/pull/1426
[#1428]: https://github.com/bottlerocket-os/bottlerocket/pull/1428
[#1431]: https://github.com/bottlerocket-os/bottlerocket/pull/1431
[#1432]: https://github.com/bottlerocket-os/bottlerocket/pull/1432
[#1434]: https://github.com/bottlerocket-os/bottlerocket/pull/1434
[#1439]: https://github.com/bottlerocket-os/bottlerocket/pull/1439
[#1441]: https://github.com/bottlerocket-os/bottlerocket/pull/1441
[#1443]: https://github.com/bottlerocket-os/bottlerocket/pull/1443
[#1444]: https://github.com/bottlerocket-os/bottlerocket/pull/1444
[#1449]: https://github.com/bottlerocket-os/bottlerocket/pull/1449
[#1460]: https://github.com/bottlerocket-os/bottlerocket/pull/1460
[#1461]: https://github.com/bottlerocket-os/bottlerocket/pull/1461
[#1462]: https://github.com/bottlerocket-os/bottlerocket/pull/1462
[#1463]: https://github.com/bottlerocket-os/bottlerocket/pull/1463
[#1464]: https://github.com/bottlerocket-os/bottlerocket/pull/1464
[#1465]: https://github.com/bottlerocket-os/bottlerocket/pull/1465
[#1466]: https://github.com/bottlerocket-os/bottlerocket/pull/1466
[#1468]: https://github.com/bottlerocket-os/bottlerocket/pull/1468
[#1469]: https://github.com/bottlerocket-os/bottlerocket/pull/1469
[#1472]: https://github.com/bottlerocket-os/bottlerocket/pull/1472
[#1473]: https://github.com/bottlerocket-os/bottlerocket/pull/1473
[#1476]: https://github.com/bottlerocket-os/bottlerocket/pull/1476

# v1.0.7 (2021-03-17)

## Security fixes

* containerd: update to 1.4.4 ([#1401])

## OS Changes

* systemd: update to 247.4 to fix segfault in some cases ([#1400])
* apiserver: reap exited child processes ([#1384])
* host-ctr: specify non-colliding runc root ([#1359])
* updog: update signal-hook dependency ([#1328])

[#1328]: https://github.com/bottlerocket-os/bottlerocket/pull/1328
[#1359]: https://github.com/bottlerocket-os/bottlerocket/pull/1359
[#1384]: https://github.com/bottlerocket-os/bottlerocket/pull/1384
[#1400]: https://github.com/bottlerocket-os/bottlerocket/pull/1400
[#1401]: https://github.com/bottlerocket-os/bottlerocket/pull/1401

# v1.0.6 (2021-03-02)

## OS Changes

* Add metricdog to support sending anonymous metrics ([#1006], [#1322])
* Add a vmware-dev variant ([#1292], [#1288], [#1290])
* Add Kubernetes static pods support ([#1317])
* Add high-level 'set' subcommand for changing settings using apiclient ([#1278])
* Allow admin container to use SSH public keys from user data ([#1331], [#1358], [#19])
* Add support for kubelet in standalone mode and TLS auth ([#1338])
* Add https-proxy and no-proxy settings to updog ([#1324])
* Add support for pulling host-containers from ECR Public ([#1296])
* Add network proxy support to aws-k8s-1.19 ([#1337])
* Modify default SELinux label for containers to align with upstream ([#1318])
* Add aliases for container-selinux types to align with community ([#1316])
* Update default versions of admin and control containers ([#1347], [#1344])
* Update ecs-agent to 1.50.2 ([#1353])
* logdog: Add eni logs for Kubernetes ([#1327])

## Build Changes

* Add the ability to output vmdk via qemu-img ([#1289])
* Add support for kmod kits to ease building of third-party kernel modules ([#1287], [#1286], [#1285], [#1357])
* storewolf: Declare dependencies on model and defaults files ([#1319])
* storewolf: Refactor default settings files to allow sharing ([#1303], [#1329])
* Switch from TermLogger to SimpleLogger ([#1282], **thanks @hencrice!**)
* Allow overriding the "pretty" name of the OS inside the image ([#1330])
* Specify bash in link-variant task for use of bash features ([#1323])
* Fix invalid symlinks when the BUILDSYS_NAME variable is set ([#1312])
* Track and clean output files for builds ([#1291])
* Update third-party software packages ([#1340], [#1336], [#1334], [#1333], [#1335], [#1190], [#1265], [#1315], [#1352], [#1356])

## Documentation Changes

* Add lockdown notes to SECURITY_GUIDANCE.md ([#1281])
* Clarify use case for update repos ([#1339])
* Fix broken link from API docs to top-level docs ([#1306])

[#1006]: https://github.com/bottlerocket-os/bottlerocket/pull/1006
[#1190]: https://github.com/bottlerocket-os/bottlerocket/pull/1190
[#1265]: https://github.com/bottlerocket-os/bottlerocket/pull/1265
[#1278]: https://github.com/bottlerocket-os/bottlerocket/pull/1278
[#1281]: https://github.com/bottlerocket-os/bottlerocket/pull/1281
[#1282]: https://github.com/bottlerocket-os/bottlerocket/pull/1282
[#1285]: https://github.com/bottlerocket-os/bottlerocket/pull/1285
[#1286]: https://github.com/bottlerocket-os/bottlerocket/pull/1286
[#1287]: https://github.com/bottlerocket-os/bottlerocket/pull/1287
[#1288]: https://github.com/bottlerocket-os/bottlerocket/pull/1288
[#1289]: https://github.com/bottlerocket-os/bottlerocket/pull/1289
[#1290]: https://github.com/bottlerocket-os/bottlerocket/pull/1290
[#1291]: https://github.com/bottlerocket-os/bottlerocket/pull/1291
[#1292]: https://github.com/bottlerocket-os/bottlerocket/pull/1292
[#1296]: https://github.com/bottlerocket-os/bottlerocket/pull/1296
[#1303]: https://github.com/bottlerocket-os/bottlerocket/pull/1303
[#1306]: https://github.com/bottlerocket-os/bottlerocket/pull/1306
[#1312]: https://github.com/bottlerocket-os/bottlerocket/pull/1312
[#1315]: https://github.com/bottlerocket-os/bottlerocket/pull/1315
[#1316]: https://github.com/bottlerocket-os/bottlerocket/pull/1316
[#1317]: https://github.com/bottlerocket-os/bottlerocket/pull/1317
[#1318]: https://github.com/bottlerocket-os/bottlerocket/pull/1318
[#1319]: https://github.com/bottlerocket-os/bottlerocket/pull/1319
[#1322]: https://github.com/bottlerocket-os/bottlerocket/pull/1322
[#1323]: https://github.com/bottlerocket-os/bottlerocket/pull/1323
[#1324]: https://github.com/bottlerocket-os/bottlerocket/pull/1324
[#1327]: https://github.com/bottlerocket-os/bottlerocket/pull/1327
[#1329]: https://github.com/bottlerocket-os/bottlerocket/pull/1329
[#1330]: https://github.com/bottlerocket-os/bottlerocket/pull/1330
[#1331]: https://github.com/bottlerocket-os/bottlerocket/pull/1331
[#1333]: https://github.com/bottlerocket-os/bottlerocket/pull/1333
[#1334]: https://github.com/bottlerocket-os/bottlerocket/pull/1334
[#1335]: https://github.com/bottlerocket-os/bottlerocket/pull/1335
[#1336]: https://github.com/bottlerocket-os/bottlerocket/pull/1336
[#1337]: https://github.com/bottlerocket-os/bottlerocket/pull/1337
[#1338]: https://github.com/bottlerocket-os/bottlerocket/pull/1338
[#1339]: https://github.com/bottlerocket-os/bottlerocket/pull/1339
[#1340]: https://github.com/bottlerocket-os/bottlerocket/pull/1340
[#1344]: https://github.com/bottlerocket-os/bottlerocket/pull/1344
[#1347]: https://github.com/bottlerocket-os/bottlerocket/pull/1347
[#1352]: https://github.com/bottlerocket-os/bottlerocket/pull/1352
[#1353]: https://github.com/bottlerocket-os/bottlerocket/pull/1353
[#1356]: https://github.com/bottlerocket-os/bottlerocket/pull/1356
[#1357]: https://github.com/bottlerocket-os/bottlerocket/pull/1357
[#1358]: https://github.com/bottlerocket-os/bottlerocket/pull/1358
[#19]: https://github.com/bottlerocket-os/bottlerocket-admin-container/pull/19

# v1.0.5 (2021-01-15)

**Note for aws-ecs-1 variant**: due to a change in the ECS agent's data store schema, the aws-ecs-1 variant cannot be downgraded after updating to v1.0.5.
Attempts to downgrade may result in inconsistencies between ECS and the Bottlerocket container instance.

## OS Changes

* Add aws-k8s-1.19 variant with Kubernetes 1.19 ([#1256])
* Update ecs-agent to 1.48.1 ([#1201])
* Add high-level update subcommands to apiclient ([#1219], [#1232])
* Add kernel lockdown settings ([#1223], [#1279])
* Add restart-commands for docker, kubelet, containerd ([#1231], [#1262], [#1258])
* Add proper restarts for host-containers ([#1230], [#1235], [#1242], [#1258])
* Fix SELinux policy ([#1236])
* Set version and revision strings for containerd ([#1248])
* Add host-container user-data setting ([#1244], [#1247])
* Add network proxy settings ([#1204], [#1262], [#1258])
* Update kernel to 5.4.80-40.140 ([#1257])
* Update third-party software packages ([#1264])
* Update Rust dependencies ([#1267])

## Build Changes

* Improve support for out-of-tree kernel modules ([#1220])
* Fix message in partition size check condition ([#1233], **thanks @pranavek!**)
* Split the datastore module into its own crate ([#1249])
* Update SDK to v0.15.0 ([#1263])
* Update Github Actions to ignore changes that only include .md files ([#1274])

## Documentation Changes

* Add documentation comments to Dockerfile ([#1254])
* Add a note about CPU usage during builds ([#1266])
* Update README to point to discussions ([#1273])

[#1201]: https://github.com/bottlerocket-os/bottlerocket/pull/1201
[#1204]: https://github.com/bottlerocket-os/bottlerocket/pull/1204
[#1219]: https://github.com/bottlerocket-os/bottlerocket/pull/1219
[#1220]: https://github.com/bottlerocket-os/bottlerocket/pull/1220
[#1223]: https://github.com/bottlerocket-os/bottlerocket/pull/1223
[#1230]: https://github.com/bottlerocket-os/bottlerocket/pull/1230
[#1231]: https://github.com/bottlerocket-os/bottlerocket/pull/1231
[#1232]: https://github.com/bottlerocket-os/bottlerocket/pull/1232
[#1233]: https://github.com/bottlerocket-os/bottlerocket/pull/1233
[#1235]: https://github.com/bottlerocket-os/bottlerocket/pull/1235
[#1236]: https://github.com/bottlerocket-os/bottlerocket/pull/1236
[#1242]: https://github.com/bottlerocket-os/bottlerocket/pull/1242
[#1244]: https://github.com/bottlerocket-os/bottlerocket/pull/1244
[#1247]: https://github.com/bottlerocket-os/bottlerocket/pull/1247
[#1248]: https://github.com/bottlerocket-os/bottlerocket/pull/1248
[#1249]: https://github.com/bottlerocket-os/bottlerocket/pull/1249
[#1254]: https://github.com/bottlerocket-os/bottlerocket/pull/1254
[#1256]: https://github.com/bottlerocket-os/bottlerocket/pull/1256
[#1257]: https://github.com/bottlerocket-os/bottlerocket/pull/1257
[#1258]: https://github.com/bottlerocket-os/bottlerocket/pull/1258
[#1259]: https://github.com/bottlerocket-os/bottlerocket/pull/1259
[#1262]: https://github.com/bottlerocket-os/bottlerocket/pull/1262
[#1263]: https://github.com/bottlerocket-os/bottlerocket/pull/1263
[#1264]: https://github.com/bottlerocket-os/bottlerocket/pull/1264
[#1266]: https://github.com/bottlerocket-os/bottlerocket/pull/1266
[#1267]: https://github.com/bottlerocket-os/bottlerocket/pull/1267
[#1273]: https://github.com/bottlerocket-os/bottlerocket/pull/1273
[#1274]: https://github.com/bottlerocket-os/bottlerocket/pull/1274
[#1279]: https://github.com/bottlerocket-os/bottlerocket/pull/1279

# v1.0.4 (2020-11-30)

## Security fixes

* Patch containerd for CVE-2020-15257 ([f3677c1406][f3677c1406])

[f3677c1406]: https://github.com/bottlerocket-os/bottlerocket/commit/f3677c1406139240d2bca6b275799953ced5a5f

# v1.0.3 (2020-11-19)

## OS Changes
* Support setting Linux kernel parameters (sysctl) via settings (see README) ([#1158], [#1171])
* Create links under `/dev/disk/ephemeral` for ephemeral storage devices ([#1173])
* Set default RLIMIT_NOFILE in CRI to 65536 soft limit and a 1048576 hard limit ([#1180])
* Add rtcsync directive to chrony config file ([#1184], **thanks @errm!**)
* Add `/etc/ssl/certs` symlink to the CA certificate bundle for compatibility with the cluster autoscaler ([#1207])
* Add procps dependency to docker-engine so that `docker top` works ([#1210])

## Build Changes
* Align optimization level for crate and dependency builds ([#1155])
* pubsys no longer requires an Infra.toml file for basic usage ([#1166])
* Makefile: Check that $BUILDSYS_ARCH has a supported value ([#1167])
* Build migrations in parallel ([#1192])
* Allow file URLs for role in pubsys-setup ([#1194])
* Update Rust dependencies ([#1196])
* Update SDK to v0.14.0 ([#1198])
* Fix an occasional issue with KMS signing in pubsys ([#1205])
* Backport selected fixes from containerd 1.4 ([#1216])
* Update third-party package dependencies ([#1176], [#1195])
* Switch to SDK v0.14.0 ([#1198])

## Documentation Changes
* Nits and fixes ([#1170], [#1179])
* Add missing prerequisites for building Bottlerocket ([#1191])

[#1158]: https://github.com/bottlerocket-os/bottlerocket/pull/1158
[#1171]: https://github.com/bottlerocket-os/bottlerocket/pull/1171
[#1173]: https://github.com/bottlerocket-os/bottlerocket/pull/1173
[#1176]: https://github.com/bottlerocket-os/bottlerocket/pull/1176
[#1180]: https://github.com/bottlerocket-os/bottlerocket/pull/1180
[#1184]: https://github.com/bottlerocket-os/bottlerocket/pull/1184
[#1195]: https://github.com/bottlerocket-os/bottlerocket/pull/1195
[#1207]: https://github.com/bottlerocket-os/bottlerocket/pull/1207
[#1155]: https://github.com/bottlerocket-os/bottlerocket/pull/1155
[#1166]: https://github.com/bottlerocket-os/bottlerocket/pull/1166
[#1167]: https://github.com/bottlerocket-os/bottlerocket/pull/1167
[#1192]: https://github.com/bottlerocket-os/bottlerocket/pull/1192
[#1194]: https://github.com/bottlerocket-os/bottlerocket/pull/1194
[#1196]: https://github.com/bottlerocket-os/bottlerocket/pull/1196
[#1198]: https://github.com/bottlerocket-os/bottlerocket/pull/1198
[#1205]: https://github.com/bottlerocket-os/bottlerocket/pull/1205
[#1170]: https://github.com/bottlerocket-os/bottlerocket/pull/1170
[#1179]: https://github.com/bottlerocket-os/bottlerocket/pull/1179
[#1191]: https://github.com/bottlerocket-os/bottlerocket/pull/1191
[#1210]: https://github.com/bottlerocket-os/bottlerocket/pull/1210
[#1216]: https://github.com/bottlerocket-os/bottlerocket/pull/1216
[#1198]: https://github.com/bottlerocket-os/bottlerocket/pull/1198

# v1.0.2 (2020-10-13)

## Breaking changes (for build process only)

* pubsys: automate setup of role and key ([#1133], [#1146])
* Store repos under repo name so you can build multiple ([#1135])

**Note:** these changes do not impact users of Bottlerocket AMIs or repos, only those who build Bottlerocket themselves.
If you use an `Infra.toml` file to automate publishing, you'll need to update the format of the file.
The root role and signing key definitions now live inside a repo definition, rather than at the top level of the file.
Please see the updated [Infra.toml.example](tools/pubsys/Infra.toml.example) file for a commented explanation of the new role and key configuration.

## OS changes

* Add aws-k8s-1.18 variant with Kubernetes 1.18 ([#1150])
* Update kernel to 5.4.50-25.83 ([#1148])
* Update glibc to 2.32 ([#1092])
* Add e2fsprogs ([#1147])
* pluto: add regional map of pause container source accounts ([#1142])
* Add option to enable spot instance draining ([#1100], **thanks @mkulke!**)
* Add 2.root.json + pubsys KMS support ([#1122])
* docker: add default nofiles ulimits for containers ([#1119])
* Fix AVC denial for`docker run --init` ([#1085])

## Build changes

* Pass Go module proxy variables through docker-go ([#1121])
* Set buildmode to pie and drop pie and debuginfo patches for Kubernetes ([#1103], **thanks @bnrjee!**)
* pubsys: use requested size for volume, keeping snapshot to minimum size ([#1118])
* Switch to SDK v0.13.0 ([#1092])
* Add `cargo make grant-ami` and `revoke-ami` tasks ([#1087])
* Allow specifying AMI name with PUBLISH_AMI_NAME ([#1091])
* Makefile.toml: clean up clean actions ([#1089])
* pubsys: check for copied AMIs in parallel ([#1086])

## Documentation changes

* Add PUBLISHING.md guide explaining pubsys and related tools ([#1138])
* README: relocate update API instructions and example ([#1124], [#1127])
* Fix grammar issues in README.md ([#1098], **thanks @jweissig!**)
* Add documentation for the aws-ecs-1 variant ([#1053])
* Update suggested Kubernetes version in sample eksctl config files ([#1090])
* Update BUILDING.md to incorporate dependencies ([#1107], **thanks @troyaws!**)


[#1053]: https://github.com/bottlerocket-os/bottlerocket/pull/1053
[#1084]: https://github.com/bottlerocket-os/bottlerocket/pull/1084
[#1085]: https://github.com/bottlerocket-os/bottlerocket/pull/1085
[#1086]: https://github.com/bottlerocket-os/bottlerocket/pull/1086
[#1087]: https://github.com/bottlerocket-os/bottlerocket/pull/1087
[#1089]: https://github.com/bottlerocket-os/bottlerocket/pull/1089
[#1090]: https://github.com/bottlerocket-os/bottlerocket/pull/1090
[#1091]: https://github.com/bottlerocket-os/bottlerocket/pull/1091
[#1092]: https://github.com/bottlerocket-os/bottlerocket/pull/1092
[#1094]: https://github.com/bottlerocket-os/bottlerocket/pull/1094
[#1098]: https://github.com/bottlerocket-os/bottlerocket/pull/1098
[#1100]: https://github.com/bottlerocket-os/bottlerocket/pull/1100
[#1103]: https://github.com/bottlerocket-os/bottlerocket/pull/1103
[#1107]: https://github.com/bottlerocket-os/bottlerocket/pull/1107
[#1109]: https://github.com/bottlerocket-os/bottlerocket/pull/1109
[#1118]: https://github.com/bottlerocket-os/bottlerocket/pull/1118
[#1119]: https://github.com/bottlerocket-os/bottlerocket/pull/1119
[#1121]: https://github.com/bottlerocket-os/bottlerocket/pull/1121
[#1122]: https://github.com/bottlerocket-os/bottlerocket/pull/1122
[#1124]: https://github.com/bottlerocket-os/bottlerocket/pull/1124
[#1127]: https://github.com/bottlerocket-os/bottlerocket/pull/1127
[#1133]: https://github.com/bottlerocket-os/bottlerocket/pull/1133
[#1135]: https://github.com/bottlerocket-os/bottlerocket/pull/1135
[#1138]: https://github.com/bottlerocket-os/bottlerocket/pull/1138
[#1142]: https://github.com/bottlerocket-os/bottlerocket/pull/1142
[#1146]: https://github.com/bottlerocket-os/bottlerocket/pull/1146
[#1147]: https://github.com/bottlerocket-os/bottlerocket/pull/1147
[#1148]: https://github.com/bottlerocket-os/bottlerocket/pull/1148
[#1149]: https://github.com/bottlerocket-os/bottlerocket/pull/1149
[#1150]: https://github.com/bottlerocket-os/bottlerocket/pull/1150

# v1.0.1 (2020-09-03)

## Security fixes

* Patch kernel for CVE-2020-14386 ([#1108])

[#1108]: https://github.com/bottlerocket-os/bottlerocket/pull/1108

# v1.0.0 (2020-08-31)

Welcome to Bottlerocket 1.0!
Since the first public preview, we've added new variants for Amazon ECS and Kubernetes 1.16 and 1.17, support for ARM instances and more EC2 regions, along with many new features and security improvements.
We appreciate all the feedback and contributions so far and look forward to working with the community on even wider support.

:partying_face: :smile_cat:

## Security fixes

* Update to chrony 3.5.1 ([#1057])
* Isolate host containers and limit access to API socket ([#1056])

## OS changes

* The `aws-ecs-1` variant is now available as a preview.
   * ecs-agent: upgrade to v1.43.0 ([#1043])
   * aws-ecs-1: add ecs.loglevel setting ([#1062])
   * aws-ecs-1: remove unsupported capabilities ([#1052])
   * aws-ecs-1: constrain ephemeral port range ([#1051])
   * aws-ecs-1: enable awslogs execution role support ([#1044])
   * ecs-agent: don't start if not configured ([#1049])
   * ecs-agent: bind introspection to localhost ([#1071])
   * Update logdog to pull ECS-related log files ([#1054])
   * Add documentation for the aws-ecs-1 variant ([#1053])
* apiclient: accept -s for --socket-path, as per usage message ([#1069])
* Fix growpart to avoid race in partition table reload ([#1058])
* Added patch for EC2 IMDSv2 support in Docker ([#1055])
* schnauzer: add a helper for ecr repos ([#1032])

## Build changes

* Add `cargo make ami-public` and `ami-private` targets ([#1033], [#1065], [#1064])
* Add `cargo make ssm` and `promote-ssm` targets for publishing parameters ([#1060], [#1070], [#1067], [#1066])
* Use per-checkout cache directories for builds ([#1050])
* Fix rust build caching and tune rpm compression ([#1045])
* Add official builds in 16 more EC2 regions. ([aws/containers-roadmap#827](https://github.com/aws/containers-roadmap/issues/827))

## Documentation changes

* Revise security guidance ([#1072])
* README: add supported architectures ([#1048])
* Update supported region list after 0.5.0 release ([#1046])
* Removed aws-cli v1 requirement in docs ([#1073])
* Update BUILDING.md for new coldsnap-based amiize.sh ([#1047])


[#1073]: https://github.com/bottlerocket-os/bottlerocket/pull/1073
[#1072]: https://github.com/bottlerocket-os/bottlerocket/pull/1072
[#1071]: https://github.com/bottlerocket-os/bottlerocket/pull/1071
[#1070]: https://github.com/bottlerocket-os/bottlerocket/pull/1070
[#1069]: https://github.com/bottlerocket-os/bottlerocket/pull/1069
[#1067]: https://github.com/bottlerocket-os/bottlerocket/pull/1067
[#1066]: https://github.com/bottlerocket-os/bottlerocket/pull/1066
[#1065]: https://github.com/bottlerocket-os/bottlerocket/pull/1065
[#1064]: https://github.com/bottlerocket-os/bottlerocket/pull/1064
[#1062]: https://github.com/bottlerocket-os/bottlerocket/pull/1062
[#1060]: https://github.com/bottlerocket-os/bottlerocket/pull/1060
[#1058]: https://github.com/bottlerocket-os/bottlerocket/pull/1058
[#1057]: https://github.com/bottlerocket-os/bottlerocket/pull/1057
[#1056]: https://github.com/bottlerocket-os/bottlerocket/pull/1056
[#1055]: https://github.com/bottlerocket-os/bottlerocket/pull/1055
[#1054]: https://github.com/bottlerocket-os/bottlerocket/pull/1054
[#1053]: https://github.com/bottlerocket-os/bottlerocket/pull/1053
[#1052]: https://github.com/bottlerocket-os/bottlerocket/pull/1052
[#1051]: https://github.com/bottlerocket-os/bottlerocket/pull/1051
[#1050]: https://github.com/bottlerocket-os/bottlerocket/pull/1050
[#1049]: https://github.com/bottlerocket-os/bottlerocket/pull/1049
[#1048]: https://github.com/bottlerocket-os/bottlerocket/pull/1048
[#1047]: https://github.com/bottlerocket-os/bottlerocket/pull/1047
[#1046]: https://github.com/bottlerocket-os/bottlerocket/pull/1046
[#1045]: https://github.com/bottlerocket-os/bottlerocket/pull/1045
[#1044]: https://github.com/bottlerocket-os/bottlerocket/pull/1044
[#1043]: https://github.com/bottlerocket-os/bottlerocket/pull/1043
[#1033]: https://github.com/bottlerocket-os/bottlerocket/pull/1033
[#1032]: https://github.com/bottlerocket-os/bottlerocket/pull/1032


# v0.5.0 (2020-08-14)

Special thanks to first-time contributor @spoonofpower ([#988])!

## Breaking changes

* Remove support for unsigned datastore migrations ([#976])

## OS changes

* Add `aws-ecs-1` variant prototype for running containers in ECS clusters ([#946], [#1005], [#1007], [#1008], [#1009], [#1017])
* Configurable `clusterDomain` kubelet setting via `settings.kubernetes.cluster-domain` ([#988], [#1036])
* Make update position within waves consistent ([#993])
* Fix kubelet configuration for `MaxPods` ([#994])
* Update `eni-max-pods` with new instance types ([#994])
* Fix `max_versions` unit test in `updata` ([#998])
* Remove injection of `label:disable` option for privileged containers in Docker ([#1013])
* Add `policycoreutils` and related tools ([#1016])
* Update third-party software packages ([#1018], [#1023], [#1025], [#1026])
* Update Rust dependencies ([#1019], [#1021])
* Update `host-ctr`'s dependencies ([#1020])
* Update the host-containers' default versions ([#1030], [#1040])
* Allow access to all device nodes for superpowered host-containers ([#1037])

## Build changes

* Add `pubsys` (`cargo make repo`, `cargo make ami`) for repo and AMI creation ([#964], [#1010], [#1028], [#1034])
* Require `updata init` before creating a new repo manifest ([#991])
* Exclude README.md files from cargo change tracking ([#995], [#996])
* Build `aws-k8s-1.17` variant by default with `cargo make` ([#1002])
* Update comments to be more accurate in Infra.toml ([#1004])
* Update `amiize` to use `coldsnap` ([#1012])
* Update Bottlerocket SDK to v0.12.0 ([#1014])
* Fix warnings for use of deprecated items in `common_migrations` ([#1022])

## Documentation changes

* Removed instructions to manually apply the manifest for aws-vpc-cni-k8s ([#1029])

[#946]: https://github.com/bottlerocket-os/bottlerocket/pull/946
[#964]: https://github.com/bottlerocket-os/bottlerocket/pull/964
[#976]: https://github.com/bottlerocket-os/bottlerocket/pull/976
[#988]: https://github.com/bottlerocket-os/bottlerocket/pull/988
[#991]: https://github.com/bottlerocket-os/bottlerocket/pull/991
[#993]: https://github.com/bottlerocket-os/bottlerocket/pull/993
[#994]: https://github.com/bottlerocket-os/bottlerocket/pull/994
[#995]: https://github.com/bottlerocket-os/bottlerocket/pull/995
[#996]: https://github.com/bottlerocket-os/bottlerocket/pull/996
[#998]: https://github.com/bottlerocket-os/bottlerocket/pull/998
[#1002]: https://github.com/bottlerocket-os/bottlerocket/pull/1002
[#1004]: https://github.com/bottlerocket-os/bottlerocket/pull/1004
[#1005]: https://github.com/bottlerocket-os/bottlerocket/pull/1005
[#1007]: https://github.com/bottlerocket-os/bottlerocket/pull/1007
[#1008]: https://github.com/bottlerocket-os/bottlerocket/pull/1008
[#1009]: https://github.com/bottlerocket-os/bottlerocket/pull/1009
[#1010]: https://github.com/bottlerocket-os/bottlerocket/pull/1010
[#1012]: https://github.com/bottlerocket-os/bottlerocket/pull/1012
[#1013]: https://github.com/bottlerocket-os/bottlerocket/pull/1013
[#1014]: https://github.com/bottlerocket-os/bottlerocket/pull/1014
[#1016]: https://github.com/bottlerocket-os/bottlerocket/pull/1016
[#1017]: https://github.com/bottlerocket-os/bottlerocket/pull/1017
[#1018]: https://github.com/bottlerocket-os/bottlerocket/pull/1018
[#1019]: https://github.com/bottlerocket-os/bottlerocket/pull/1019
[#1020]: https://github.com/bottlerocket-os/bottlerocket/pull/1020
[#1021]: https://github.com/bottlerocket-os/bottlerocket/pull/1021
[#1022]: https://github.com/bottlerocket-os/bottlerocket/pull/1022
[#1023]: https://github.com/bottlerocket-os/bottlerocket/pull/1023
[#1025]: https://github.com/bottlerocket-os/bottlerocket/pull/1025
[#1026]: https://github.com/bottlerocket-os/bottlerocket/pull/1026
[#1028]: https://github.com/bottlerocket-os/bottlerocket/pull/1028
[#1029]: https://github.com/bottlerocket-os/bottlerocket/pull/1029
[#1030]: https://github.com/bottlerocket-os/bottlerocket/pull/1030
[#1034]: https://github.com/bottlerocket-os/bottlerocket/pull/1034
[#1036]: https://github.com/bottlerocket-os/bottlerocket/pull/1036
[#1037]: https://github.com/bottlerocket-os/bottlerocket/pull/1037
[#1040]: https://github.com/bottlerocket-os/bottlerocket/pull/1040

# v0.4.1 (2020-07-13)

## Security fixes

* Patch Kubernetes for CVE-2020-8558 ([#977])
* Update `tough` to 0.7.1 to patch CVE-2020-15093 ([#979])

## OS changes

* Add a new `aws-k8s-1.17` variant for Kubernetes 1.17 ([#973])
* Confine `chrony`, `wicked`, and `dbus-broker` via SELinux, and persist their state to disk ([#970])
* Persist `systemd` journal to disk ([#970])
* Add an API for OS updates ([#942], [#959], [#986])
* Add migration helpers to add / remove multiple settings at once ([#958])
* Fix SELinux policy to allow CSI driver mounts and transition used by Kaniko ([#983])
* Update to new repo URL via migration to ensure signed migration support ([#980])

## Build changes

* Fix environment variable override for build output directory ([#963])
* Update `.dockerignore` to account for the new build output directory structure ([#967])
* Remove the `preview-docs` task from `Makefile` ([#969])

## Documentation changes

* Document new update APIs and add associated diagrams ([#962])
* Add `ap-south-1` to supported regions ([#965])
* Fix `storewolf`'s documentation and usage message as it expects a semver value ([#957])

[#942]: https://github.com/bottlerocket-os/bottlerocket/pull/942
[#957]: https://github.com/bottlerocket-os/bottlerocket/pull/957
[#958]: https://github.com/bottlerocket-os/bottlerocket/pull/958
[#959]: https://github.com/bottlerocket-os/bottlerocket/pull/959
[#962]: https://github.com/bottlerocket-os/bottlerocket/pull/962
[#963]: https://github.com/bottlerocket-os/bottlerocket/pull/963
[#965]: https://github.com/bottlerocket-os/bottlerocket/pull/965
[#967]: https://github.com/bottlerocket-os/bottlerocket/pull/967
[#969]: https://github.com/bottlerocket-os/bottlerocket/pull/969
[#970]: https://github.com/bottlerocket-os/bottlerocket/pull/970
[#973]: https://github.com/bottlerocket-os/bottlerocket/pull/973
[#977]: https://github.com/bottlerocket-os/bottlerocket/pull/977
[#979]: https://github.com/bottlerocket-os/bottlerocket/pull/979
[#980]: https://github.com/bottlerocket-os/bottlerocket/pull/980
[#983]: https://github.com/bottlerocket-os/bottlerocket/pull/983
[#986]: https://github.com/bottlerocket-os/bottlerocket/pull/986

# v0.4.0 (2020-06-25)

## Breaking changes

* Remove all permissive types from the SELinux policy ([#945]). Actions that were not allowed by the SELinux policy now fail instead of only being logged.

## OS changes

* Use update repository metadata and signatures to run settings migrations ([#930])
* Mount debugfs in superpowered host containers, such as the admin container, to support tools like `bcc` and `bpftrace` ([#934])
* Protect container snapshot layers in SELinux policy ([#935])
* Add `POST /actions/reboot` API path ([#936])
* Update `tough` to v0.6.0 ([#944])
* Fix behavior of `signpost cancel-upgrade` ([#950])
* Update to kernel 5.4.46 ([#953])

## Build changes

* Canonicalize architecture names in amiize.sh ([#932])
* Split build output directories by variant and architecture ([#948])
* Move intermediate RPM output from `build/packages` to `build/rpms` ([#948])
* Fix `chmod` usage for building on macOS ([#951])

## Documentation changes

* Document platform-specific settings in README.md ([#941])

[#930]: https://github.com/bottlerocket-os/bottlerocket/pull/930
[#932]: https://github.com/bottlerocket-os/bottlerocket/pull/932
[#934]: https://github.com/bottlerocket-os/bottlerocket/pull/934
[#935]: https://github.com/bottlerocket-os/bottlerocket/pull/935
[#936]: https://github.com/bottlerocket-os/bottlerocket/pull/936
[#941]: https://github.com/bottlerocket-os/bottlerocket/pull/941
[#944]: https://github.com/bottlerocket-os/bottlerocket/pull/944
[#945]: https://github.com/bottlerocket-os/bottlerocket/pull/945
[#948]: https://github.com/bottlerocket-os/bottlerocket/pull/948
[#950]: https://github.com/bottlerocket-os/bottlerocket/pull/950
[#951]: https://github.com/bottlerocket-os/bottlerocket/pull/951
[#953]: https://github.com/bottlerocket-os/bottlerocket/pull/953

# v0.3.4 (2020-05-27)

## OS changes

* Add a new Kubernetes 1.16 variant ([#919])
* Use SELinux to restrict datastore modifications ([#917])
* Add variant override to updog arguments ([#923])

## Build changes

* Update systemd to v245 ([#916])
* Update build SDK to v0.11.0 ([#926])
* Allow specifying a start time for waves in updata ([#927])
* Update `tough` dependencies to v0.5.0 ([#928])

[#916]: https://github.com/bottlerocket-os/bottlerocket/pull/916
[#917]: https://github.com/bottlerocket-os/bottlerocket/pull/917
[#919]: https://github.com/bottlerocket-os/bottlerocket/pull/919
[#923]: https://github.com/bottlerocket-os/bottlerocket/pull/923
[#926]: https://github.com/bottlerocket-os/bottlerocket/pull/926
[#927]: https://github.com/bottlerocket-os/bottlerocket/pull/927
[#928]: https://github.com/bottlerocket-os/bottlerocket/pull/928

# v0.3.3 (2020-05-14)

## OS changes

* Security: update kernel to 5.4.38 ([#924])

[#924]: https://github.com/bottlerocket-os/bottlerocket/pull/924

# v0.3.2 (2020-04-20)

Special thanks to our first contributors, @inductor ([#853]), @smoser ([#871]), and @gliptak ([#870])!

## OS changes

* Update kernel to 5.4.20 ([#898])
* Expand SELinux policy to include all classes and actions in 5.4 kernel ([#888])
* Include error messages in apiserver error responses ([#897])
* Add "logdog" to help users collect debug logs ([#880])
* Include objtool in kernel-devel for compiling external modules ([#874])
* Ignore termination signals in updog right before initiating reboot ([#869])
* Pass `--containerd` flag to kubelet to specify containerd socket path, fixing some cAdvisor metrics ([#868])
* Fix delay on reboot or power off ([#859])
* Add `systemd.log_color=0` to remove ANSI color escapes from console log ([#836])
* Reduce containerd logging when no errors have occurred ([#886])
* Update admin container to v0.5.0 ([#903])

## Build changes

* Set up GitHub Actions to test OS builds for PRs ([#837])
* Update SDK to v0.10.1 ([#866])
* Move built RPMs to `build/packages` ([#863])
* Bump cargo-make to 0.30.0 ([#870])
* Pass proxy environment variables through to docker containers ([#871])
* Add parse-datetime crate ([#875])
* Update third-party software packages ([#895])
* Update Rust dependencies ([#896])
* Remove unused Rust dependencies ([#894])
* Add upstream fix for arm64 in coreutils ([#879])
* Add ability to add waves using TOML files ([#883])
* Add default wave files ([#881])
* Fix migrations builds ([#906])

## Documentation changes

* QUICKSTART: Clarify which setup is optional ([#902])
* QUICKSTART: add easier setup instructions using new eksctl release ([#849])
* QUICKSTART: add note about allowing SSH access ([#839])
* QUICKSTART: add section on finding AMIs through SSM parameters ([#838])
* QUICKSTART: Add supported region list ([73d120c9])
* QUICKSTART: Add info about persistent volume CSI plugin ([#899])
* QUICKSTART and README: Add appropriate ECR policy guidance ([#856])
* README: Fix feedback link to point at existing section ([#833])
* README: Add sentence about preview phase with feedback link ([#832])
* README: Fixes and updates ([#831])
* Update name of early-boot-config in API system diagram ([#840])
* Fix updater README's reference to data store version ([#844])
* Fix example wave files ([#908])

[#831]: https://github.com/bottlerocket-os/bottlerocket/pull/831
[#832]: https://github.com/bottlerocket-os/bottlerocket/pull/832
[#833]: https://github.com/bottlerocket-os/bottlerocket/pull/833
[#836]: https://github.com/bottlerocket-os/bottlerocket/pull/836
[#837]: https://github.com/bottlerocket-os/bottlerocket/pull/837
[#838]: https://github.com/bottlerocket-os/bottlerocket/pull/838
[#839]: https://github.com/bottlerocket-os/bottlerocket/pull/839
[#840]: https://github.com/bottlerocket-os/bottlerocket/pull/840
[#844]: https://github.com/bottlerocket-os/bottlerocket/pull/844
[#849]: https://github.com/bottlerocket-os/bottlerocket/pull/849
[#853]: https://github.com/bottlerocket-os/bottlerocket/pull/853
[#856]: https://github.com/bottlerocket-os/bottlerocket/pull/856
[#859]: https://github.com/bottlerocket-os/bottlerocket/pull/859
[#860]: https://github.com/bottlerocket-os/bottlerocket/pull/860
[#863]: https://github.com/bottlerocket-os/bottlerocket/pull/863
[#866]: https://github.com/bottlerocket-os/bottlerocket/pull/866
[#868]: https://github.com/bottlerocket-os/bottlerocket/pull/868
[#869]: https://github.com/bottlerocket-os/bottlerocket/pull/869
[#870]: https://github.com/bottlerocket-os/bottlerocket/pull/870
[#871]: https://github.com/bottlerocket-os/bottlerocket/pull/871
[#874]: https://github.com/bottlerocket-os/bottlerocket/pull/874
[#875]: https://github.com/bottlerocket-os/bottlerocket/pull/875
[#879]: https://github.com/bottlerocket-os/bottlerocket/pull/879
[#880]: https://github.com/bottlerocket-os/bottlerocket/pull/880
[#881]: https://github.com/bottlerocket-os/bottlerocket/pull/881
[#883]: https://github.com/bottlerocket-os/bottlerocket/pull/883
[#886]: https://github.com/bottlerocket-os/bottlerocket/pull/886
[#888]: https://github.com/bottlerocket-os/bottlerocket/pull/888
[#894]: https://github.com/bottlerocket-os/bottlerocket/pull/894
[#895]: https://github.com/bottlerocket-os/bottlerocket/pull/895
[#896]: https://github.com/bottlerocket-os/bottlerocket/pull/896
[#897]: https://github.com/bottlerocket-os/bottlerocket/pull/897
[#898]: https://github.com/bottlerocket-os/bottlerocket/pull/898
[#899]: https://github.com/bottlerocket-os/bottlerocket/pull/899
[#902]: https://github.com/bottlerocket-os/bottlerocket/pull/902
[#903]: https://github.com/bottlerocket-os/bottlerocket/pull/903
[#906]: https://github.com/bottlerocket-os/bottlerocket/pull/906
[#908]: https://github.com/bottlerocket-os/bottlerocket/pull/908
[73d120c9]: https://github.com/bottlerocket-os/bottlerocket/commit/73d120c9

# v0.3.1 (2020-03-10)

## OS changes

* Log migration errors to console ([#795])
* Enable BTF debug info (`CONFIG_DEBUG_INFO_BTF`) ([#799])
* Move migrations from private partition to data partition ([#818])
* Add top-level model struct ([#824])
* Update ca-certificates, cni-plugins, coreutils, dbus-broker, iproute, kmod, libcap, libxcrypt, ncurses, socat, and wicked ([#826])

## Build changes

* Update Rust dependencies ([#798], [#806], [#809], [#810])
* Add additional cleanup steps to amiize.sh ([#804])
* Work around warnings for unused licenses ([#827])

## Documentation changes

* Add [GLOSSARY.md](GLOSSARY.md), [SECURITY_FEATURES.md](SECURITY_FEATURES.md), and [SECURITY_GUIDANCE.md](SECURITY_GUIDANCE.md) ([#800], [#807], [#821])
* Add additional information to top section of [README.md](README.md) ([#802])
* Add license information to OpenAPI specification ([#803])
* Add description of source mirroring ([#817])
* Update [CHARTER.md](CHARTER.md) wording ([#823])

[#795]: https://github.com/bottlerocket-os/bottlerocket/pull/795
[#798]: https://github.com/bottlerocket-os/bottlerocket/pull/798
[#799]: https://github.com/bottlerocket-os/bottlerocket/pull/799
[#800]: https://github.com/bottlerocket-os/bottlerocket/pull/800
[#802]: https://github.com/bottlerocket-os/bottlerocket/pull/802
[#803]: https://github.com/bottlerocket-os/bottlerocket/pull/803
[#804]: https://github.com/bottlerocket-os/bottlerocket/pull/804
[#806]: https://github.com/bottlerocket-os/bottlerocket/pull/806
[#807]: https://github.com/bottlerocket-os/bottlerocket/pull/807
[#809]: https://github.com/bottlerocket-os/bottlerocket/pull/809
[#810]: https://github.com/bottlerocket-os/bottlerocket/pull/810
[#817]: https://github.com/bottlerocket-os/bottlerocket/pull/817
[#818]: https://github.com/bottlerocket-os/bottlerocket/pull/818
[#821]: https://github.com/bottlerocket-os/bottlerocket/pull/821
[#823]: https://github.com/bottlerocket-os/bottlerocket/pull/823
[#824]: https://github.com/bottlerocket-os/bottlerocket/pull/824
[#826]: https://github.com/bottlerocket-os/bottlerocket/pull/826
[#827]: https://github.com/bottlerocket-os/bottlerocket/pull/827

# v0.3.0 (2020-02-27)

Welcome to Bottlerocket!
Bottlerocket is the new name for the OS.

In preparation for public preview, v0.3.0 includes a number of breaking changes that mean upgrades from previous versions are not possible.
This is not done lightly, but had to be done to accommodate all we've learned during private preview.

## Breaking Changes

* Rename to Bottlerocket ([#722], [#740]).
* Change partition labels to `BOTTLEROCKET-*` ([#726]).
* Switch to new updates repository URIs under `updates.bottlerocket.aws` ([#778]).
* Update Kubernetes to 1.15 ([#749]).
* Rename aws-k8s variant to aws-k8s-1.15 to enable versioning ([#785]).
* Update Linux kernel to 5.4.16-8.72.amzn2 ([#731]).
* Rename `settings.target-base-url` to `settings.targets-base-url` ([#788]).

## OS Changes

* Mount kernel modules and development headers into containers from a squashfs file on the host ([#701]).
* Include third-party licenses at `/usr/share/licenses` ([#723]).
* Add initial implementation of SELinux ([#683], [#724]).
* Support transactions in the API ([#715], [#727]).
* Add support for platform-specific settings like AWS region ([#636]).
* Support templated settings with new tool 'schnauzer' ([#637]).
* Generate container image URIs with parameterized regions using schnauzer ([#638]).
* Respect update release waves when using `updog check-updates` ([#615]).
* Fix an issue with failed updates through certain https connections ([#730]).
* Add support for EC2 IMDSv2 ([#705], [#706], [#709]).
* Remove update-checking boot service ([#772]).
* Remove old migrations and mitigations that no longer apply ([#774]).
* Add /os API to expose variant, arch, version, etc. ([#777]).
* Update host container packages ([#707]).
* Allow removing settings in migrations ([#644]).
* Create abstractions for creating common migrations ([#712], [#717]).
* Remove the datastore version, instead use Bottlerocket version ([#760]).
* Improve datastore migration naming convention and build migrations during cargo make ([#704], [#716]).
* Update dependencies of third-party packages in base OS ([#691], [#696], [#698], [#699], [#700], [#708], [#728], [#786]).
* Update dependencies of Rust packages ([#738], [#730]).
* Rename `moondog` to `early-boot-config` ([#757]).
* Update admin and control containers to v0.4.0 ([#789]).
* Update container runtime socket path to more common `/run/dockershim.sock` ([#796])

## Documentation

* Add copyright statement and Bottlerocket license ([#746]).
* General documentation improvements ([#681], [#693], [#736], [#761], [#762]).
* Added READMEs for [packages](packages/) and [variants](variants/) ([#773]).
* Split INSTALL guide into BUILDING and QUICKSTART ([#780]).
* Update CNI plugin in documentation and conformance test scripts ([#739]).

## Build Changes

* General improvements to third-party license scanning ([#686], [#719], [#768]).
* Add policycoreutils, secilc, and squashfs-tools to SDK ([#678], [#690]).
* Update to Rust 1.41 and Go 1.13.8 ([#711], [#733]).
* Disallow upstream source fallback by default ([#735]).
* Move host, operator, and SDK containers to their own git repos ([#743], [#751], [#775]).
  * [SDK Container](https://github.com/bottlerocket-os/bottlerocket-sdk)
  * [Admin Container](https://github.com/bottlerocket-os/bottlerocket-admin-container)
  * [Control Container](https://github.com/bottlerocket-os/bottlerocket-control-container)
  * [Bottlerocket Update Operator](https://github.com/bottlerocket-os/bottlerocket-update-operator)
* Improve the syntax of migrations listed in Release.toml ([#687]).
* Add arm64 builds for host-containers ([#694]).
* Build stable image paths using symlinks in `build/latest/` ([#767]).
* Add a `set-migrations` subcommand to the `updata` tool ([#756]).
* Remove `rpm_crashtraceback` tag from go builds ([#779]).
* Rename built artifacts to specify variant before arch ([#776]).
* Update SDK to v0.9.0 ([#790]).
* Fix architecture conditional in glibc spec ([#787]).
* Rename the `workspaces` directory to `sources` and the `workspaces` package to `os`. ([#770]).

[#615]: https://github.com/bottlerocket-os/bottlerocket/pull/615
[#636]: https://github.com/bottlerocket-os/bottlerocket/pull/636
[#637]: https://github.com/bottlerocket-os/bottlerocket/pull/637
[#638]: https://github.com/bottlerocket-os/bottlerocket/pull/638
[#644]: https://github.com/bottlerocket-os/bottlerocket/pull/644
[#678]: https://github.com/bottlerocket-os/bottlerocket/pull/678
[#681]: https://github.com/bottlerocket-os/bottlerocket/pull/681
[#683]: https://github.com/bottlerocket-os/bottlerocket/pull/683
[#686]: https://github.com/bottlerocket-os/bottlerocket/pull/686
[#687]: https://github.com/bottlerocket-os/bottlerocket/pull/687
[#690]: https://github.com/bottlerocket-os/bottlerocket/pull/690
[#691]: https://github.com/bottlerocket-os/bottlerocket/pull/691
[#693]: https://github.com/bottlerocket-os/bottlerocket/pull/693
[#694]: https://github.com/bottlerocket-os/bottlerocket/pull/694
[#696]: https://github.com/bottlerocket-os/bottlerocket/pull/696
[#698]: https://github.com/bottlerocket-os/bottlerocket/pull/698
[#699]: https://github.com/bottlerocket-os/bottlerocket/pull/699
[#700]: https://github.com/bottlerocket-os/bottlerocket/pull/700
[#701]: https://github.com/bottlerocket-os/bottlerocket/pull/701
[#704]: https://github.com/bottlerocket-os/bottlerocket/pull/704
[#705]: https://github.com/bottlerocket-os/bottlerocket/pull/705
[#706]: https://github.com/bottlerocket-os/bottlerocket/pull/706
[#707]: https://github.com/bottlerocket-os/bottlerocket/pull/707
[#708]: https://github.com/bottlerocket-os/bottlerocket/pull/708
[#709]: https://github.com/bottlerocket-os/bottlerocket/pull/709
[#711]: https://github.com/bottlerocket-os/bottlerocket/pull/711
[#712]: https://github.com/bottlerocket-os/bottlerocket/pull/712
[#715]: https://github.com/bottlerocket-os/bottlerocket/pull/715
[#716]: https://github.com/bottlerocket-os/bottlerocket/pull/716
[#717]: https://github.com/bottlerocket-os/bottlerocket/pull/717
[#719]: https://github.com/bottlerocket-os/bottlerocket/pull/719
[#722]: https://github.com/bottlerocket-os/bottlerocket/pull/722
[#723]: https://github.com/bottlerocket-os/bottlerocket/pull/723
[#724]: https://github.com/bottlerocket-os/bottlerocket/pull/724
[#726]: https://github.com/bottlerocket-os/bottlerocket/pull/726
[#727]: https://github.com/bottlerocket-os/bottlerocket/pull/727
[#728]: https://github.com/bottlerocket-os/bottlerocket/pull/728
[#730]: https://github.com/bottlerocket-os/bottlerocket/pull/730
[#731]: https://github.com/bottlerocket-os/bottlerocket/pull/731
[#733]: https://github.com/bottlerocket-os/bottlerocket/pull/733
[#735]: https://github.com/bottlerocket-os/bottlerocket/pull/735
[#736]: https://github.com/bottlerocket-os/bottlerocket/pull/736
[#738]: https://github.com/bottlerocket-os/bottlerocket/pull/738
[#739]: https://github.com/bottlerocket-os/bottlerocket/pull/739
[#740]: https://github.com/bottlerocket-os/bottlerocket/pull/740
[#743]: https://github.com/bottlerocket-os/bottlerocket/pull/743
[#746]: https://github.com/bottlerocket-os/bottlerocket/pull/746
[#749]: https://github.com/bottlerocket-os/bottlerocket/pull/749
[#751]: https://github.com/bottlerocket-os/bottlerocket/pull/751
[#756]: https://github.com/bottlerocket-os/bottlerocket/pull/756
[#757]: https://github.com/bottlerocket-os/bottlerocket/pull/757
[#758]: https://github.com/bottlerocket-os/bottlerocket/pull/758
[#760]: https://github.com/bottlerocket-os/bottlerocket/pull/760
[#761]: https://github.com/bottlerocket-os/bottlerocket/pull/761
[#762]: https://github.com/bottlerocket-os/bottlerocket/pull/762
[#767]: https://github.com/bottlerocket-os/bottlerocket/pull/767
[#768]: https://github.com/bottlerocket-os/bottlerocket/pull/768
[#770]: https://github.com/bottlerocket-os/bottlerocket/pull/770
[#772]: https://github.com/bottlerocket-os/bottlerocket/pull/772
[#773]: https://github.com/bottlerocket-os/bottlerocket/pull/773
[#774]: https://github.com/bottlerocket-os/bottlerocket/pull/774
[#775]: https://github.com/bottlerocket-os/bottlerocket/pull/775
[#776]: https://github.com/bottlerocket-os/bottlerocket/pull/776
[#777]: https://github.com/bottlerocket-os/bottlerocket/pull/777
[#778]: https://github.com/bottlerocket-os/bottlerocket/pull/778
[#779]: https://github.com/bottlerocket-os/bottlerocket/pull/779
[#780]: https://github.com/bottlerocket-os/bottlerocket/pull/780
[#782]: https://github.com/bottlerocket-os/bottlerocket/pull/782
[#785]: https://github.com/bottlerocket-os/bottlerocket/pull/785
[#786]: https://github.com/bottlerocket-os/bottlerocket/pull/786
[#787]: https://github.com/bottlerocket-os/bottlerocket/pull/787
[#788]: https://github.com/bottlerocket-os/bottlerocket/pull/788
[#789]: https://github.com/bottlerocket-os/bottlerocket/pull/789
[#790]: https://github.com/bottlerocket-os/bottlerocket/pull/790
[#796]: https://github.com/bottlerocket-os/bottlerocket/pull/796

# v0.2.1 (2020-01-20)

## OS changes

* Make `signpost` usage clearer to avoid updating into empty partition ([#444]).
* Fix handling of wave bounds in `updog` that could result in seeing an update but not accepting it ([#539]).
* Add support for query parameters in repo requests to allow for basic telemetry ([#542]).
* Enable support for SELinux in OS packages (not yet enforcing) ([#579]).
* Make grub reboot when config or kernel loading fails so it can try other partition sets ([#585]).
* Add support for image "variants" with separate API models ([#578], [#588], [#589], [#591], [#597], [#613], [#625], [#626], [#627], [#653]).
  The default variant is "aws-k8s" for Kubernetes usage, and an "aws-dev" variant can be built that has a local Docker daemon and debug tools.
* Remove unused cri-tools package ([#602]).
* Update Linux kernel to 4.19.75-28.73.amzn2 ([#622]).
* Make containerd.service stop containerd-shims to fix shutdown/reboot delay ([#652]).
* Ensure `updog` only removes known extensions from migration filenames ([#662]).
* Add OS version to "pretty name" so it's visible in console log ([#663]).

## Documentation changes

* Reorganize "getting started" documentation for clarity ([#581]).
* Fix formatting of kube-proxy options in install guide ([#584]).
* Specify compatible cargo-deny version in install guide ([#631]).
* Fix typos and improve clarity of install guide ([#639]).

## Build changes

* Add scripts to ease Kubernetes conformance testing through Sonobuoy ([#530]).
* Add release metadata file to be used in future automation ([#556], [#594]).
* Update dependencies of third-party packages in base OS ([#595]).
* Update dependencies of Rust packages ([#598]).
* Update SDK container to include Rust 1.40.0, GCC 9.2, and other small fixes ([#603], [#628]).
* Fix aarch64 build failure for libcap ([#621]).
* Add initial container definitions and scripts for CI process ([#619], [#624], [#633], [#646], [#647], [#651], [#654], [#658]).

[#444]: ../../pull/444
[#530]: ../../pull/530
[#539]: ../../pull/539
[#542]: ../../pull/542
[#556]: ../../pull/556
[#578]: ../../pull/578
[#579]: ../../pull/579
[#581]: ../../pull/581
[#584]: ../../pull/584
[#585]: ../../pull/585
[#588]: ../../pull/588
[#589]: ../../pull/589
[#591]: ../../pull/591
[#594]: ../../pull/594
[#595]: ../../pull/595
[#597]: ../../pull/597
[#598]: ../../pull/598
[#602]: ../../pull/602
[#603]: ../../pull/603
[#613]: ../../pull/613
[#619]: ../../pull/619
[#621]: ../../pull/621
[#622]: ../../pull/622
[#624]: ../../pull/624
[#625]: ../../pull/625
[#626]: ../../pull/626
[#627]: ../../pull/627
[#628]: ../../pull/628
[#631]: ../../pull/631
[#633]: ../../pull/633
[#639]: ../../pull/639
[#646]: ../../pull/646
[#647]: ../../pull/647
[#651]: ../../pull/651
[#652]: ../../pull/652
[#653]: ../../pull/653
[#654]: ../../pull/654
[#658]: ../../pull/658
[#662]: ../../pull/662
[#663]: ../../pull/663

# v0.2.0 (2019-12-09)

## Breaking changes

* Several settings now have added validation for their contents.  Upgrades from v0.1 that use invalid settings values will result in a broken system.
  * Host container names (e.g. `admin` in `settings.host-containers.admin`) are restricted to ASCII alphanumeric characters and hyphens ([#450]).
  * `settings.kubernetes.api-server`, `settings.updates.metadata-base-url` and `target-base-url`, `settings.host-containers.*.sources`, and `settings.ntp.time-servers` are now validated to be URIs ([#549]).
  * `settings.kubernetes.cluster_name`, `settings.kubernetes.node-labels`, and `settings.kubernetes.node-taints` are now verified to fit Kubernetes naming conventions ([#549]).
  * Most settings values disallow multi-line strings ([#453], [#483]).
* Additional characters are permitted in API keys; for example, dots and slashes in Kubernetes labels. Downgrades from v0.2 that use dots and slashes in API keys will result in a broken system ([#511]).

## OS changes

* Add `dogswatch`, a Kubernetes operator for managing OS upgrades ([#239]).
* More accurately represent data type of update seed ([#430]).
* Retry host container pulls with exponential backoff ([#433]).
* Better model startup dependencies in systemd units ([#442]).
* Enable panic on disk corruption detected with dm_verity ([#445]).
* Add persistent storage for host containers, mapped to `/.bottlerocket/host-containers/[CONTAINER_NAME]` ([#450], [#555]).
* Persist SSH host keys for admin container ([#450]).
* Use admin container v0.2 by default ([#450], [#536]).
* Use control container v0.2 by default ([#472], [#536]).
* Print most critical errors to the console to aid debugging ([#476], [#479], [#546]).
* Update Linux kernel to 4.19.75-27.58.amzn2 ([#478]).
* Updated partitions are marked `successful` after services start ([#481]).
* Kernel config is available at `/proc/config.gz` ([#482]).
* Prepare `tough` for separate release, including:
  * Allow library consumers to override the transport mechanism ([#488]).
  * Merge `tough_schema` back into `tough` ([#496]).
  * Add locking around tough datastore write operations ([#497]).
* Simplify representation of default metadata ([#491]).
* `apiclient` (available via the host containers) exits non-zero on HTTP response errors ([#498]).
* `apiclient` builds as a static binary ([#552]).
* `/proc/kheaders.tar.xz` is enabled in the kernel ([#557]).
* `settings-committer` no longer errors at boot when there are no changes to commit ([#559]).
* `migrator` and `updog` set migrations executable before running to work around a v0.1.6 bug ([#561], [#567]).

## Documentation changes

* Document how to use Bottlerocket's default for the `nf_conntrack_max` kernel parameter when using `kube-proxy` ([#391]).
* Fix example user data for enabling admin container ([#448]).
* Update build documentation for using Docker instead of `buildkitd` ([#506]).
* Update recommended CNI plugin version ([#507]).
* Document `settings.ntp.time-servers` ([#550]).
* Update INSTALL.md to use the instance role created by `eksctl` instead of creating a new one ([#569]).

## Build changes

* Add `updata` tool, which builds update repository metadata ([#265]).
* Create versioned symlinks to output images ([#434]).
* Add code and CloudFormation template for TUF repository canary ([#490]).
* Move the TUF client library, `tough`, to [its own repository](https://github.com/awslabs/tough) and [crates.io packages](https://crates.io/crates/tough) ([#499]).
* Remove build dependency on the BuildKit daemon ([#506]).
* Switch to SDK container as toolchain for builds, rather than requiring local build of toolchain ([#525]).
* Turn `buildsys` into a binary and remove the `cascade` feature ([#562]).

[#239]: ../../pull/239
[#265]: ../../pull/265
[#391]: ../../pull/391
[#430]: ../../pull/430
[#433]: ../../pull/433
[#434]: ../../pull/434
[#442]: ../../pull/442
[#445]: ../../pull/445
[#448]: ../../pull/448
[#450]: ../../pull/450
[#453]: ../../pull/453
[#472]: ../../pull/472
[#476]: ../../pull/476
[#478]: ../../pull/478
[#479]: ../../pull/479
[#481]: ../../pull/481
[#482]: ../../pull/482
[#483]: ../../pull/483
[#488]: ../../pull/488
[#490]: ../../pull/490
[#491]: ../../pull/491
[#496]: ../../pull/496
[#497]: ../../pull/497
[#498]: ../../pull/498
[#499]: ../../pull/499
[#506]: ../../pull/506
[#507]: ../../pull/507
[#511]: ../../pull/511
[#525]: ../../pull/525
[#536]: ../../pull/536
[#546]: ../../pull/546
[#549]: ../../pull/549
[#550]: ../../pull/550
[#552]: ../../pull/552
[#555]: ../../pull/555
[#557]: ../../pull/557
[#559]: ../../pull/559
[#561]: ../../pull/561
[#562]: ../../pull/562
[#567]: ../../pull/567
[#569]: ../../pull/569

# v0.1.6 (2019-10-21)

## OS changes

* The system fetches the pause container from ECR before starting `kubelet` ([#382]).
* New settings: `settings.kubernetes.node-labels` and `settings.kubernetes.node-taints` ([#390], [#408]).
* The control container has an `enable-admin-container` helper ([#405], [#413]). Made default in v0.2.0 ([#472]).
* Rust dependencies updated ([#410]).
* `thar-be-settings` added trace-level messages in the client module ([#411]).
* `updog` no longer checks for migrations from new root images ([#416]).
* `pluto` was cleaned up to create an HTTP connection more consistently ([#419]).
* Settings that are usually generated may have defaults, and `settings.kubernetes.max-pods` defaults to `110` if the EC2 instance type cannot be determined ([#420]).
* The admin container MOTD is clearer about where the host's filesystem is mounted ([#424]).
* `block-party` (used in `growpart` and `signpost`) errors are better structured ([#425]).
* `thar-be-settings` logs render errors when running in `--all` mode ([#427]).
* [Recommended `sysctl` settings from the Kernel Self Protection Project](https://kernsec.org/wiki/index.php/Kernel_Self_Protection_Project/Recommended_Settings#sysctls) are now used ([#435]).
* `acpid` is enabled by default to handle power button signals sent by EC2 on stop/restart/terminate events ([#437]).
* `host-ctr` correctly fetches images from non-ECR registries ([#439]; this regression occurred after v0.1.5).

## Build changes

* amiize uses a short connection timeout when testing SSH connectivity ([#409]).
* `tuftool` only downloads an arbitrary `root.json` with `--allow-root-download` ([#421]).
* BuildKit updated to v0.6.2 ([#423], [#429]).
* First-party Rust code is built in the same `rpmbuild` invocation to improve build times ([#428]).
* `tuftool` correctly uses the `--timestamp-{version,expires}` arguments instead of the `--snapshot-{version,expires}` arguments in the timestamp role ([#438]).
* `tuftool` accepts relative dates ([#438]).

## Documentation changes

* The `sources/updater` crates are better documented ([#381]).
* INSTALL.md's subnet selection documentation is improved ([#422]).

[#381]: ../../pull/381
[#382]: ../../pull/382
[#390]: ../../pull/390
[#405]: ../../pull/405
[#408]: ../../pull/408
[#409]: ../../pull/409
[#410]: ../../pull/410
[#411]: ../../pull/411
[#413]: ../../pull/413
[#416]: ../../pull/416
[#419]: ../../pull/419
[#420]: ../../pull/420
[#421]: ../../pull/421
[#422]: ../../pull/422
[#423]: ../../pull/423
[#424]: ../../pull/424
[#425]: ../../pull/425
[#427]: ../../pull/427
[#428]: ../../pull/428
[#429]: ../../pull/429
[#435]: ../../pull/435
[#437]: ../../pull/437
[#438]: ../../pull/438
[#439]: ../../pull/439
