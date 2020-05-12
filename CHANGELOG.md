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
