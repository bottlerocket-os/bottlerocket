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
* Add persistent storage for host containers, mapped to `/.thar/host-containers/[CONTAINER_NAME]` ([#450], [#555]).
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

* Document how to use Thar's default for the `nf_conntrack_max` kernel parameter when using `kube-proxy` ([#391]).
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

* The `workspaces/updater` crates are better documented ([#381]).
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
