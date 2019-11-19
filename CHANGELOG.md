# v0.2.0 (2019-11-??)

## Breaking changes

* Host container names (e.g. `admin` in `settings.host-containers.admin`) are restricted to ASCII alphanumeric characters and hyphens. Upgrades from v0.1 that use additional characters in host container names will result in a broken system ([#450]).
* Most settings values disallow multi-line strings. Upgrades from v0.1 that use multi-line strings in settings values will result in a broken system ([#453], [#483]).
* Additional characters are permitted in API keys; for example, dots and slashes in Kubernetes labels. Downgrades from v0.2 that use dots and slashes in API keys will result in a broken system ([#511]).

## OS changes

* Add `dogswatch`, a Kubernetes operator for managing OS upgrades ([#239]).
* More accurately represent data type of update seed ([#430]).
* Retry host container pulls with exponential backoff ([#433]).
* Better model startup dependencies in systemd units ([#442]).
* Enable panic on disk corruption detected with dm_verity ([#445]).
* Add persistent storage for host containers, mapped to `/.thar/host-containers/[CONTAINER_NAME]` ([#450]).
* Persist SSH host keys for admin container ([#450]).
* Use admin container v0.2 by default ([#450], [#536]).
* Use control container v0.2 by default ([#472], [#536]).
* Print most critical errors to the console to aid debugging ([#476], [#479]).
* Update Linux kernel to 4.19.75-27.58.amzn2 ([#478]).
* Updated partitions are marked `successful` after services start ([#481]).
* Kernel config is available at `/proc/config.gz` ([#482]).
* Prepare `tough` for separate release, including:
  * Allow library consumers to override the transport mechanism ([#488]).
  * Merge `tough_schema` back into `tough` ([#496]).
  * Add locking around tough datastore write operations ([#497]).
* Simplify representation of default metadata ([#491]).
* `apiclient` (available via the host containers) exits non-zero on HTTP response errors ([#498]).

## Documentation changes

* Fix example user data for enabling admin container ([#448]).
* Update build documentation for using Docker instead of `buildkitd` ([#506]).
* Update recommended CNI plugin version ([#507]).

## Build changes

* Add `updata` tool, which builds update repository metadata ([#265]).
* Create versioned symlinks to output images ([#434]).
* Introduce infrastructure for fetching external sources ([#485], [#523]).
* Add code and CloudFormation template for TUF repository canary ([#490]).
* Move the TUF client library, `tough`, to [its own repository](https://github.com/awslabs/tough) and [crates.io packages](https://crates.io/crates/tough) ([#499]).
* Remove build dependency on the BuildKit daemon ([#506]).

[#239]: ../../pull/239
[#265]: ../../pull/265
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
[#523]: ../../pull/523
[#536]: ../../pull/536

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
