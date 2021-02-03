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
