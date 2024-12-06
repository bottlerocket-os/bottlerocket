# v1.28.0 (2024-12-08)

## Release Highlights
* Enable EFA support to Bottlerocket AMIs ([#4290])
* Fix `io_uring` regression in 6.1 kernel ([bottlerocket-core-kit#284](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/284))
* Allow overriding the max-pods file with one from your variant ([bottlerocket-core-kit#279](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/279)) - thanks @tzneal

## Build Changes
* Update bottlerocket-core-kit to 4.0.1 ([#4322])

### OS Changes
* Update host containers ([#4312])
* Update Twoliter to 0.6.0 ([#4323])

### Documentation Changes
* Update models README references ([#4138])

[#4138]: https://github.com/bottlerocket-os/bottlerocket/pull/4138
[#4290]: https://github.com/bottlerocket-os/bottlerocket/pull/4290
[#4312]: https://github.com/bottlerocket-os/bottlerocket/pull/4312
[#4322]: https://github.com/bottlerocket-os/bottlerocket/pull/4322
[#4323]: https://github.com/bottlerocket-os/bottlerocket/pull/4323

# v1.27.1 (2024-11-16)

## Release Highlights
* Add patch for kernel-5.15 to fix issues when using IPv6 ([bottlerocket-core-kit#266](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/266))

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to 3.3.2 ([#4301])

[#4301]: https://github.com/bottlerocket-os/bottlerocket/pull/4301

# v1.27.0 (2024-11-12)

## Release Highlights
* Add FIPS variants ([#4274], [#1667], [#4267])
* Drop k8s 1.28 and k8s 1.29 metal variants ([#4287])

## OS Changes
* Add aws-creds settings defaults to all AWS variants ([#4285])
* Add support for migrations to modify aws-config setting generators ([#4271])

## Build Changes
* Update bottlerocket-core-kit to 3.3.0 ([#4292])
* Update bottlerocket-sdk to 0.47.0 ([#4286])

[#1667]: https://github.com/bottlerocket-os/bottlerocket/pull/1667
[#4267]: https://github.com/bottlerocket-os/bottlerocket/pull/4267
[#4271]: https://github.com/bottlerocket-os/bottlerocket/pull/4271
[#4274]: https://github.com/bottlerocket-os/bottlerocket/pull/4274
[#4285]: https://github.com/bottlerocket-os/bottlerocket/pull/4285
[#4286]: https://github.com/bottlerocket-os/bottlerocket/pull/4286
[#4287]: https://github.com/bottlerocket-os/bottlerocket/pull/4287
[#4292]: https://github.com/bottlerocket-os/bottlerocket/pull/4292

# v1.26.2 (2024-11-04)

## Release Highlights
* Wait for kubelet device-manager socket before starting nvidia-k8s-device-plugin ([bottlerocket-core-kit#238](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/238))

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to 3.1.5 ([#4280])

[#4280]: https://github.com/bottlerocket-os/bottlerocket/pull/4280

# v1.26.1 (2024-10-24)

## Release Highlights
* Revert system-wide configuration to block writeable/executable memory in systemd services ([bottlerocket-core-kit#215](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/215))

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to 3.1.1 ([#4264])

[#4264]: https://github.com/bottlerocket-os/bottlerocket/pull/4264

# v1.26.0 (2024-10-23)

## Release Highlights
* Update NVIDIA driver to 535.216.01 ([#4254])
* Move kmod-5.10-nvidia tesla package for aws-ecs-1-nvidia variant from branch R470 to R535 ([#4251])

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to 3.1.0 ([#4254], [#4251])
* Update NVIDIA driver to 535.216.01 ([#4254])
* Update twoliter to 0.5.0 ([#4251])
* Update bottlerocket-sdk to 0.46 ([#4251])
* Standardize RPM release fields for RPM packages ([#4244])

## Orchestrator Changes

### ECS
* Move kmod-5.10-nvidia tesla package for aws-ecs-1-nvidia variant from branch R470 to R535 ([#4251])

### Documentation Changes
* Add link to bootstrap-commands documentation ([#4247])

[#4244]: https://github.com/bottlerocket-os/bottlerocket/pull/4244
[#4247]: https://github.com/bottlerocket-os/bottlerocket/pull/4247
[#4251]: https://github.com/bottlerocket-os/bottlerocket/pull/4251
[#4254]: https://github.com/bottlerocket-os/bottlerocket/pull/4254

# v1.25.0 (2024-10-15)

## Release Highlights
* Remove aws-k8s-1.23 variants (https://github.com/bottlerocket-os/bottlerocket/issues/4083)
* Add support for NVIDIA GPU time slicing (closes https://github.com/bottlerocket-os/bottlerocket/issues/2347)

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to 2.9.0 ([#4242])
* Update host containers ([#4241])
* Update twoliter to v0.4.7 ([#4236])
* Fix permissions for kubelet-exec-start-conf file ([#4199])
* Add support for NVIDIA GPU time slicing ([#4230])

## Orchestrator Changes

### Kubernetes
* Drop Kubernetes 1.23 AWS variants ([#4227], [#4237])

### Documentation Changes
* Add security guidance for NVIDIA GPU time-slicing ([#4240])

[#4199]: https://github.com/bottlerocket-os/bottlerocket/pull/4199
[#4227]: https://github.com/bottlerocket-os/bottlerocket/pull/4227
[#4230]: https://github.com/bottlerocket-os/bottlerocket/pull/4230
[#4236]: https://github.com/bottlerocket-os/bottlerocket/pull/4236
[#4237]: https://github.com/bottlerocket-os/bottlerocket/pull/4237
[#4240]: https://github.com/bottlerocket-os/bottlerocket/pull/4240
[#4241]: https://github.com/bottlerocket-os/bottlerocket/pull/4241
[#4242]: https://github.com/bottlerocket-os/bottlerocket/pull/4242

# v1.24.1 (2024-10-04)

## Release Highlights
* Update ecs-agent to 1.86.3 ([bottlerocket-core-kit#168](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/168)) - Closes issue [#4186](https://github.com/bottlerocket-os/bottlerocket/issues/4186)

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to 2.8.4 ([#4231])
* Update host containers ([#4233])

### Documentation Changes
* Update QUICKSTART-EKS.md ([#4228]) - Thanks @bryanhsu00 for the suggested fix!

[#4228]: https://github.com/bottlerocket-os/bottlerocket/pull/4228
[#4231]: https://github.com/bottlerocket-os/bottlerocket/pull/4231
[#4233]: https://github.com/bottlerocket-os/bottlerocket/pull/4233

# v1.24.0 (2024-09-27)

## Release Highlights
* Use open GPU drivers on P4 and P5 instances automatically [bottlerocket-core-kit#114](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/114)
* Update to nvidia-container-toolkit 1.16.2 [bottlerocket-core-kit#161](https://github.com/bottlerocket-os/bottlerocket-core-kit/pull/161)

## Build Changes

### OS Changes
* Update bottlerocket-core-kit to v2.8.1 ([#4222])

### Settings Extensions
* Drop dependency on glibc-devel ([#4213])

### Documentation Changes
* Update QUICKSTART-ECS.md and QUICKSTART-EKS.md ([#4169])  Thanks @bryantbiggs!

[#4169]: https://github.com/bottlerocket-os/bottlerocket/pull/4169
[#4213]: https://github.com/bottlerocket-os/bottlerocket/pull/4213
[#4222]: https://github.com/bottlerocket-os/bottlerocket/pull/4222

# v1.23.0 (2024-09-19)

## Orchestrator Changes

### Kubernetes
* Support Kubernetes NVIDIA Device Plugin configurations through API ([#4182])
* Support NVIDIA Container Toolkit configurations through API ([#4182])

## Build Changes
* Update bottlerocket-sdk to 0.45 ([#4189])
* Add `Twoliter.override` to `.gitignore` ([#4202])

### Twoliter
* Update bottlerocket-core-kit ([#4189], [#4203], [#4211])
* Perform binary checksum validation ([#4192])
* Update Twoliter to v0.4.6 ([#4200])

### Settings Extensions
* Update bottlerocket-settings-models to v0.4.0 ([#4182])

### Documentation Changes
* Add NVIDIA Device Plugin and NVIDIA Container Toolkit notes to SECURITY_GUIDANCE.md ([#4205])

[#4182]: https://github.com/bottlerocket-os/bottlerocket/pull/4182
[#4189]: https://github.com/bottlerocket-os/bottlerocket/pull/4189
[#4192]: https://github.com/bottlerocket-os/bottlerocket/pull/4192
[#4200]: https://github.com/bottlerocket-os/bottlerocket/pull/4200
[#4202]: https://github.com/bottlerocket-os/bottlerocket/pull/4202
[#4203]: https://github.com/bottlerocket-os/bottlerocket/pull/4203
[#4205]: https://github.com/bottlerocket-os/bottlerocket/pull/4205
[#4211]: https://github.com/bottlerocket-os/bottlerocket/pull/4211

# v1.22.0 (2024-09-10)

## Orchestrator Changes

### Kubernetes
* Add Kubernetes 1.31 variants ([#4142])

## OS Changes
* Update host containers ([#4171])
* Add support for bootstrap commands ([#4131])

## Build Changes

### Twoliter
* Update bottlerocket-core-kit to v2.4.1 ([#4183], [#4177], [#4168])

### Settings Extensions
* Update bottlerocket-settings-models to v0.4.0 ([#4131])

[#4131]: https://github.com/bottlerocket-os/bottlerocket/pull/4131
[#4142]: https://github.com/bottlerocket-os/bottlerocket/pull/4142
[#4168]: https://github.com/bottlerocket-os/bottlerocket/pull/4168
[#4171]: https://github.com/bottlerocket-os/bottlerocket/pull/4171
[#4177]: https://github.com/bottlerocket-os/bottlerocket/pull/4177
[#4183]: https://github.com/bottlerocket-os/bottlerocket/pull/4183

# v1.21.1 (2024-08-21)

## OS Changes

* Update host containers ([#4153])

## Build Changes
* Use workspace dependencies for all dependencies ([#4132])

### Twoliter
* Update bottlerocket-core-kit to v2.3.5 ([#4156], [#4152], [#4143], [#4139])
* Update Twoliter to v0.4.5 ([#4159])

### Settings Extensions
* Update bottlerocket-settings-models to v0.3.0 ([#4145])

## README changes
* Update command for SSM Start session on host container ([#4129]) - Thanks @Veronica4036!

[#4129]: https://github.com/bottlerocket-os/bottlerocket/pull/4129
[#4132]: https://github.com/bottlerocket-os/bottlerocket/pull/4132
[#4139]: https://github.com/bottlerocket-os/bottlerocket/pull/4139
[#4143]: https://github.com/bottlerocket-os/bottlerocket/pull/4143
[#4145]: https://github.com/bottlerocket-os/bottlerocket/pull/4145
[#4152]: https://github.com/bottlerocket-os/bottlerocket/pull/4152
[#4153]: https://github.com/bottlerocket-os/bottlerocket/pull/4153
[#4156]: https://github.com/bottlerocket-os/bottlerocket/pull/4156
[#4159]: https://github.com/bottlerocket-os/bottlerocket/pull/4159

# v1.21.0 (2024-08-06)

## OS Changes

* Update host containers ([#4117])

## Orchestrator Changes

### Kubernetes

* Enable k8s reserved cpus ([#3964])
* Drop k8s 1.27 metal and VMware variants ([#4079])
* Drop k8s 1.26 metal and VMware variants ([#4018])
* Build the pause image from upstream ([#3940]) - Thanks @tzneal!

### ECS

* Port to the ECS settings extension ([#3984])

## Build Changes

* Archive previous release migrations ([#4014])
* Update Go dependencies ([#3999])

### Twoliter

* Migrate to core kit ([#4060])
* Remove leftover vendor section ([#4071])
* Update Twoliter to 0.4.4 ([#4008], [#4086], [#4093], [#4123])
* Update bottlerocket-core-kit to v2.3.1 ([#4122])
* Update bottlerocket-sdk to 0.43 ([#4122])

### Settings Extensions

* Use settings models vended by bottlerocket-settings-sdk ([#4057])
* Migrate to settings plugins and eliminate variant-based conditional compilation ([#4038])
* Enable settings extensions ([#4050])
* Update to bottlerocket-settings-models v0.2.0 ([#4118])


## Platform Changes

### AWS

* Add udev rule to create symlinks using EBS volumes’ device names ([#3977])

## Package changes

* Add Neuron kmod for 6.1 kernel ([#3982])
* Update containerd to 1.7.20 ([#4122])

## README changes

* Fix OpenAPI spec link ([#4062])
* Fix NVIDIA variants in SSM parameters ([#4047])
* Add k8s command to retrieve log archive ([#3993])
* Fix netdog reference link ([#3974]) - Thanks @emmanuel-ferdman!
* Update BUILDING.md with the latest Docker requirements ([#4098])

[#3940]: https://github.com/bottlerocket-os/bottlerocket/pull/3940
[#3964]: https://github.com/bottlerocket-os/bottlerocket/pull/3964
[#3974]: https://github.com/bottlerocket-os/bottlerocket/pull/3974
[#3977]: https://github.com/bottlerocket-os/bottlerocket/pull/3977
[#3982]: https://github.com/bottlerocket-os/bottlerocket/pull/3982
[#3984]: https://github.com/bottlerocket-os/bottlerocket/pull/3984
[#3993]: https://github.com/bottlerocket-os/bottlerocket/pull/3993
[#3999]: https://github.com/bottlerocket-os/bottlerocket/pull/3999
[#4008]: https://github.com/bottlerocket-os/bottlerocket/pull/4008
[#4014]: https://github.com/bottlerocket-os/bottlerocket/pull/4014
[#4018]: https://github.com/bottlerocket-os/bottlerocket/pull/4018
[#4027]: https://github.com/bottlerocket-os/bottlerocket/pull/4027
[#4038]: https://github.com/bottlerocket-os/bottlerocket/pull/4038
[#4047]: https://github.com/bottlerocket-os/bottlerocket/pull/4047
[#4050]: https://github.com/bottlerocket-os/bottlerocket/pull/4050
[#4057]: https://github.com/bottlerocket-os/bottlerocket/pull/4057
[#4060]: https://github.com/bottlerocket-os/bottlerocket/pull/4060
[#4062]: https://github.com/bottlerocket-os/bottlerocket/pull/4062
[#4071]: https://github.com/bottlerocket-os/bottlerocket/pull/4071
[#4079]: https://github.com/bottlerocket-os/bottlerocket/pull/4079
[#4086]: https://github.com/bottlerocket-os/bottlerocket/pull/4086
[#4093]: https://github.com/bottlerocket-os/bottlerocket/pull/4093
[#4098]: https://github.com/bottlerocket-os/bottlerocket/pull/4098
[#4117]: https://github.com/bottlerocket-os/bottlerocket/pull/4117
[#4118]: https://github.com/bottlerocket-os/bottlerocket/pull/4118
[#4122]: https://github.com/bottlerocket-os/bottlerocket/pull/4122
[#4123]: https://github.com/bottlerocket-os/bottlerocket/pull/4123

# v1.20.5 (2024-07-30)

## OS Changes

* Update docker-engine to v25.0.6 ([#4111])
* Update containerd to 1.6.34 ([#4113])
* Update kernels: 5.10.220, 5.15.162, and 6.1.97 ([#4104])
* Update host containers ([#4110])

## Orchestrator Changes

### Kubernetes

* Add latest instance types to eni-max-pods mapping ([#4108])

[#4104]: https://github.com/bottlerocket-os/bottlerocket/pull/4104
[#4108]: https://github.com/bottlerocket-os/bottlerocket/pull/4108
[#4110]: https://github.com/bottlerocket-os/bottlerocket/pull/4110
[#4111]: https://github.com/bottlerocket-os/bottlerocket/pull/4111
[#4113]: https://github.com/bottlerocket-os/bottlerocket/pull/4113

# v1.20.4 (2024-07-15)

## OS Changes
* Update kernels: 5.10.219 and 6.1.94 ([#4080])
* Update docker-engine and docker-cli to v25.0.5 ([#4091])

## Orchestrator Changes

### Kubernetes
* Update patches for kubernetes 1.23, 1.24, 1.25, and 1.26 ([#4084])
* Update sources for kubernetes 1.27, 1.28, 1.29, and 1.30 ([#4089])

[#4080]: https://github.com/bottlerocket-os/bottlerocket/pull/4080
[#4084]: https://github.com/bottlerocket-os/bottlerocket/pull/4084
[#4089]: https://github.com/bottlerocket-os/bottlerocket/pull/4089
[#4091]: https://github.com/bottlerocket-os/bottlerocket/pull/4091

# v1.20.3 (2024-06-26)

## OS Changes
* Update kernels: 5.10.218, 5.15.160, and 6.1.92 ([#4064], [#4066])

[#4064]: https://github.com/bottlerocket-os/bottlerocket/pull/4064
[#4066]: https://github.com/bottlerocket-os/bottlerocket/pull/4066

# v1.20.2 (2024-06-12)

## OS Changes
* Update kernel to 5.10.217 [#4039]
* Mount static kmod as /usr/local/sbin/modprobe [#4037]

[#4037]: https://github.com/bottlerocket-os/bottlerocket/pull/4037
[#4039]: https://github.com/bottlerocket-os/bottlerocket/pull/4039

# v1.20.1 (2024-06-04)

## OS Changes
* Update kernels to 6.1.90, 5.15.158, and 5.10.216 ([#3976], [#3972])
* Include statically linked version of kmod ([#3981])
* Specify AWS EULA as license for kmod-*-nvidia packages ([#3991])
* Update source for Fabric Manager binaries ([#4015])
* Update NVIDIA driver versions to 470.256.02 and 535.183.01 ([#4029])

[#3972]: https://github.com/bottlerocket-os/bottlerocket/pull/3972
[#3976]: https://github.com/bottlerocket-os/bottlerocket/pull/3976
[#3981]: https://github.com/bottlerocket-os/bottlerocket/pull/3981
[#3991]: https://github.com/bottlerocket-os/bottlerocket/pull/3991
[#4015]: https://github.com/bottlerocket-os/bottlerocket/pull/4015
[#4029]: https://github.com/bottlerocket-os/bottlerocket/pull/4029

# v1.20.0 (2024-05-13)

## OS Changes
* Update third party packages ([#3939])
* Enable file system encryption in 5.15 and 6.1 kernels ([#3906], [#3908])
* Backport fix for loading SELinux modules ([#3907])
* Add Fabric Manager support ([#3873])
* Update host containers ([#3947])
* Add setting to configure ntp options ([#3852] thanks @domgoodwin)
* Include swap utilities ([#3829])
* Update kernels to 6.1.87, 5.15.156, 5.10.215 ([#3934], [#3930])

## Orchestrator Changes

### Kubernetes
* Drop Kubernetes 1.25 Metal and VMware variants ([#3896])
* Add Kubernetes 1.30 variants ([#3859], [#3936])
* Add container-runtime settings to `aws-k8s-*-nvidia` variants ([#3945])

### ECS
* Update ecs-agent to 1.82.3 ([#3939])
* Use systemd drop-ins to configure the ECS agent ([#3834])

## Build Changes
* Update twoliter and the SDK ([#3938], [#3885])
* Remove liblzma and libbzip2 ([#3861], [#3944])
* Pessimize Rust builds that require the AWS SDK ([#3892])
* Reduce variant matrix in CI/CD ([#3863])
* Document package build tools for go dependencies ([#3882])
* Update Go lints in CI/CD ([#3884])
* Out-of-tree build enablement
  * systemd: use build defaults and kernel parameters for unified cgroups ([#3886], [#3935])
  * early-boot-config: Use standalone provider binaries to fetch user data ([#3637], [#3890])
  * logdog: retrieve settings via API client ([#3946])
  * netdog: remove conditional compilation, add hostname helpers ([#3700], [#3898])
  * schnauzer: add if_not_null template helper ([#3838])
  * static-pods: remove conditional compilation, switch to config file ([#3891], [#3927], [#3913])
  * host-containers: switch to config file ([#3777], [#3842])
  * bootstrap-containers: switch to config file ([#3724])
  * corndog: switch to config file ([#3715])
  * prairiedog: switch to config file ([#3713], [#3814], [#3836])
  * thar-be-updates: switch to config file ([#3721])
  * updog: use modeled types ([#3901])
  * kernel: remove variant sensitivity ([#3897], [#3905], [#3932])
* FIPS enablement
  * add FIPS report to the API ([#3894])
  * add release-fips package for FIPS functionality ([#3893])
  * build Go binaries for FIPS and non-FIPS ([#3887])

[#3637]: https://github.com/bottlerocket-os/bottlerocket/pull/3637
[#3700]: https://github.com/bottlerocket-os/bottlerocket/pull/3700
[#3713]: https://github.com/bottlerocket-os/bottlerocket/pull/3713
[#3715]: https://github.com/bottlerocket-os/bottlerocket/pull/3715
[#3721]: https://github.com/bottlerocket-os/bottlerocket/pull/3721
[#3724]: https://github.com/bottlerocket-os/bottlerocket/pull/3724
[#3777]: https://github.com/bottlerocket-os/bottlerocket/pull/3777
[#3814]: https://github.com/bottlerocket-os/bottlerocket/pull/3814
[#3829]: https://github.com/bottlerocket-os/bottlerocket/pull/3829
[#3834]: https://github.com/bottlerocket-os/bottlerocket/pull/3834
[#3836]: https://github.com/bottlerocket-os/bottlerocket/pull/3836
[#3838]: https://github.com/bottlerocket-os/bottlerocket/pull/3838
[#3842]: https://github.com/bottlerocket-os/bottlerocket/pull/3842
[#3852]: https://github.com/bottlerocket-os/bottlerocket/pull/3852
[#3859]: https://github.com/bottlerocket-os/bottlerocket/pull/3859
[#3861]: https://github.com/bottlerocket-os/bottlerocket/pull/3861
[#3863]: https://github.com/bottlerocket-os/bottlerocket/pull/3863
[#3873]: https://github.com/bottlerocket-os/bottlerocket/pull/3873
[#3882]: https://github.com/bottlerocket-os/bottlerocket/pull/3882
[#3884]: https://github.com/bottlerocket-os/bottlerocket/pull/3884
[#3885]: https://github.com/bottlerocket-os/bottlerocket/pull/3885
[#3886]: https://github.com/bottlerocket-os/bottlerocket/pull/3886
[#3887]: https://github.com/bottlerocket-os/bottlerocket/pull/3887
[#3890]: https://github.com/bottlerocket-os/bottlerocket/pull/3890
[#3891]: https://github.com/bottlerocket-os/bottlerocket/pull/3891
[#3892]: https://github.com/bottlerocket-os/bottlerocket/pull/3892
[#3893]: https://github.com/bottlerocket-os/bottlerocket/pull/3893
[#3894]: https://github.com/bottlerocket-os/bottlerocket/pull/3894
[#3896]: https://github.com/bottlerocket-os/bottlerocket/pull/3896
[#3897]: https://github.com/bottlerocket-os/bottlerocket/pull/3897
[#3898]: https://github.com/bottlerocket-os/bottlerocket/pull/3898
[#3901]: https://github.com/bottlerocket-os/bottlerocket/pull/3901
[#3905]: https://github.com/bottlerocket-os/bottlerocket/pull/3905
[#3906]: https://github.com/bottlerocket-os/bottlerocket/pull/3906
[#3907]: https://github.com/bottlerocket-os/bottlerocket/pull/3907
[#3908]: https://github.com/bottlerocket-os/bottlerocket/pull/3908
[#3913]: https://github.com/bottlerocket-os/bottlerocket/pull/3913
[#3927]: https://github.com/bottlerocket-os/bottlerocket/pull/3927
[#3930]: https://github.com/bottlerocket-os/bottlerocket/pull/3930
[#3932]: https://github.com/bottlerocket-os/bottlerocket/pull/3932
[#3934]: https://github.com/bottlerocket-os/bottlerocket/pull/3934
[#3935]: https://github.com/bottlerocket-os/bottlerocket/pull/3935
[#3936]: https://github.com/bottlerocket-os/bottlerocket/pull/3936
[#3938]: https://github.com/bottlerocket-os/bottlerocket/pull/3938
[#3939]: https://github.com/bottlerocket-os/bottlerocket/pull/3939
[#3944]: https://github.com/bottlerocket-os/bottlerocket/pull/3944
[#3945]: https://github.com/bottlerocket-os/bottlerocket/pull/3945
[#3946]: https://github.com/bottlerocket-os/bottlerocket/pull/3946
[#3947]: https://github.com/bottlerocket-os/bottlerocket/pull/3947


# v1.19.5 (2024-05-01)

## OS Changes
* Update kernel to 5.10.214, 5.15.153, 6.1.84 [#3906]
* Update third party packages ([#3910], [#3914])
* Update host containers (#[3911])

## Orchestrator Changes

### Kubernetes
* Provide runtime cgroup to kubelet ([#3804])

## Build Changes
* Update twoliter to v0.1.1 ([#3880], [#3900])
* Update ecs-gpu-init, amazon-ssm-agent, and nvidia-k8s-device-plugin builds for new SDK ([#3920], [#3921], [#3924])

[#3804]: https://github.com/bottlerocket-os/bottlerocket/pull/3804
[#3880]: https://github.com/bottlerocket-os/bottlerocket/pull/3880
[#3900]: https://github.com/bottlerocket-os/bottlerocket/pull/3900
[#3906]: https://github.com/bottlerocket-os/bottlerocket/pull/3906
[#3910]: https://github.com/bottlerocket-os/bottlerocket/pull/3910
[#3911]: https://github.com/bottlerocket-os/bottlerocket/pull/3911
[#3914]: https://github.com/bottlerocket-os/bottlerocket/pull/3914
[#3920]: https://github.com/bottlerocket-os/bottlerocket/pull/3920
[#3921]: https://github.com/bottlerocket-os/bottlerocket/pull/3921
[#3924]: https://github.com/bottlerocket-os/bottlerocket/pull/3924

# v1.19.4 (2024-04-06)

## OS Changes
* Update kernel to 5.10.213, 5.15.152, 6.1.82 ([#3865])
* Update containerd to 1.6.31 ([#3869])

[#3865]: https://github.com/bottlerocket-os/bottlerocket/pull/3865
[#3869]: https://github.com/bottlerocket-os/bottlerocket/pull/3869

# v1.19.3 (2024-03-26)

## OS Changes
* Update kernel to 5.10.210, 5.15.149, 6.1.79 ([#3853])
* Update third party packages ([#3793], [#3832])
* Update host containers ([#3837])
* Support auditctl in bootstrap containers ([#3831])

## Orchestrator Changes

### Kubernetes
* Add latest instance types to eni-max-pods mapping ([#3824])

### ECS

## Build Changes
* Update Rust dependencies ([#3830])
* Update Go dependencies ([#3830])
* twoliter updated to v0.0.7 ([#3839])

[#3793]: https://github.com/bottlerocket-os/bottlerocket/pull/3793
[#3824]: https://github.com/bottlerocket-os/bottlerocket/pull/3824
[#3832]: https://github.com/bottlerocket-os/bottlerocket/pull/3832
[#3830]: https://github.com/bottlerocket-os/bottlerocket/pull/3830
[#3831]: https://github.com/bottlerocket-os/bottlerocket/pull/3831
[#3837]: https://github.com/bottlerocket-os/bottlerocket/pull/3837
[#3839]: https://github.com/bottlerocket-os/bottlerocket/pull/3839
[#3853]: https://github.com/bottlerocket-os/bottlerocket/pull/3853

# v1.19.2 (2024-02-26)

## OS Changes
* Update third party packages ([#3789])
* Update kernel to 5.10.209, 5.15.148, 6.1.77 ([#3797])
* Add AWS settings extension ([#3738], [#3770])
* Allow CSI helpers in the SELinux policy ([#3779])
* Update to latest NVIDIA drivers ([#3798])

## Orchestrator Changes

### Kubernetes
* Enable NVIDIA GPU isolation using volume mounts ([#3718] thanks @chiragjn , [#3790])
* Clean up CNI results cache on boot ([#3792])

### ECS
* Add `settings.ecs.enable-container-metadata` ([#3782])

## Build Changes
* Adjust certdog to utilize a configuration file instead of the API server ([#3706], [#3778], [#3787])
* Don't use parallel make for shim package ([#3771])
* Renumber unit files in release package ([#3769])
* Ignore EKS patches for k8s-1.23 in Git ([#3774])

[#3706]: https://github.com/bottlerocket-os/bottlerocket/pull/3706
[#3718]: https://github.com/bottlerocket-os/bottlerocket/pull/3718
[#3738]: https://github.com/bottlerocket-os/bottlerocket/pull/3738
[#3769]: https://github.com/bottlerocket-os/bottlerocket/pull/3769
[#3770]: https://github.com/bottlerocket-os/bottlerocket/pull/3770
[#3771]: https://github.com/bottlerocket-os/bottlerocket/pull/3771
[#3774]: https://github.com/bottlerocket-os/bottlerocket/pull/3774
[#3778]: https://github.com/bottlerocket-os/bottlerocket/pull/3778
[#3779]: https://github.com/bottlerocket-os/bottlerocket/pull/3779
[#3782]: https://github.com/bottlerocket-os/bottlerocket/pull/3782
[#3787]: https://github.com/bottlerocket-os/bottlerocket/pull/3787
[#3789]: https://github.com/bottlerocket-os/bottlerocket/pull/3789
[#3790]: https://github.com/bottlerocket-os/bottlerocket/pull/3790
[#3792]: https://github.com/bottlerocket-os/bottlerocket/pull/3792
[#3797]: https://github.com/bottlerocket-os/bottlerocket/pull/3797
[#3798]: https://github.com/bottlerocket-os/bottlerocket/pull/3798

# v1.19.1 (2024-02-06)

## OS Changes
* Update kernel to 5.10.209, 5.15.148 ([#3765])
* Update host containers ([#3763])

## Orchestrator Changes

### Kubernetes
* Mark pause container image as "pinned" to prevent garbage collection ([#3757])

### ECS
* Update Docker engine and Docker CLI to v25.0.2 ([#3759])
* Update ECS agent to 1.81.0 ([#3759])
* Update AWS SSM agent to 3.2.2222.0 ([#3762])

[#3765]: https://github.com/bottlerocket-os/bottlerocket/pull/3765
[#3763]: https://github.com/bottlerocket-os/bottlerocket/pull/3763
[#3757]: https://github.com/bottlerocket-os/bottlerocket/pull/3757
[#3759]: https://github.com/bottlerocket-os/bottlerocket/pull/3759
[#3762]: https://github.com/bottlerocket-os/bottlerocket/pull/3762

# v1.19.0 (2024-02-01)

## OS Changes
* Adjust unit dependencies for systemd-sysusers ([#3720])
* Update third party packages ([#3722], [#3750])
* Add kernel settings extension ([#3727])
* Update kernel to 5.10.205, 5.15.145, 6.1.72 ([#3734])
* Update runc to 1.1.12 and containerd to 1.6.28 ([#3751])

## Orchestrator Changes

### Kubernetes
* Add latest instance types to eni-max-pods mapping ([#3741])
* Drop Kubernetes 1.24 Metal and VMware variants ([#3742])

### ECS
* Add additional ECS settings for ECS_BACKEND_HOST and ECS_AWSVPC_BLOCK_IMDS ([#3749])

## Build Changes
* twoliter updated to v0.0.6 ([#3744])

[#3720]: https://github.com/bottlerocket-os/bottlerocket/pull/3720
[#3722]: https://github.com/bottlerocket-os/bottlerocket/pull/3722
[#3727]: https://github.com/bottlerocket-os/bottlerocket/pull/3727
[#3734]: https://github.com/bottlerocket-os/bottlerocket/pull/3734
[#3741]: https://github.com/bottlerocket-os/bottlerocket/pull/3741
[#3742]: https://github.com/bottlerocket-os/bottlerocket/pull/3742
[#3744]: https://github.com/bottlerocket-os/bottlerocket/pull/3744
[#3749]: https://github.com/bottlerocket-os/bottlerocket/pull/3749
[#3750]: https://github.com/bottlerocket-os/bottlerocket/pull/3750
[#3751]: https://github.com/bottlerocket-os/bottlerocket/pull/3751

# v1.18.0 (2024-01-16)

## OS Changes

* Remove unused runc SELinux policy rule ([#3673])
* Update third party packages ([#3692])
* Fix creation of kprobes using unqualified names ([#3699], [#3708])
* Update host containers ([#3704])
* Update kernel to 5.10.205, 5.15.145, 6.1.66 ([#3686], [#3708])
* Add container-registry settings extension ([#3674])
* Add updates settings extension ([#3689])

## Orchestrator Changes

### Kubernetes

* Add Kubernetes 1.29 variants ([#3628])
* Update Kubernetes 1.23 to release 33 ([#3692])
* Add latest instance types to eni-max-pods mapping ([#3695])

### ECS

* Update ecs-agent to 1.79.2 ([#3692])

## Build Changes

* Export symbols for packages that include dynamically linked Go binaries ([#3680])
* Update to Bottlerocket SDK v0.37.0 ([#3690])
  + Upgrades to Go 1.21.5

[#3628]: https://github.com/bottlerocket-os/bottlerocket/pull/3628
[#3673]: https://github.com/bottlerocket-os/bottlerocket/pull/3673
[#3674]: https://github.com/bottlerocket-os/bottlerocket/pull/3674
[#3680]: https://github.com/bottlerocket-os/bottlerocket/pull/3680
[#3686]: https://github.com/bottlerocket-os/bottlerocket/pull/3686
[#3689]: https://github.com/bottlerocket-os/bottlerocket/pull/3689
[#3690]: https://github.com/bottlerocket-os/bottlerocket/pull/3690
[#3692]: https://github.com/bottlerocket-os/bottlerocket/pull/3692
[#3695]: https://github.com/bottlerocket-os/bottlerocket/pull/3695
[#3699]: https://github.com/bottlerocket-os/bottlerocket/pull/3699
[#3704]: https://github.com/bottlerocket-os/bottlerocket/pull/3704
[#3708]: https://github.com/bottlerocket-os/bottlerocket/pull/3708

# v1.17.0 (2023-12-12)

## OS Changes

* Generate valid hostname when IPv6 reverse lookup fails ([#3592])
* Avoid mounting the EFI system partition at `/boot` ([#3591])
* Update kernel to 5.10.201, 5.15.139, 6.1.61 ([#3611], [#3643])
* Switch to async `tough` ([#3566])
* Update host containers ([#3646])
* Move template migrations to `schnauzer` v2 ([#3633])
* Handle proxy credentials properly in `pluto` ([#3639], [#3667])
* Update third party packages ([#3612], [#3642])

## Orchestrator Changes

### Kubernetes

* Update `nvidia-k8s-device-plugin` to address CVEs ([#3612])
* Update to Kubernetes 1.28.4 ([#3612])
* Update to Kubernetes 1.27.8 ([#3612])
* Update to Kubernetes 1.26.11 ([#3612])
* Update to Kubernetes 1.25.16 ([#3612])


### ECS

* Update `ecs-agent` to address CVEs ([#3612])

## Build Changes

* Update to Bottlerocket SDK v0.36.1 ([#3640], [#3670])

[#3566]: https://github.com/bottlerocket-os/bottlerocket/pull/3566
[#3591]: https://github.com/bottlerocket-os/bottlerocket/pull/3591
[#3592]: https://github.com/bottlerocket-os/bottlerocket/pull/3592
[#3611]: https://github.com/bottlerocket-os/bottlerocket/pull/3611
[#3612]: https://github.com/bottlerocket-os/bottlerocket/pull/3612
[#3633]: https://github.com/bottlerocket-os/bottlerocket/pull/3633
[#3639]: https://github.com/bottlerocket-os/bottlerocket/pull/3639
[#3640]: https://github.com/bottlerocket-os/bottlerocket/pull/3640
[#3642]: https://github.com/bottlerocket-os/bottlerocket/pull/3642
[#3643]: https://github.com/bottlerocket-os/bottlerocket/pull/3643
[#3646]: https://github.com/bottlerocket-os/bottlerocket/pull/3646
[#3667]: https://github.com/bottlerocket-os/bottlerocket/pull/3667
[#3670]: https://github.com/bottlerocket-os/bottlerocket/pull/3670

# v1.16.1 (2023-11-13)

## OS Changes

* Update open-vm-tools to 12.3.5 to address CVE-2023-34058 and CVE-2023-34059 ([#3553])
* Update NVIDIA drivers to 470.223.02 and 535.129.03 to address CVE‑2023‑31022 and CVE‑2023‑31018 ([#3561])
* Improvements to Bottlerocket CIS benchmark checks ([#3552] [#3562] [#3564])
* Regenerate updog proxy configuration when settings.network.proxy gets updated ([#3578])
* kernel: Update to 5.10.198, 5.15.136, and 6.1.59 ([#3572])

## Orchestrator Changes

### Kubernetes
* Update Kubernetes versions to address HTTP v2 x/net CVE-2023-39325 ([#3581])
* Avoid specifying `hostname-override` kubelet option if `cloud-provider` is set to `aws` ([#3582])

[#3552]: https://github.com/bottlerocket-os/bottlerocket/pull/3552
[#3553]: https://github.com/bottlerocket-os/bottlerocket/pull/3553
[#3561]: https://github.com/bottlerocket-os/bottlerocket/pull/3561
[#3562]: https://github.com/bottlerocket-os/bottlerocket/pull/3562
[#3564]: https://github.com/bottlerocket-os/bottlerocket/pull/3564
[#3572]: https://github.com/bottlerocket-os/bottlerocket/pull/3572
[#3578]: https://github.com/bottlerocket-os/bottlerocket/pull/3578
[#3581]: https://github.com/bottlerocket-os/bottlerocket/pull/3581
[#3582]: https://github.com/bottlerocket-os/bottlerocket/pull/3582

# v1.16.0 (2023-10-25)

## OS Changes

* Adjust netlink timeout to prevent interfaces from entering a failed state ([#3520])
* Update third-party packages ([#3535])
* Add XFS CLI utilities for managing XFS-formatted storage ([#3444])
* Add facilities to auto-load kernel modules ([#3460])
* Update to kernels 5.10.197, 5.15.134, and 6.1.55 ([#3509] [#3542])
* Fix reporting for Bottlerocket CIS Benchmark 4.1.2 ([#3547])
* Update systemd to 252.18 ([#3533])
* Allow fanotify permission events for trusted subjects in SELinux policy ([#3540])

## Orchestrator Changes

### Kubernetes

* Drop Kubernetes 1.23 Metal and VMware variants ([#3531])

### ECS

* Update ecs-agent ([#3535])

## Build Changes

* Update to Bottlerocket SDK v0.35.0 ([#3528])

[#3444]: https://github.com/bottlerocket-os/bottlerocket/pull/3444
[#3460]: https://github.com/bottlerocket-os/bottlerocket/pull/3460
[#3509]: https://github.com/bottlerocket-os/bottlerocket/pull/3509
[#3520]: https://github.com/bottlerocket-os/bottlerocket/pull/3520
[#3528]: https://github.com/bottlerocket-os/bottlerocket/pull/3528
[#3531]: https://github.com/bottlerocket-os/bottlerocket/pull/3531
[#3533]: https://github.com/bottlerocket-os/bottlerocket/pull/3533
[#3535]: https://github.com/bottlerocket-os/bottlerocket/pull/3535
[#3540]: https://github.com/bottlerocket-os/bottlerocket/pull/3540
[#3542]: https://github.com/bottlerocket-os/bottlerocket/pull/3542
[#3547]: https://github.com/bottlerocket-os/bottlerocket/pull/3547

# v1.15.1 (2023-10-9)

## OS Changes

* Allow older ext4 snapshot volumes to be mounted in newer variants that default to xfs ([#3499])
* Update `apiclient` Rust dependencies ([#3491])
* Update `pluto` Rust dependencies ([#3439])
* Patch glibc to address CVE-2023-4806, CVE-2023-4911, and CVE-2023-5156 ([#3501])
* Update open-vm-tools to 12.3.0 to address CVE-2023-20900 ([#3500])

## Build Changes

* Update `twoliter` to v0.0.4 ([#3480])

[#3439]: https://github.com/bottlerocket-os/bottlerocket/pull/3439
[#3480]: https://github.com/bottlerocket-os/bottlerocket/pull/3480
[#3491]: https://github.com/bottlerocket-os/bottlerocket/pull/3491
[#3499]: https://github.com/bottlerocket-os/bottlerocket/pull/3499
[#3500]: https://github.com/bottlerocket-os/bottlerocket/pull/3500
[#3501]: https://github.com/bottlerocket-os/bottlerocket/pull/3501

# v1.15.0 (2023-09-18)

## Major Features

This release brings support for Secure Boot on platforms using UEFI boot; the Linux 6.1 kernel; systemd-networkd and systemd-resolved for host networking; and XFS as the filesystem for local storage.

These features are enabled by default in the new variants. Existing variants will continue to use earlier kernels, `wicked` for host networking, and EXT4 as the filesystem for local storage.

## Known Incompatibilities

* Variants using the 6.1 kernel (`aws-ecs-2`/`aws-ecs-2-nvidia`, `aws-k8s-1.28`/`aws-k8s-1.28-nvidia`, `vmware-k8s-1.28`, and `metal-k8s-1.28`) do not support [LustreFS](https://aws.amazon.com/fsx/lustre/) ([#3459])

## Deprecation Notice

The functionality to apply a hotpatch for log4j CVE-2021-44228 has been removed. The corresponding setting, `settings.oci-hooks.log4j-hotpatch-enabled`, is still available for backwards compatibility. However, it has no effect beyond printing a deprecation warning to the system logs. ([#3401])

## OS Changes

* Add kernel 6.1 ([#3121], [#3441])
* Update admin and control containers ([#3368])
* Update third party packages and dependencies ([#3362], [#3369], [#3330], [#3339], [#3355], [#3441], [#3456])
* Updated to systemd 252 ([#3290])
* Add support for Secure Boot ([#3097])
* Add support for XFS ([#3198])
* Add `apiclient report` command ([#3258]) and Bottlerocket CIS benchmark report ([#2881])
* Add resource-limit settings for OCI defaults ([#3206])
* Use `systemd-networkd` and `systemd-resolved` instead of `wicked` for `aws-k8s-1.28`, `aws-ecs-2`, and `*-dev` variants ([#3134], [#3232], [#3266], [#3311], [#3394], [#3395], [#3451], [#3455])

## Orchestrator Changes

### ECS

* Add `aws-ecs-2` variants ([#3273])
  * Enables Secure Boot, systemd-networkd, and XFS for the data partition
* Add support for AppMesh ([#3267])

### Kubernetes

* Add Kubernetes 1.28 variants ([#3329])
  * Enables Secure Boot, systemd-networkd, and XFS for the data partition
* Drop Kubernetes 1.22 variants ([#2988])
* Update to Kubernetes 1.27.4 ([#3319])
* Update to Kubernetes 1.26.7 ([#3320])
* Update to Kubernetes 1.25.12 ([#3321])
* Update to Kubernetes 1.24.16 ([#3322])
* Add support for SeccompDefault setting for k8s 1.25+ ([#3334])
* Add Kubernetes CIS benchmark report ([#3239])

## Platform Changes

### AWS
* Retry on empty PrivateDnsName from EC2 ([#3364])

### Metal
* Enable Intel VMD driver ([#3419])
* Add linux-firmware ([#3296], [#3418])
* Add aws-iam-authenticator to k8s variants ([#3357])

## Build Changes

* Upgrade to Bottlerocket SDK v0.34.1 ([#3445])
* Use [Twoliter] to enable work on [out-of-tree builds]. Most `tools` have moved to [Twoliter] ([#3379], [#3429], [#3392], [#3342])
* Only limit concurrency while building RPMs ([#3343])


[Twoliter]: https://github.com/bottlerocket-os/twoliter
[out-of-tree builds]: https://github.com/bottlerocket-os/bottlerocket/issues/2669
[#2881]: https://github.com/bottlerocket-os/bottlerocket/pull/2881
[#2988]: https://github.com/bottlerocket-os/bottlerocket/pull/2988
[#3075]: https://github.com/bottlerocket-os/bottlerocket/pull/3075
[#3097]: https://github.com/bottlerocket-os/bottlerocket/pull/3097
[#3121]: https://github.com/bottlerocket-os/bottlerocket/pull/3121
[#3134]: https://github.com/bottlerocket-os/bottlerocket/pull/3134
[#3198]: https://github.com/bottlerocket-os/bottlerocket/pull/3198
[#3206]: https://github.com/bottlerocket-os/bottlerocket/pull/3206
[#3232]: https://github.com/bottlerocket-os/bottlerocket/pull/3232
[#3239]: https://github.com/bottlerocket-os/bottlerocket/pull/3239
[#3258]: https://github.com/bottlerocket-os/bottlerocket/pull/3258
[#3266]: https://github.com/bottlerocket-os/bottlerocket/pull/3266
[#3267]: https://github.com/bottlerocket-os/bottlerocket/pull/3267
[#3273]: https://github.com/bottlerocket-os/bottlerocket/pull/3273
[#3290]: https://github.com/bottlerocket-os/bottlerocket/pull/3290
[#3296]: https://github.com/bottlerocket-os/bottlerocket/pull/3296
[#3311]: https://github.com/bottlerocket-os/bottlerocket/pull/3311
[#3319]: https://github.com/bottlerocket-os/bottlerocket/pull/3319
[#3320]: https://github.com/bottlerocket-os/bottlerocket/pull/3320
[#3321]: https://github.com/bottlerocket-os/bottlerocket/pull/3321
[#3322]: https://github.com/bottlerocket-os/bottlerocket/pull/3322
[#3329]: https://github.com/bottlerocket-os/bottlerocket/pull/3329
[#3330]: https://github.com/bottlerocket-os/bottlerocket/pull/3330
[#3334]: https://github.com/bottlerocket-os/bottlerocket/pull/3334
[#3339]: https://github.com/bottlerocket-os/bottlerocket/pull/3339
[#3342]: https://github.com/bottlerocket-os/bottlerocket/pull/3342
[#3342]: https://github.com/bottlerocket-os/bottlerocket/pull/3342
[#3343]: https://github.com/bottlerocket-os/bottlerocket/pull/3343
[#3355]: https://github.com/bottlerocket-os/bottlerocket/pull/3355
[#3357]: https://github.com/bottlerocket-os/bottlerocket/pull/3357
[#3362]: https://github.com/bottlerocket-os/bottlerocket/pull/3362
[#3364]: https://github.com/bottlerocket-os/bottlerocket/pull/3364
[#3366]: https://github.com/bottlerocket-os/bottlerocket/pull/3366
[#3368]: https://github.com/bottlerocket-os/bottlerocket/pull/3368
[#3369]: https://github.com/bottlerocket-os/bottlerocket/pull/3369
[#3379]: https://github.com/bottlerocket-os/bottlerocket/pull/3379
[#3392]: https://github.com/bottlerocket-os/bottlerocket/pull/3392
[#3394]: https://github.com/bottlerocket-os/bottlerocket/pull/3394
[#3395]: https://github.com/bottlerocket-os/bottlerocket/pull/3395
[#3401]: https://github.com/bottlerocket-os/bottlerocket/pull/3401
[#3418]: https://github.com/bottlerocket-os/bottlerocket/pull/3418
[#3419]: https://github.com/bottlerocket-os/bottlerocket/pull/3419
[#3429]: https://github.com/bottlerocket-os/bottlerocket/pull/3429
[#3441]: https://github.com/bottlerocket-os/bottlerocket/pull/3441
[#3445]: https://github.com/bottlerocket-os/bottlerocket/pull/3445
[#3451]: https://github.com/bottlerocket-os/bottlerocket/pull/3451
[#3455]: https://github.com/bottlerocket-os/bottlerocket/pull/3455
[#3456]: https://github.com/bottlerocket-os/bottlerocket/pull/3456
[#3459]: https://github.com/bottlerocket-os/bottlerocket/issues/3459

# v1.14.3 (2023-08-10)

## OS Changes

* Apply patches to 5.10 and 5.15 kernels to address CVE-2023-20593 ([#3300])
* Update admin and control containers ([#3307])
* Update eni-max-pods with new instance types ([#3324])

## Orchestrator Changes

### Kubernetes

* Update Kubernetes v1.23.17 to include latest EKS-D patches ([#3323])

[#3300]: https://github.com/bottlerocket-os/bottlerocket/pull/3300
[#3307]: https://github.com/bottlerocket-os/bottlerocket/pull/3307
[#3323]: https://github.com/bottlerocket-os/bottlerocket/pull/3323
[#3324]: https://github.com/bottlerocket-os/bottlerocket/pull/3324

# v1.14.2 (2023-07-06)

## OS Changes

* Improve the reliability of acquiring a DHCPv6 lease ([#3211], [#3212])
* Update kernel-5.10 to 5.10.184 and kernel-5.15 to 5.15.117 ([#3238])
* Update eni-max-pods with new instance types ([#3193])
* Make `pluto` outbound API requests more resilient to intermittent network errors ([#3214])
* Update runc to 1.1.6 ([#3249])

## Orchestrator Changes

### ECS

* Add image cleanup settings to control task image cleanup frequency ([#3231])

### Kubernetes

* Update to Kubernetes v1.24.15 ([#3234])
* Update to Kubernetes v1.25.11 ([#3235])
* Update to Kubernetes v1.26.6 ([#3236])
* Update to Kubernetes v1.27.3 ([#3237])

## Build Changes

* Updated Bottlerocket SDK version to v0.33.0 ([#3213])

[#3211]: https://github.com/bottlerocket-os/bottlerocket/pull/3211
[#3212]: https://github.com/bottlerocket-os/bottlerocket/pull/3212
[#3213]: https://github.com/bottlerocket-os/bottlerocket/pull/3213
[#3214]: https://github.com/bottlerocket-os/bottlerocket/pull/3214
[#3231]: https://github.com/bottlerocket-os/bottlerocket/pull/3231
[#3234]: https://github.com/bottlerocket-os/bottlerocket/pull/3234
[#3235]: https://github.com/bottlerocket-os/bottlerocket/pull/3235
[#3236]: https://github.com/bottlerocket-os/bottlerocket/pull/3236
[#3237]: https://github.com/bottlerocket-os/bottlerocket/pull/3237
[#3238]: https://github.com/bottlerocket-os/bottlerocket/pull/3238
[#3193]: https://github.com/bottlerocket-os/bottlerocket/pull/3193
[#3249]: https://github.com/bottlerocket-os/bottlerocket/pull/3249

# v1.14.1 (2023-05-31)

## OS Changes

* Apply patches to 5.10 and 5.15 kernels to address CVE-2023-32233 ([#3128])
* Add fallback container image source parsing for regions not yet supported by the `aws-go-sdk` in `host-ctr` ([#3138])
* Increase default `max_dgram_qlen` sysctl value to `512` for both 5.10 and 5.15 kernels ([#3139])

## Orchestrator Changes

### Kubernetes

* Kubernetes package updates
  * Update Kubernetes v1.22.17 to include latest EKS-D patches ([#3108])
  * Update Kubernetes v1.23.17 to include latest EKS-D patches ([#3119])
  * Update to Kubernetes v1.24.14 ([#3119])
  * Update to Kubernetes v1.25.9 ([#3119])
  * Update to Kubernetes v1.26.4 ([#3119])
  * Update Kubernetes v1.27.1 to include latest EKS-D patches ([#3119])
* Change `nvidia-k8s-device-plugin` service dependency on `kubelet` ([#3141])

## Build Changes

* Fix `pubsys` bug preventing multiple SSM parameter promotions in `promote-ssm` Makefile target ([#3137])

[#3108]: https://github.com/bottlerocket-os/bottlerocket/pull/3108
[#3119]: https://github.com/bottlerocket-os/bottlerocket/pull/3119
[#3128]: https://github.com/bottlerocket-os/bottlerocket/pull/3128
[#3137]: https://github.com/bottlerocket-os/bottlerocket/pull/3137
[#3138]: https://github.com/bottlerocket-os/bottlerocket/pull/3138
[#3139]: https://github.com/bottlerocket-os/bottlerocket/pull/3139
[#3141]: https://github.com/bottlerocket-os/bottlerocket/pull/3141

# v1.14.0 (2023-05-11)

## OS Changes

* Update kernel-5.10 to 5.10.178 and kernel-5.15 to 5.15.108 ([#3077])
* Update admin and control containers ([#3090])
* Update third party packages and dependencies ([#2991], [#3082])
* Enable `SCSI_VIRTIO` driver for better hypervisor support ([#3047])
* Disable panic on hung task for kernel 5.15 ([#3091])
* Create symlink to `inventory` path using Storewolf ([#3035])

## Orchestrator Changes

### ECS

* Add support for ECS Exec ([#3075])

### Kubernetes

* Add Kubernetes 1.27 variants ([#3046])
  * Switch to using Kubernetes default values for `kube-api-burst` and `kube-api-qps` ([#3094])
* Add more Kubernetes settings ([#2930], [#2986])
  * Soft eviction policy
  * Graceful shutdown
  * CPU quota enforcement
  * Memory manager policy
  * CPU manager policy
* Fix Kubernetes 1.26 credential provider apiVersion ([#3070])
* Add ability to pass environment variables to image credential providers ([#2934])

## Build Changes

* Upgrade to Bottlerocket SDK v0.32.0 ([#3071])
* Add AMI validation to PubSys ([#3020])
* Add SSM parameter validation to PubSys ([#2969])
* Add `validate-ami` and `validate-ssm` Makefile targets ([#3043])
* Add `check-migrations` Makefile target to check for common migration problems ([#3051])

## Testing Changes

* Update testsys to v0.0.7 ([#3065])
* Add support for node provisioning with Karpenter ([#3067])
* Enable using custom Sonobuoy images ([#3068])

[#3077]: https://github.com/bottlerocket-os/bottlerocket/pull/3077
[#3090]: https://github.com/bottlerocket-os/bottlerocket/pull/3090
[#2991]: https://github.com/bottlerocket-os/bottlerocket/pull/2991
[#3082]: https://github.com/bottlerocket-os/bottlerocket/pull/3082
[#3047]: https://github.com/bottlerocket-os/bottlerocket/pull/3047
[#3091]: https://github.com/bottlerocket-os/bottlerocket/pull/3091
[#3071]: https://github.com/bottlerocket-os/bottlerocket/pull/3071
[#3035]: https://github.com/bottlerocket-os/bottlerocket/pull/3035
[#3075]: https://github.com/bottlerocket-os/bottlerocket/pull/3075
[#3046]: https://github.com/bottlerocket-os/bottlerocket/pull/3046
[#3094]: https://github.com/bottlerocket-os/bottlerocket/pull/3094
[#2930]: https://github.com/bottlerocket-os/bottlerocket/pull/2930
[#2986]: https://github.com/bottlerocket-os/bottlerocket/pull/2986
[#3070]: https://github.com/bottlerocket-os/bottlerocket/pull/3070
[#2934]: https://github.com/bottlerocket-os/bottlerocket/pull/2934
[#3051]: https://github.com/bottlerocket-os/bottlerocket/pull/3051
[#3020]: https://github.com/bottlerocket-os/bottlerocket/pull/3020
[#2969]: https://github.com/bottlerocket-os/bottlerocket/pull/2969
[#3043]: https://github.com/bottlerocket-os/bottlerocket/pull/3043
[#3065]: https://github.com/bottlerocket-os/bottlerocket/pull/3065
[#3067]: https://github.com/bottlerocket-os/bottlerocket/pull/3067
[#3068]: https://github.com/bottlerocket-os/bottlerocket/pull/3068

# v1.13.5 (2023-05-01)

## OS Changes

* Revert `runc` update to move back to 1.1.5 ([#3054])

[#3054]: https://github.com/bottlerocket-os/bottlerocket/pull/3054

# v1.13.4 (2023-04-24)

## OS Changes

* Ensure the first hostname is used when a VPC DHCP option set has multiple domains ([#3032])
* Update `runc` to version 1.1.6 ([#3037])

## Orchestrator Changes

### Kubernetes

* Generate and pass `--hostname-override` flag to kubelet in `aws-k8s-1.26` variants ([#3033])

[#3032]: https://github.com/bottlerocket-os/bottlerocket/pull/3032
[#3033]: https://github.com/bottlerocket-os/bottlerocket/pull/3033
[#3037]: https://github.com/bottlerocket-os/bottlerocket/pull/3037

# v1.13.3 (2023-04-17)

## OS Changes

* Update kernel-5.10 to 5.10.173 and kernel-5.15 to 5.15.102 ([#2948], [#3002])
* Fix check for rule existence in ip6tables v1.8.9  ([#3001])
* Backport systemd fixes for skipped udevd events ([#2999])
* Check platform-specific mechanisms for hostname first ([#3021])

## Orchestrator Changes

### Kubernetes

* Generate 'provider-id' setting for aws-k8s variants ([#3026])

[#2948]: https://github.com/bottlerocket-os/bottlerocket/pull/2948
[#2999]: https://github.com/bottlerocket-os/bottlerocket/pull/2999
[#3001]: https://github.com/bottlerocket-os/bottlerocket/pull/3001
[#3002]: https://github.com/bottlerocket-os/bottlerocket/pull/3002
[#3021]: https://github.com/bottlerocket-os/bottlerocket/pull/3021
[#3026]: https://github.com/bottlerocket-os/bottlerocket/pull/3026

# v1.13.2 (2023-04-04)

## OS Changes

* Update `runc` to version 1.1.5 ([#2946])

## Orchestrator Changes

### Kubernetes

* Update to Kubernetes v1.26.2 ([#2929])
* Update `aws-iam-authenticator` package to v0.6.8 ([#2965])

[#2946]: https://github.com/bottlerocket-os/bottlerocket/pull/2946
[#2929]: https://github.com/bottlerocket-os/bottlerocket/pull/2929
[#2965]: https://github.com/bottlerocket-os/bottlerocket/pull/2965

# v1.13.1 (2023-03-23)

## OS Changes

* Improve logic around repartitioning and disk expansion by using symlinks to differentiate "fallback" and "preferred" data partitions ([#2935])
* Add `keyutils` package to enable mounting CIFS shares ([#2907])

## Orchestrator Changes

### Kubernetes

* Fix AWS profile rendering in credential provider ([#2904])
* Change CredentialProviderConfig api version to `v1beta1` for Kubernetes 1.25 variants ([#2906])

[#2904]: https://github.com/bottlerocket-os/bottlerocket/pull/2904
[#2906]: https://github.com/bottlerocket-os/bottlerocket/pull/2906
[#2907]: https://github.com/bottlerocket-os/bottlerocket/pull/2907
[#2935]: https://github.com/bottlerocket-os/bottlerocket/pull/2935

# v1.13.0 (2023-03-15)

## OS Changes

* Add `ethtool` to Bottlerocket ([#2829])
* Improve logging in `migrator` to track ongoing migrations ([#2751])
* Improve random-access read performance of root volume on some devices ([#2863])
* Add `CAP_SYS_MODULE` and `CAP_CHROOT` to bootstrap containers ([#2772])
* Add support for cgroup v2 ([#2875], [#2802])
* Disable IA and SafeSetID LSM for kernel-5.15 ([#2789])
* Update kernel-5.10 to 5.10.165 and kernel-5.15 to 5.15.90 ([#2795])
* Allow `=` in bootconfig values ([#2806])
* Include `systemd-analyze plot` for `logdog` ([#2880])
* Update host containers ([#2864])
* Update third party packages ([#2825], [#2842])

## Orchestrator Changes

### Kubernetes

* **Remove Kubernetes 1.21 variants ([#2700])**
* Add Kubernetes 1.26 variants ([#2771], ([#2876])
* Change `kubelet` service to have restart policy `always` ([#2774])
* Update to Kubernetes v1.25.6 ([#2782])
* Update to Kubernetes v1.24.10 ([#2790])
* Update to Kubernetes v1.23.16 ([#2791])
* Update Kubernetes 1.22.17 to include latest EKS-D patches ([#2792])

### ECS

* Enable FireLens capability in `aws-ecs-1` variant ([#2819])

## Platform Changes

### AWS

* Set NVMe IO request timeouts for EBS according to AWS recommendations ([#2820])
* Support an alternate data partition on EC2 instances launched with a single volume ([#2807], [#2879], [#2873])
* Update `eni-max-pod` mappings to include the latest AWS instance types ([#2818])

### VMware

* Remove `k8s.gcr.io` in favor of `public.ecr.aws` ([#2861], ([#2786])
* Disable UDP offload for primary interface ([#2850])

## Build Changes

* Ensure empty build/rpms directory is included in build context ([#2784])
* Add image feature flag for cgroup v2 ([#2845])
* Enable `systemd-networkd` development via build flag ([#2741], [#2832], [#2750])
* Fix `clippy` linter warnings in source files and add `clippy` CI coverage ([#2745])
* Use `clippy` provided in SDK image ([#2793]) ([#2868])
* Remove unnecessary `time` 0.1.x dependency ([#2748], [#2851])
* Remove unnecessary patch from `containerd` ([#2755])
* Update Bottlerocket SDK to v0.30.2 ([#2866], [#2857], [#2836])
* Remove outdated `rust_2018_idioms` enforcement ([#2837])
* Update Rust edition to `2021` ([#2835])
* Upgraded Rust code dependencies ([#2816], [#2869], [#2851], [#2736], [#2895])
* Upgraded Go code dependencies ([#2828], [#2826], [#2813])
* Rename `ncurses` to `libncurses` ([#2769])
* Update schnauzer's registry map ([#2867])

## Testing Changes

* Add support for Kubernetes workloads in `testsys` ([#2830])
* Add support for a `tests` directory ([#2737], [#2775])
* Provide advanced config controls to `testsys` ([#2799])
* Fix incorrect migration starting image for VMware testing in `testsys` ([#2804])
* Use testsys v0.0.6 ([#2865])

## Documentation Changes

* Add boot sequence documentation ([#2735])
* Update Bottlerocket version in provisioning step in `PROVISIONING-METAL.md` ([#2785])
* Add user-data example for setting container registry credentials in `README.md` ([#2803])
* Fix missing trailing backslashes on `ami` commands in `TESTING.md` ([#2838])

[#2700]: https://github.com/bottlerocket-os/bottlerocket/pull/2700
[#2735]: https://github.com/bottlerocket-os/bottlerocket/pull/2735
[#2736]: https://github.com/bottlerocket-os/bottlerocket/pull/2736
[#2737]: https://github.com/bottlerocket-os/bottlerocket/pull/2737
[#2741]: https://github.com/bottlerocket-os/bottlerocket/pull/2741
[#2745]: https://github.com/bottlerocket-os/bottlerocket/pull/2745
[#2748]: https://github.com/bottlerocket-os/bottlerocket/pull/2748
[#2749]: https://github.com/bottlerocket-os/bottlerocket/pull/2749
[#2750]: https://github.com/bottlerocket-os/bottlerocket/pull/2750
[#2751]: https://github.com/bottlerocket-os/bottlerocket/pull/2751
[#2755]: https://github.com/bottlerocket-os/bottlerocket/pull/2755
[#2769]: https://github.com/bottlerocket-os/bottlerocket/pull/2769
[#2771]: https://github.com/bottlerocket-os/bottlerocket/pull/2771
[#2772]: https://github.com/bottlerocket-os/bottlerocket/pull/2772
[#2774]: https://github.com/bottlerocket-os/bottlerocket/pull/2774
[#2775]: https://github.com/bottlerocket-os/bottlerocket/pull/2775
[#2782]: https://github.com/bottlerocket-os/bottlerocket/pull/2782
[#2784]: https://github.com/bottlerocket-os/bottlerocket/pull/2784
[#2785]: https://github.com/bottlerocket-os/bottlerocket/pull/2785
[#2786]: https://github.com/bottlerocket-os/bottlerocket/pull/2786
[#2789]: https://github.com/bottlerocket-os/bottlerocket/pull/2789
[#2790]: https://github.com/bottlerocket-os/bottlerocket/pull/2790
[#2791]: https://github.com/bottlerocket-os/bottlerocket/pull/2791
[#2792]: https://github.com/bottlerocket-os/bottlerocket/pull/2792
[#2793]: https://github.com/bottlerocket-os/bottlerocket/pull/2793
[#2795]: https://github.com/bottlerocket-os/bottlerocket/pull/2795
[#2797]: https://github.com/bottlerocket-os/bottlerocket/pull/2797
[#2799]: https://github.com/bottlerocket-os/bottlerocket/pull/2799
[#2802]: https://github.com/bottlerocket-os/bottlerocket/pull/2802
[#2803]: https://github.com/bottlerocket-os/bottlerocket/pull/2803
[#2804]: https://github.com/bottlerocket-os/bottlerocket/pull/2804
[#2806]: https://github.com/bottlerocket-os/bottlerocket/pull/2806
[#2807]: https://github.com/bottlerocket-os/bottlerocket/pull/2807
[#2813]: https://github.com/bottlerocket-os/bottlerocket/pull/2813
[#2816]: https://github.com/bottlerocket-os/bottlerocket/pull/2816
[#2818]: https://github.com/bottlerocket-os/bottlerocket/pull/2818
[#2819]: https://github.com/bottlerocket-os/bottlerocket/pull/2819
[#2820]: https://github.com/bottlerocket-os/bottlerocket/pull/2820
[#2825]: https://github.com/bottlerocket-os/bottlerocket/pull/2825
[#2826]: https://github.com/bottlerocket-os/bottlerocket/pull/2826
[#2828]: https://github.com/bottlerocket-os/bottlerocket/pull/2828
[#2829]: https://github.com/bottlerocket-os/bottlerocket/pull/2829
[#2830]: https://github.com/bottlerocket-os/bottlerocket/pull/2830
[#2832]: https://github.com/bottlerocket-os/bottlerocket/pull/2832
[#2835]: https://github.com/bottlerocket-os/bottlerocket/pull/2835
[#2836]: https://github.com/bottlerocket-os/bottlerocket/pull/2836
[#2837]: https://github.com/bottlerocket-os/bottlerocket/pull/2837
[#2838]: https://github.com/bottlerocket-os/bottlerocket/pull/2838
[#2842]: https://github.com/bottlerocket-os/bottlerocket/pull/2842
[#2845]: https://github.com/bottlerocket-os/bottlerocket/pull/2845
[#2846]: https://github.com/bottlerocket-os/bottlerocket/pull/2846
[#2850]: https://github.com/bottlerocket-os/bottlerocket/pull/2850
[#2851]: https://github.com/bottlerocket-os/bottlerocket/pull/2851
[#2857]: https://github.com/bottlerocket-os/bottlerocket/pull/2857
[#2861]: https://github.com/bottlerocket-os/bottlerocket/pull/2861
[#2863]: https://github.com/bottlerocket-os/bottlerocket/pull/2863
[#2864]: https://github.com/bottlerocket-os/bottlerocket/pull/2864
[#2865]: https://github.com/bottlerocket-os/bottlerocket/pull/2865
[#2866]: https://github.com/bottlerocket-os/bottlerocket/pull/2866
[#2867]: https://github.com/bottlerocket-os/bottlerocket/pull/2867
[#2868]: https://github.com/bottlerocket-os/bottlerocket/pull/2868
[#2869]: https://github.com/bottlerocket-os/bottlerocket/pull/2869
[#2873]: https://github.com/bottlerocket-os/bottlerocket/pull/2873
[#2875]: https://github.com/bottlerocket-os/bottlerocket/pull/2875
[#2876]: https://github.com/bottlerocket-os/bottlerocket/pull/2876
[#2879]: https://github.com/bottlerocket-os/bottlerocket/pull/2879
[#2880]: https://github.com/bottlerocket-os/bottlerocket/pull/2880
[#2895]: https://github.com/bottlerocket-os/bottlerocket/pull/2895

# v 1.12.0 (2023-01-24)

## OS Changes

* Disable strict aliasing for c-utf-8 library strict aliasing in dbus-broker ([#2730])
* Add `/sys/firmware` to privileged mounts in host-ctr ([#2714])
* Use user-provided registry credentials for public.ecr.aws in host-ctr ([#2676])
* Build masked paths list dynamically in host-ctr ([#2637])
* Enable EFI option in systemd ([#2714])
* Allow simple enums as map keys in datastore ([#2687])
* Improve reliability of `settings.network.hostname` generator ([#2647])
* Add support for bonding and VLANS in `net.toml` ([#2596])
* Keep only one intermediate datastore during migration ([#2589])
* Widen access to filesystem relabel in SELinux policy ([#2738])
* Update hotdog to 1.05 ([#2728])
* Update systemd to 250.9 ([#2718])
* Update third party packages and dependencies ([#2588], [#2717])
* Update host containers ([#2739])
* Update eksd ([#2690], [#2693], [#2694], thanks @rcrozean)

## Orchestrator Changes

### Kubernetes

* Add support for Kubernetes 1.25 variants ([#2699])
* Allow access to public kubelet certificates ([#2639])
* During kubelet prestart, skip pause image pull if image exists ([#2587])
* Delay kubelet.service until after warm-pool-wait service runs ([#2562])
* Add OCI default spec and settings to containerd ([#2697])

## Platform Changes

### VMware

* Downgrade iopl warning when fetching guestinfo in `early-boot-config` ([#2732])

## Build Changes

* Treat alias warning as errors ([#2730])
* Suppress "missing changelog" warning in build ([#2730])
* Update Bottlerocket SDK version to 0.29.0 ([#2730])
* Improve error messages for publish-ami command ([#2695])
* Disallow private AMIs in public SSM parameters ([#2680])
* Rework `start-local-vm` image selection to use `latest` symlink ([#2696])
* Improve integration testing through `cargo make test` ([#2560], [#2592], [#2618], [#2646], [#2653], [#2683], [#2674], [#2723], [#2724], [#2725])


[#2560]: https://github.com/bottlerocket-os/bottlerocket/pull/2560
[#2562]: https://github.com/bottlerocket-os/bottlerocket/pull/2562
[#2587]: https://github.com/bottlerocket-os/bottlerocket/pull/2587
[#2589]: https://github.com/bottlerocket-os/bottlerocket/pull/2589
[#2592]: https://github.com/bottlerocket-os/bottlerocket/pull/2592
[#2596]: https://github.com/bottlerocket-os/bottlerocket/pull/2596
[#2618]: https://github.com/bottlerocket-os/bottlerocket/pull/2618
[#2637]: https://github.com/bottlerocket-os/bottlerocket/pull/2637
[#2639]: https://github.com/bottlerocket-os/bottlerocket/pull/2639
[#2646]: https://github.com/bottlerocket-os/bottlerocket/pull/2646
[#2647]: https://github.com/bottlerocket-os/bottlerocket/pull/2647
[#2650]: https://github.com/bottlerocket-os/bottlerocket/pull/2650
[#2653]: https://github.com/bottlerocket-os/bottlerocket/pull/2653
[#2674]: https://github.com/bottlerocket-os/bottlerocket/pull/2674
[#2676]: https://github.com/bottlerocket-os/bottlerocket/pull/2676
[#2680]: https://github.com/bottlerocket-os/bottlerocket/pull/2680
[#2683]: https://github.com/bottlerocket-os/bottlerocket/pull/2683
[#2687]: https://github.com/bottlerocket-os/bottlerocket/pull/2687
[#2690]: https://github.com/bottlerocket-os/bottlerocket/pull/2690
[#2693]: https://github.com/bottlerocket-os/bottlerocket/pull/2693
[#2694]: https://github.com/bottlerocket-os/bottlerocket/pull/2694
[#2695]: https://github.com/bottlerocket-os/bottlerocket/pull/2695
[#2696]: https://github.com/bottlerocket-os/bottlerocket/pull/2696
[#2697]: https://github.com/bottlerocket-os/bottlerocket/pull/2697
[#2699]: https://github.com/bottlerocket-os/bottlerocket/pull/2699
[#2714]: https://github.com/bottlerocket-os/bottlerocket/pull/2714
[#2717]: https://github.com/bottlerocket-os/bottlerocket/pull/2717
[#2718]: https://github.com/bottlerocket-os/bottlerocket/pull/2718
[#2723]: https://github.com/bottlerocket-os/bottlerocket/pull/2723
[#2724]: https://github.com/bottlerocket-os/bottlerocket/pull/2724
[#2725]: https://github.com/bottlerocket-os/bottlerocket/pull/2725
[#2728]: https://github.com/bottlerocket-os/bottlerocket/pull/2728
[#2730]: https://github.com/bottlerocket-os/bottlerocket/pull/2730
[#2732]: https://github.com/bottlerocket-os/bottlerocket/pull/2732
[#2738]: https://github.com/bottlerocket-os/bottlerocket/pull/2738
[#2739]: https://github.com/bottlerocket-os/bottlerocket/pull/2739

# v1.11.1 (2022-11-28)

## Security Fixes

* Update NVIDIA driver for 5.10 and 5.15 to include recent security fixes ([74d2c5c13ab0][64f3967373a5])
* Apply patch to systemd for CVE-2022-3821 ([#2611])

[74d2c5c13ab0]: https://github.com/bottlerocket-os/bottlerocket/commit/74d2c5c13ab0f6839b9849a9f058a70e82f6ffb8
[64f3967373a5]: https://github.com/bottlerocket-os/bottlerocket/commit/64f3967373a53096219a73580fd81409c846266c
[#2611]: https://github.com/bottlerocket-os/bottlerocket/pull/2611

# v1.11.0 (2022-11-15)

## OS Changes

* Prevent a panic in `early-boot-config` when there is no IMDS region ([#2493])
* Update grub to 2.06-42 ([#2503])
* Bring back wicked support for matching interfaces via hardware address ([#2519])
* Allow bootstrap containers to manage swap ([#2537])
* Add `systemd-analyze` commands to troubleshooting log collection tool ([#2550])
* Allow bootstrap containers to manage network configuration ([#2558])
* Serialize bootconfig values correctly when the value is empty ([#2565])
* Update zlib, libexpat, libdbus, docker-cli ([#2583])
* Update host containers ([#2574])
* Unmask /sys/firmware from host containers ([#2573])

## Orchestrator Changes

### ECS

* Add additional ECS API configurations ([#2527])
  * `ECS_CONTAINER_STOP_TIMEOUT`
  * `ECS_ENGINE_TASK_CLEANUP_WAIT_DURATION`
  * `ECS_TASK_METADATA_RPS_LIMIT`
  * `ECS_RESERVED_MEMORY`

### Kubernetes

* Add a timeout when calling EKS for configuration values ([#2566])
* Enable IAM Roles Anywhere with the k8s `ecr-credential-provider` plugin ([#2377], [#2553])
* Kubernetes EKS-D updates
  * v1.24.6 ([#2582])
  * v1.23.13 ([#2578])
  * v1.22.15 ([#2580], [#2490])

## Platform Changes

### AWS

* Add driver support for AWS variants in hybrid environments ([#2554])

## Build Changes

* Add support for publishing to AWS organizations ([#2484])
* Remove unnecessary dependencies when building grub ([#2495])
* Switch to the latest Dockerfile frontend for builds ([#2496])
* Prepare foundations for Secure Boot and image re-signing ([#2505])
* Fix EFI file system to fit partition size ([#2528])
* Add ShellCheck to `check-lints` for build scripts ([#2532])
* Update the SDK to v0.28.0 ([#2543])
* Use `rustls-native-certs` instead of `webpki-roots` ([#2551])
* Handle absolute paths for output directory in kernel build script ([#2563])

## Documentation Changes

* Add a Roadmap markdown file ([#2549])

[#2377]: https://github.com/bottlerocket-os/bottlerocket/pull/2377
[#2484]: https://github.com/bottlerocket-os/bottlerocket/pull/2484
[#2488]: https://github.com/bottlerocket-os/bottlerocket/pull/2488
[#2490]: https://github.com/bottlerocket-os/bottlerocket/pull/2490
[#2493]: https://github.com/bottlerocket-os/bottlerocket/pull/2493
[#2495]: https://github.com/bottlerocket-os/bottlerocket/pull/2495
[#2496]: https://github.com/bottlerocket-os/bottlerocket/pull/2496
[#2503]: https://github.com/bottlerocket-os/bottlerocket/pull/2503
[#2505]: https://github.com/bottlerocket-os/bottlerocket/pull/2505
[#2519]: https://github.com/bottlerocket-os/bottlerocket/pull/2519
[#2523]: https://github.com/bottlerocket-os/bottlerocket/pull/2523
[#2527]: https://github.com/bottlerocket-os/bottlerocket/pull/2527
[#2528]: https://github.com/bottlerocket-os/bottlerocket/pull/2528
[#2532]: https://github.com/bottlerocket-os/bottlerocket/pull/2532
[#2536]: https://github.com/bottlerocket-os/bottlerocket/pull/2536
[#2537]: https://github.com/bottlerocket-os/bottlerocket/pull/2537
[#2540]: https://github.com/bottlerocket-os/bottlerocket/pull/2540
[#2541]: https://github.com/bottlerocket-os/bottlerocket/pull/2541
[#2542]: https://github.com/bottlerocket-os/bottlerocket/pull/2542
[#2543]: https://github.com/bottlerocket-os/bottlerocket/pull/2543
[#2547]: https://github.com/bottlerocket-os/bottlerocket/pull/2547
[#2549]: https://github.com/bottlerocket-os/bottlerocket/pull/2549
[#2550]: https://github.com/bottlerocket-os/bottlerocket/pull/2550
[#2551]: https://github.com/bottlerocket-os/bottlerocket/pull/2551
[#2553]: https://github.com/bottlerocket-os/bottlerocket/pull/2553
[#2554]: https://github.com/bottlerocket-os/bottlerocket/pull/2554
[#2558]: https://github.com/bottlerocket-os/bottlerocket/pull/2558
[#2563]: https://github.com/bottlerocket-os/bottlerocket/pull/2563
[#2565]: https://github.com/bottlerocket-os/bottlerocket/pull/2565
[#2566]: https://github.com/bottlerocket-os/bottlerocket/pull/2566
[#2574]: https://github.com/bottlerocket-os/bottlerocket/pull/2574
[#2573]: https://github.com/bottlerocket-os/bottlerocket/pull/2573
[#2578]: https://github.com/bottlerocket-os/bottlerocket/pull/2578
[#2580]: https://github.com/bottlerocket-os/bottlerocket/pull/2580
[#2582]: https://github.com/bottlerocket-os/bottlerocket/pull/2582
[#2583]: https://github.com/bottlerocket-os/bottlerocket/pull/2583

# v1.10.1 (2022-10-19)

## OS Changes
* Support container runtime settings: enable-unprivileged-icmp, enable-unprivileged-ports, max-concurrent-downloads, max-container-log-line-size ([#2494])
* Update EKS-D to 1.22-11 ([#2490])
* Update EKS-D to 1.23-6 ([#2488])

[#2488]: https://github.com/bottlerocket-os/bottlerocket/pull/2488
[#2490]: https://github.com/bottlerocket-os/bottlerocket/pull/2490
[#2494]: https://github.com/bottlerocket-os/bottlerocket/pull/2494

# v1.10.0 (2022-10-10)

## OS Changes
* Add optional settings to reboot into new kernel command line parameters ([#2375])
* Support for static IP addressing ([#2204], [#2330], [#2445])
* Add support for NVIDIA driver version 515 ([#2455])
* Set mode for tmpfs mounts ([#2473])
* Increase inotify default limits ([#2335])
* Align `vm.max_map_count` with the EKS Optimized AMI ([#2344])
* Add support for configuring DNS settings ([#2353])
* Migrate `netdog` from `serde_xml_rs` to `quick-xml` ([#2311])
* Support versioning for `net.toml` ([#2281])
* Update admin and control container ([#2471], [#2472])

## Orchestrator Changes

### ECS
* Add `cargo make` tasks for testing ECS variants ([#2348])

### Kubernetes

* Add support for Kubernetes 1.24 variants ([#2437])
* Remove Kubernetes aws-k8s-1.19 variants ([#2316])
* Increase the kube-api-server QPS from 5/10 to 10/20 ([#2436], thanks @tzneal)
* Update eni-max-pods with new instance types ([#2416])
* Add setting to change `kubelet`'s log level ([#2460], [#2470])
* Add `cargo make` tasks to perform migration testing for Kubernetes variants in AWS ([#2273])

## Platform Changes

### AWS
* Disable drivers for USB-attached network interfaces ([#2328])

### Metal
* Add driver support for Solarflare, Pensando, Myricom, Huawei, Emulex, Chelsio, Broadcom, AMD and Intel 10G+ network cards ([#2379])

## Build Changes
* Extend `external-files` to vendor go modules ([#2378], [#2403], [#2430])
* Make `net_config` unit tests reusable across versions ([#2385])
* Add `diff-kernel-config` to identify kernel config changes ([#2368])
* Extended support for variants in buildsys ([#2339])
* Clarify crossbeam license ([#2447])
* Honor `BUILDSYS_ARCH` and `BUILDSYS_VARIANT` env variables when set ([#2425])
* Use architecture specific json payloads in unit tests ([#2367], [#2363])
* Add unified `check` target in `Makefile.toml` for review readiness ([#2384])
* Update Go dependencies of first-party go projects ([#2424], [#2440], [#2450], [#2452], [#2456])
* Update Rust dependencies ([#2458], [#2476])
* Update third-party packages ([#2397], [#2398], [#2464], [#2465], thanks @kschumy)
* Update Bottlerocket SDK to 0.27.0 ([#2428])
* Migrate `pubsys` and `infrasys` to the AWS SDK for Rust ([#2414], [#2415], [#2454])
* Update `testsys` dependencies ([#2392])
* Fix `hotdog`'s spec URL to the correct upstream link ([#2326])
* Fix clippy warnings and enable lints on pull requests ([#2337], [#2346], [#2443])
* Format issue field in PR template ([#2314])

## Documentation Changes
* Update checksum for new `root.json` ([#2405])
* Mention that boot settings are available in Kubernetes 1.23 variants ([#2358])
* Mention the need for AWS credentials in BUILDING.md and PUBLISHING-AWS.md ([#2334])
* Add China to supported regions lists ([#2315])
* Add community section to README.md ([#2305], [#2383])
* Standardize `userdata.toml` as the filename used in different docs ([#2446])
* Remove commit from image name in PROVISIONING-METAL.md ([#2312])
* Add note to CONTRIBUTING.md that outlines filenames' casing ([#2306])
* Fix typos in `Makefile.toml`, QUICKSTART-ECS.md, QUICKSTART-EKS.md, `netdog` and `prairiedog` ([#2318], thanks @kianmeng)
* Fix casing for GitHub and VMware in CHANGELOG.md ([#2329])
* Fix typo in test setup command ([#2477])
* Fix TESTING.md link typo ([#2438])
* Fix positional `fetch-license` argument ([#2457])

[#2204]: https://github.com/bottlerocket-os/bottlerocket/pull/2204
[#2273]: https://github.com/bottlerocket-os/bottlerocket/pull/2273
[#2281]: https://github.com/bottlerocket-os/bottlerocket/pull/2281
[#2305]: https://github.com/bottlerocket-os/bottlerocket/pull/2305
[#2306]: https://github.com/bottlerocket-os/bottlerocket/pull/2306
[#2311]: https://github.com/bottlerocket-os/bottlerocket/pull/2311
[#2312]: https://github.com/bottlerocket-os/bottlerocket/pull/2312
[#2314]: https://github.com/bottlerocket-os/bottlerocket/pull/2314
[#2315]: https://github.com/bottlerocket-os/bottlerocket/pull/2315
[#2316]: https://github.com/bottlerocket-os/bottlerocket/pull/2316
[#2318]: https://github.com/bottlerocket-os/bottlerocket/pull/2318
[#2326]: https://github.com/bottlerocket-os/bottlerocket/pull/2326
[#2328]: https://github.com/bottlerocket-os/bottlerocket/pull/2328
[#2329]: https://github.com/bottlerocket-os/bottlerocket/pull/2329
[#2330]: https://github.com/bottlerocket-os/bottlerocket/pull/2330
[#2334]: https://github.com/bottlerocket-os/bottlerocket/pull/2334
[#2335]: https://github.com/bottlerocket-os/bottlerocket/pull/2335
[#2337]: https://github.com/bottlerocket-os/bottlerocket/pull/2337
[#2339]: https://github.com/bottlerocket-os/bottlerocket/pull/2339
[#2344]: https://github.com/bottlerocket-os/bottlerocket/pull/2344
[#2346]: https://github.com/bottlerocket-os/bottlerocket/pull/2346
[#2348]: https://github.com/bottlerocket-os/bottlerocket/pull/2348
[#2353]: https://github.com/bottlerocket-os/bottlerocket/pull/2353
[#2358]: https://github.com/bottlerocket-os/bottlerocket/pull/2358
[#2363]: https://github.com/bottlerocket-os/bottlerocket/pull/2363
[#2367]: https://github.com/bottlerocket-os/bottlerocket/pull/2367
[#2368]: https://github.com/bottlerocket-os/bottlerocket/pull/2368
[#2375]: https://github.com/bottlerocket-os/bottlerocket/pull/2375
[#2378]: https://github.com/bottlerocket-os/bottlerocket/pull/2378
[#2379]: https://github.com/bottlerocket-os/bottlerocket/pull/2379
[#2383]: https://github.com/bottlerocket-os/bottlerocket/pull/2383
[#2384]: https://github.com/bottlerocket-os/bottlerocket/pull/2384
[#2385]: https://github.com/bottlerocket-os/bottlerocket/pull/2385
[#2392]: https://github.com/bottlerocket-os/bottlerocket/pull/2392
[#2397]: https://github.com/bottlerocket-os/bottlerocket/pull/2397
[#2398]: https://github.com/bottlerocket-os/bottlerocket/pull/2398
[#2403]: https://github.com/bottlerocket-os/bottlerocket/pull/2403
[#2405]: https://github.com/bottlerocket-os/bottlerocket/pull/2405
[#2414]: https://github.com/bottlerocket-os/bottlerocket/pull/2414
[#2415]: https://github.com/bottlerocket-os/bottlerocket/pull/2415
[#2416]: https://github.com/bottlerocket-os/bottlerocket/pull/2416
[#2424]: https://github.com/bottlerocket-os/bottlerocket/pull/2424
[#2425]: https://github.com/bottlerocket-os/bottlerocket/pull/2425
[#2428]: https://github.com/bottlerocket-os/bottlerocket/pull/2428
[#2430]: https://github.com/bottlerocket-os/bottlerocket/pull/2430
[#2436]: https://github.com/bottlerocket-os/bottlerocket/pull/2436
[#2437]: https://github.com/bottlerocket-os/bottlerocket/pull/2437
[#2438]: https://github.com/bottlerocket-os/bottlerocket/pull/2438
[#2440]: https://github.com/bottlerocket-os/bottlerocket/pull/2440
[#2443]: https://github.com/bottlerocket-os/bottlerocket/pull/2443
[#2445]: https://github.com/bottlerocket-os/bottlerocket/pull/2445
[#2446]: https://github.com/bottlerocket-os/bottlerocket/pull/2446
[#2447]: https://github.com/bottlerocket-os/bottlerocket/pull/2447
[#2450]: https://github.com/bottlerocket-os/bottlerocket/pull/2450
[#2452]: https://github.com/bottlerocket-os/bottlerocket/pull/2452
[#2454]: https://github.com/bottlerocket-os/bottlerocket/pull/2454
[#2455]: https://github.com/bottlerocket-os/bottlerocket/pull/2455
[#2456]: https://github.com/bottlerocket-os/bottlerocket/pull/2456
[#2457]: https://github.com/bottlerocket-os/bottlerocket/pull/2457
[#2458]: https://github.com/bottlerocket-os/bottlerocket/pull/2458
[#2460]: https://github.com/bottlerocket-os/bottlerocket/pull/2460
[#2464]: https://github.com/bottlerocket-os/bottlerocket/pull/2464
[#2465]: https://github.com/bottlerocket-os/bottlerocket/pull/2465
[#2470]: https://github.com/bottlerocket-os/bottlerocket/pull/2470
[#2471]: https://github.com/bottlerocket-os/bottlerocket/pull/2471
[#2472]: https://github.com/bottlerocket-os/bottlerocket/pull/2472
[#2473]: https://github.com/bottlerocket-os/bottlerocket/pull/2473
[#2476]: https://github.com/bottlerocket-os/bottlerocket/pull/2476
[#2477]: https://github.com/bottlerocket-os/bottlerocket/pull/2477

# v1.9.2 (2022-08-31)

## Build Changes

* Archive old migrations ([#2357])
* Update `runc` to version 1.1.4 ([#2380])

[#2357]: https://github.com/bottlerocket-os/bottlerocket/pull/2357
[#2380]: https://github.com/bottlerocket-os/bottlerocket/pull/2380

# v1.9.1 (2022-08-17)

## OS Changes

* Change kernel module compression from zstd to xz ([#2323])
* Update ECR registry map for new AWS regions ([#2336])
* Add new regions to pause registry map ([#2349])
* Update `tough` to v0.8.1 ([#2338])

[#2323]: https://github.com/bottlerocket-os/bottlerocket/pull/2323
[#2336]: https://github.com/bottlerocket-os/bottlerocket/pull/2336
[#2338]: https://github.com/bottlerocket-os/bottlerocket/pull/2338
[#2349]: https://github.com/bottlerocket-os/bottlerocket/pull/2349

# v1.9.0 (2022-07-28)

## OS Changes

* SELinux policy now suppresses audit for tmpfs relabels ([#2222])
* Restrict permissions for `/boot` and `System.map` ([#2223])
* Remove unused crates `growpart` and `servicedog` ([#2238])
* New mount in host containers for system logs ([#2295])
* Apply strict mount options and enforce execution rules ([#2239])
* Switch to a more commonly used syntax for disabling kernel config settings ([#2290])
* Respect proxy settings when running setting generators ([#2227])
* Add `NET_CAP_ADMIN` to bootstrap containers ([#2266])
* Reduce log output for DHCP services ([#2260])
* Fix invalid kernel config options ([#2269])
* Improve support for container storage mounts ([#2240])
* Disable uncommon filesystems and network protocols ([#2255])
* Add support for blocking kernel modules ([#2274])
* Fix `ntp` service restart when settings change ([#2270])
* Add kernel 5.15 sources ([#2226])
* Defer `squashfs` mounts to later in the boot process ([#2276])
* Improve boot speed and rootfs size ([#2296])
* Add "quiet" kernel parameter for some variants ([#2277])

## Orchestrator Changes

### Kubernetes

* Make new instance types available ([#2221] , thanks @cablespaghetti)
* Update Kubernetes versions ([#2230], [#2232], [#2262], [#2263], thanks @kschumy)
* Add kubelet image GC threshold settings ([#2219])

### ECS

* Add iptables rules for ECS introspection server ([#2267])

## Platform Changes

### AWS

* Add support for AWS China regions ([#2224], [#2242], [#2247], [#2285])
* Migrate to using `aws-sdk-rust` for first-party OS Rust packages ([#2300])

### VMware

* Remove `console=ttyS0` from kernel params ([#2248])

### Metal

* Enable Mellanox modules in 5.10 kernel ([#2241])
* Add bnxt module for Broadcom 10/25Gb network adapters in 5.10 kernel ([#2243])
* Split out baremetal specific config options ([#2264])
* Add driver support for Cisco UCS platforms ([#2271])
* Only build baremetal variant specific drivers for baremetal variants ([#2279])
* Enable the metal-dev build for the ARM architecture ([#2272])

## Build Changes

* Add Makefile targets to create and validate Boot Configuration ([#2189])
* Create symlinks to images with friendly names ([#2215])
* Add `start-local-vm` script ([#2194])
* Add the testsys CLI and new cargo make tasks for testing aws-k8s variants ([#2165])
* Update Rust and Go dependencies ([#2303], [#2299])
* Update third-party packages ([#2309])

## Documentation Changes

* Add NVIDIA ECS variant to README ([#2244])
* Add documentation for metal variants ([#2205])
* Add missing step in building packages guide ([#2259])
* Add quickstart for running Bottlerocket in QEMU/KVM VMs ([#2280])
* Address lints in README markdown caught by `markdownlint` ([#2283])

[#2165]: https://github.com/bottlerocket-os/bottlerocket/pull/2165
[#2189]: https://github.com/bottlerocket-os/bottlerocket/pull/2189
[#2194]: https://github.com/bottlerocket-os/bottlerocket/pull/2194
[#2205]: https://github.com/bottlerocket-os/bottlerocket/pull/2205
[#2215]: https://github.com/bottlerocket-os/bottlerocket/pull/2215
[#2219]: https://github.com/bottlerocket-os/bottlerocket/pull/2219
[#2221]: https://github.com/bottlerocket-os/bottlerocket/pull/2221
[#2222]: https://github.com/bottlerocket-os/bottlerocket/pull/2222
[#2223]: https://github.com/bottlerocket-os/bottlerocket/pull/2223
[#2224]: https://github.com/bottlerocket-os/bottlerocket/pull/2224
[#2226]: https://github.com/bottlerocket-os/bottlerocket/pull/2226
[#2227]: https://github.com/bottlerocket-os/bottlerocket/pull/2227
[#2230]: https://github.com/bottlerocket-os/bottlerocket/pull/2230
[#2232]: https://github.com/bottlerocket-os/bottlerocket/pull/2232
[#2238]: https://github.com/bottlerocket-os/bottlerocket/pull/2238
[#2239]: https://github.com/bottlerocket-os/bottlerocket/pull/2239
[#2240]: https://github.com/bottlerocket-os/bottlerocket/pull/2240
[#2241]: https://github.com/bottlerocket-os/bottlerocket/pull/2241
[#2242]: https://github.com/bottlerocket-os/bottlerocket/pull/2242
[#2243]: https://github.com/bottlerocket-os/bottlerocket/pull/2243
[#2244]: https://github.com/bottlerocket-os/bottlerocket/pull/2244
[#2247]: https://github.com/bottlerocket-os/bottlerocket/pull/2247
[#2248]: https://github.com/bottlerocket-os/bottlerocket/pull/2248
[#2255]: https://github.com/bottlerocket-os/bottlerocket/pull/2255
[#2259]: https://github.com/bottlerocket-os/bottlerocket/pull/2259
[#2260]: https://github.com/bottlerocket-os/bottlerocket/pull/2260
[#2262]: https://github.com/bottlerocket-os/bottlerocket/pull/2262
[#2263]: https://github.com/bottlerocket-os/bottlerocket/pull/2263
[#2264]: https://github.com/bottlerocket-os/bottlerocket/pull/2264
[#2266]: https://github.com/bottlerocket-os/bottlerocket/pull/2266
[#2267]: https://github.com/bottlerocket-os/bottlerocket/pull/2267
[#2269]: https://github.com/bottlerocket-os/bottlerocket/pull/2269
[#2270]: https://github.com/bottlerocket-os/bottlerocket/pull/2270
[#2271]: https://github.com/bottlerocket-os/bottlerocket/pull/2271
[#2272]: https://github.com/bottlerocket-os/bottlerocket/pull/2272
[#2274]: https://github.com/bottlerocket-os/bottlerocket/pull/2274
[#2276]: https://github.com/bottlerocket-os/bottlerocket/pull/2276
[#2277]: https://github.com/bottlerocket-os/bottlerocket/pull/2277
[#2279]: https://github.com/bottlerocket-os/bottlerocket/pull/2279
[#2280]: https://github.com/bottlerocket-os/bottlerocket/pull/2280
[#2283]: https://github.com/bottlerocket-os/bottlerocket/pull/2283
[#2285]: https://github.com/bottlerocket-os/bottlerocket/pull/2285
[#2290]: https://github.com/bottlerocket-os/bottlerocket/pull/2290
[#2295]: https://github.com/bottlerocket-os/bottlerocket/pull/2295
[#2296]: https://github.com/bottlerocket-os/bottlerocket/pull/2296
[#2299]: https://github.com/bottlerocket-os/bottlerocket/pull/2299
[#2300]: https://github.com/bottlerocket-os/bottlerocket/pull/2300
[#2303]: https://github.com/bottlerocket-os/bottlerocket/pull/2303
[#2309]: https://github.com/bottlerocket-os/bottlerocket/pull/2309

# v1.8.0 (2022-06-08)

## OS Changes

### General
* Update admin and control containers ([#2191])
* Update to containerd 1.6.x ([#2158])
* Restart container runtimes when certificates store changes ([#2076])
* Add support for providing kernel parameters via Boot Configuration ([#1980])
* Restart long-running systemd services on exit ([#2162])
* Ignore zero blocks on dm-verity root ([#2169])
* Add support for static DNS mappings in `/etc/hosts` ([#2129])
* Enable network configuration generation via `netdog` ([#2066])
* Add support for non-`eth0` default interfaces ([#2144])
* Update to IMDS schema `2021-07-15` ([#2190])

### Kubernetes
* Add support for Kubernetes 1.23 variants ([#2188])
* Improve Kubernetes pod start times by unsetting `configMapAndSecretChangeDetectionStrategy` in kubelet config ([#2166])
* Add new setting for configuring kubelet's `provider-id` configuration ([#2192])
* Add new setting for configuring kubelet's `podPidsLimit` configuration ([#2138])
* Allow a list of IP addresses in `settings.kubernetes.cluster-dns-ip` ([#2176])
* Set the default for `settings.kubernetes.cloud-provider` on metal variants to an empty string ([#2188])
* Add c7g instance data for max pods calculation in AWS variants ([#2107], thanks, @lizthegrey!)

### ECS
* Add aws-ecs-1-nvidia variant with Nvidia driver support ([#2128], [#2100], [#2098], [#2167], [#2097], [#2090], [#2099])
* Add support for ECS ImagePullBehavior and WarmPoolsSupport ([#2063], thanks, @mello7tre!)

### Hardware
* Build smartpqi driver for Microchip Smart Storage devices into 5.10 kernel ([#2184])
* Add support for Broadcom ethernet cards in 5.10 kernel ([#2143])
* Add support for MegaRAID SAS in 5.10 kernel ([#2133])

## Build Changes
* Remove aws-k8s-1.18 variant ([#2044], [#2092])
* Update third-party packages ([#2178], [#2187], [#2145])
* Update Rust and Go dependencies ([#2183], [#2181], [#2180], [#2085], [#2110], [#2068], [#2075], [#2074], [#2048], [#2059], [#2049], [#2036], [#2033])
* Update Bottlerocket SDK to 0.26.0 ([#2157])
* Speed up kernel builds by installing headers and modules in parallel ([#2185])
* Removed unused patch from Docker CLI ([#2030], thanks, @thaJeztah!)

## Documentation Changes
* Standardize README generation in buildsys ([#2134])
* Clarify migration README ([#2141])
* Fix typos in BUILDING.md and QUICKSTART-VMWARE.md ([#2159], thanks, @ryanrussell!)
* Add additional documentation for using GPUs with Kubernetes variants ([#2078])
* Document examples for using `enter-admin-container` ([#2028])

[#1980]: https://github.com/bottlerocket-os/bottlerocket/pull/1980
[#2028]: https://github.com/bottlerocket-os/bottlerocket/pull/2028
[#2030]: https://github.com/bottlerocket-os/bottlerocket/pull/2030
[#2033]: https://github.com/bottlerocket-os/bottlerocket/pull/2033
[#2036]: https://github.com/bottlerocket-os/bottlerocket/pull/2036
[#2044]: https://github.com/bottlerocket-os/bottlerocket/pull/2044
[#2048]: https://github.com/bottlerocket-os/bottlerocket/pull/2048
[#2049]: https://github.com/bottlerocket-os/bottlerocket/pull/2049
[#2059]: https://github.com/bottlerocket-os/bottlerocket/pull/2059
[#2063]: https://github.com/bottlerocket-os/bottlerocket/pull/2063
[#2066]: https://github.com/bottlerocket-os/bottlerocket/pull/2066
[#2068]: https://github.com/bottlerocket-os/bottlerocket/pull/2068
[#2074]: https://github.com/bottlerocket-os/bottlerocket/pull/2074
[#2075]: https://github.com/bottlerocket-os/bottlerocket/pull/2075
[#2076]: https://github.com/bottlerocket-os/bottlerocket/pull/2076
[#2078]: https://github.com/bottlerocket-os/bottlerocket/pull/2078
[#2085]: https://github.com/bottlerocket-os/bottlerocket/pull/2085
[#2090]: https://github.com/bottlerocket-os/bottlerocket/pull/2090
[#2092]: https://github.com/bottlerocket-os/bottlerocket/pull/2092
[#2097]: https://github.com/bottlerocket-os/bottlerocket/pull/2097
[#2098]: https://github.com/bottlerocket-os/bottlerocket/pull/2098
[#2099]: https://github.com/bottlerocket-os/bottlerocket/pull/2099
[#2100]: https://github.com/bottlerocket-os/bottlerocket/pull/2100
[#2107]: https://github.com/bottlerocket-os/bottlerocket/pull/2107
[#2110]: https://github.com/bottlerocket-os/bottlerocket/pull/2110
[#2128]: https://github.com/bottlerocket-os/bottlerocket/pull/2128
[#2129]: https://github.com/bottlerocket-os/bottlerocket/pull/2129
[#2133]: https://github.com/bottlerocket-os/bottlerocket/pull/2133
[#2134]: https://github.com/bottlerocket-os/bottlerocket/pull/2134
[#2138]: https://github.com/bottlerocket-os/bottlerocket/pull/2138
[#2141]: https://github.com/bottlerocket-os/bottlerocket/pull/2141
[#2142]: https://github.com/bottlerocket-os/bottlerocket/pull/2142
[#2143]: https://github.com/bottlerocket-os/bottlerocket/pull/2143
[#2144]: https://github.com/bottlerocket-os/bottlerocket/pull/2144
[#2145]: https://github.com/bottlerocket-os/bottlerocket/pull/2145
[#2146]: https://github.com/bottlerocket-os/bottlerocket/pull/2146
[#2157]: https://github.com/bottlerocket-os/bottlerocket/pull/2157
[#2158]: https://github.com/bottlerocket-os/bottlerocket/pull/2158
[#2159]: https://github.com/bottlerocket-os/bottlerocket/pull/2159
[#2162]: https://github.com/bottlerocket-os/bottlerocket/pull/2162
[#2166]: https://github.com/bottlerocket-os/bottlerocket/pull/2166
[#2167]: https://github.com/bottlerocket-os/bottlerocket/pull/2167
[#2169]: https://github.com/bottlerocket-os/bottlerocket/pull/2169
[#2176]: https://github.com/bottlerocket-os/bottlerocket/pull/2176
[#2178]: https://github.com/bottlerocket-os/bottlerocket/pull/2178
[#2180]: https://github.com/bottlerocket-os/bottlerocket/pull/2180
[#2181]: https://github.com/bottlerocket-os/bottlerocket/pull/2181
[#2183]: https://github.com/bottlerocket-os/bottlerocket/pull/2183
[#2184]: https://github.com/bottlerocket-os/bottlerocket/pull/2184
[#2185]: https://github.com/bottlerocket-os/bottlerocket/pull/2185
[#2187]: https://github.com/bottlerocket-os/bottlerocket/pull/2187
[#2188]: https://github.com/bottlerocket-os/bottlerocket/pull/2188
[#2190]: https://github.com/bottlerocket-os/bottlerocket/pull/2190
[#2191]: https://github.com/bottlerocket-os/bottlerocket/pull/2191
[#2192]: https://github.com/bottlerocket-os/bottlerocket/pull/2192

# v1.7.2 (2022-04-22)

## Security Fixes

* Update kernel-5.4 to patch CVE-2022-1015, CVE-2022-1016, CVE-2022-25636, CVE-2022-26490, CVE-2022-27666, CVE-2022-28356 ([a3b4674f7108][a3b4674f7108])
* Update kernel-5.10 to patch CVE-2022-1015, CVE-2022-1016, CVE-2022-25636, CVE-2022-1048, CVE-2022-26490, CVE-2022-27666, CVE-2022-28356 ([37095415bab6][37095415bab6])

## OS Changes

* Update eni-max-pods with new instance types ([#2079])
* Add support for AWS region ap-southeast-3: Jakarta ([#2080])

[a3b4674f7108]: https://github.com/bottlerocket-os/bottlerocket/commit/a3b4674f7108a7f69f108a011042be2a5b91e563
[37095415bab6]: https://github.com/bottlerocket-os/bottlerocket/commit/37095415bab67a24240d95b59c7bf20a112d7ae1
[#2079]: https://github.com/bottlerocket-os/bottlerocket/pull/2079
[#2080]: https://github.com/bottlerocket-os/bottlerocket/pull/2080

# v1.7.1 (2022-04-05)

## Security Fixes

* Apply patch to hotdog for CVE-2022-0071 ([1a3f35b2fe8e][1a3f35b2fe8e])

## OS Changes

* Enable checkpoint restore (`CONFIG_CHECKPOINT_RESTORE`) for aarch64 ([6e3d6ed4b83e][6e3d6ed4b83e])

[1a3f35b2fe8e]: https://github.com/bottlerocket-os/bottlerocket/commit/1a3f35b2fe8ed9a7078e43940545dc941c5de99f
[6e3d6ed4b83e]: https://github.com/bottlerocket-os/bottlerocket/commit/6e3d6ed4b83ecefa5de5885f8c4a30cd9df8b689

# v1.7.0 (2022-03-30)

With this release, an inventory of software installed in Bottlerocket will now be reported to SSM if the control container is in use and inventorying has been enabled.

## OS Changes

* Generate host software inventory and make it available to host containers ([#1996])
* Update admin and control containers ([#2014])

## Build Changes

* Update third-party packages ([#1977], [#1983], [#1987], [#1992], [#2022])
* Update Rust and Go dependencies ([#2016], [#2019])
* Makefile: lock tuftool version ([#2009])
* Fix tmpfilesd configuration for kmod-5.10-nvidia ([#2020])

## Documentation Changes

* Fix tuftool download instruction in VMware Quickstart ([#1994])
* Explain data partition extension ([#2013])

[#1977]: https://github.com/bottlerocket-os/bottlerocket/pull/1977
[#1983]: https://github.com/bottlerocket-os/bottlerocket/pull/1983
[#1987]: https://github.com/bottlerocket-os/bottlerocket/pull/1987
[#1992]: https://github.com/bottlerocket-os/bottlerocket/pull/1992
[#1994]: https://github.com/bottlerocket-os/bottlerocket/pull/1994
[#1996]: https://github.com/bottlerocket-os/bottlerocket/pull/1996
[#2009]: https://github.com/bottlerocket-os/bottlerocket/pull/2009
[#2013]: https://github.com/bottlerocket-os/bottlerocket/pull/2013
[#2014]: https://github.com/bottlerocket-os/bottlerocket/pull/2014
[#2016]: https://github.com/bottlerocket-os/bottlerocket/pull/2016
[#2019]: https://github.com/bottlerocket-os/bottlerocket/pull/2019
[#2020]: https://github.com/bottlerocket-os/bottlerocket/pull/2020
[#2022]: https://github.com/bottlerocket-os/bottlerocket/pull/2022

# v1.6.2 (2022-03-08)

With this release, the vmware-k8s variants have graduated from preview status and are now generally available.
:tada:

## Security Fixes

* Update kernel-5.4 and kernel-5.10 to include recent security fixes ([a8e4a20ca7d1][a8e4a20ca7d1], [3d0c10abeecb][3d0c10abeecb])

## OS Changes

* Add support for Kubernetes 1.22 variants ([#1962])
* Add settings support for registry credentials ([#1955])
* Add support for AWS CloudFormation signaling ([#1728], thanks, @mello7tre!)
* Add TCMU support to the kernel ([#1953], thanks, @cvlc!)
* Fix issue with closing frame construction in apiserver ([#1948])

## Build Changes

* Fix dead code warning during build in netdog ([#1949])

## Documentation Changes

* Correct variable name in bootstrap-containers/README.md ([#1959], thanks, @dangen-effy!)
* Add art to the console ([#1970])

[a8e4a20ca7d1]: https://github.com/bottlerocket-os/bottlerocket/commit/a8e4a20ca7d1dde4e8b5f679e4e11d9687b6ef09
[3d0c10abeecb]: https://github.com/bottlerocket-os/bottlerocket/commit/3d0c10abeecb9f69b6ec598fd5137cb146a46b6e
[#1728]: https://github.com/bottlerocket-os/bottlerocket/pull/1728
[#1948]: https://github.com/bottlerocket-os/bottlerocket/pull/1948
[#1949]: https://github.com/bottlerocket-os/bottlerocket/pull/1949
[#1953]: https://github.com/bottlerocket-os/bottlerocket/pull/1953
[#1955]: https://github.com/bottlerocket-os/bottlerocket/pull/1955
[#1959]: https://github.com/bottlerocket-os/bottlerocket/pull/1959
[#1962]: https://github.com/bottlerocket-os/bottlerocket/pull/1962
[#1970]: https://github.com/bottlerocket-os/bottlerocket/pull/1970

# v1.6.1 (2022-03-02)

## Security Fixes

* Apply patch to containerd for CVE-2022-23648 ([0de1b39efa64][0de1b39efa64])
* Update kernel-5.4 and kernel-5.10 to include recent security fixes ([#1973])

[0de1b39efa64]: https://github.com/bottlerocket-os/bottlerocket/commit/0de1b39efa6437fa57388918e1554174ca2f02e4
[#1973]: https://github.com/bottlerocket-os/bottlerocket/pull/1973

# v1.6.0 (2022-02-07)

## Deprecation Notice

The Kubernetes 1.18 variant, `aws-k8s-1.18`, will lose support in March 2022.
Kubernetes 1.18 is no longer receiving support upstream.
We recommend replacing `aws-k8s-1.18` nodes with a later variant, preferably `aws-k8s-1.21` if your cluster supports it.
See [this issue](https://github.com/bottlerocket-os/bottlerocket/issues/1942) for more details.

## Security Fixes

* Apply patch to the kernel for CVE-2022-0492 ([#1943])

## OS Changes
* Add aws-k8s-1.21-nvidia variant with Nvidia driver support ([#1859], [#1860], [#1861], [#1862], [#1900], [#1912], [#1915], [#1916], [#1928])
* Add metal-k8s-1.21 variant with support for running on bare metal ([#1904])
* Update host containers to the latest version ([#1939])
* Add driverdog, a configuration-driven utility for linking kernel modules at runtime ([#1867])
* Kubernetes: Fix a potential inconsistency with IPv6 node-ip comparisons ([#1932])
* Allow setting multiple Kubernetes node taints with the same key ([#1906])
* Fix a bug which would prevent Bottlerocket from booting when setting `container-registry` to an empty table ([#1910])
* Add `/etc/bottlerocket-release` to host containers ([#1883])
* Send grub output to the local console on BIOS systems ([#1894])
* Fix minor issues with systemd units ([#1889])

## Build Changes
* Update third-party packages ([#1936])
* Update Rust dependencies ([#1940])
* Update Go dependencies of `host-ctr` ([#1938])
* Add the ability to fetch licenses at build time ([#1901])
* Pin tuftool to a specific version ([#1940])

## Documentation Changes
* Add a no-proxy setting example to the README ([#1765] thanks, @mrajashree!)
* Document variant `image-layout` options in the README ([#1896])


[#1765]: https://github.com/bottlerocket-os/bottlerocket/pull/1765
[#1859]: https://github.com/bottlerocket-os/bottlerocket/pull/1859
[#1860]: https://github.com/bottlerocket-os/bottlerocket/pull/1860
[#1861]: https://github.com/bottlerocket-os/bottlerocket/pull/1861
[#1862]: https://github.com/bottlerocket-os/bottlerocket/pull/1862
[#1867]: https://github.com/bottlerocket-os/bottlerocket/pull/1867
[#1883]: https://github.com/bottlerocket-os/bottlerocket/pull/1883
[#1889]: https://github.com/bottlerocket-os/bottlerocket/pull/1889
[#1894]: https://github.com/bottlerocket-os/bottlerocket/pull/1894
[#1896]: https://github.com/bottlerocket-os/bottlerocket/pull/1896
[#1900]: https://github.com/bottlerocket-os/bottlerocket/pull/1900
[#1901]: https://github.com/bottlerocket-os/bottlerocket/pull/1901
[#1904]: https://github.com/bottlerocket-os/bottlerocket/pull/1904
[#1906]: https://github.com/bottlerocket-os/bottlerocket/pull/1906
[#1910]: https://github.com/bottlerocket-os/bottlerocket/pull/1910
[#1912]: https://github.com/bottlerocket-os/bottlerocket/pull/1912
[#1915]: https://github.com/bottlerocket-os/bottlerocket/pull/1915
[#1916]: https://github.com/bottlerocket-os/bottlerocket/pull/1916
[#1928]: https://github.com/bottlerocket-os/bottlerocket/pull/1928
[#1932]: https://github.com/bottlerocket-os/bottlerocket/pull/1932
[#1936]: https://github.com/bottlerocket-os/bottlerocket/pull/1936
[#1938]: https://github.com/bottlerocket-os/bottlerocket/pull/1938
[#1939]: https://github.com/bottlerocket-os/bottlerocket/pull/1939
[#1940]: https://github.com/bottlerocket-os/bottlerocket/pull/1940
[#1943]: https://github.com/bottlerocket-os/bottlerocket/pull/1943

# v1.5.3 (2022-01-25)

## Security Fixes
* Update Bottlerocket SDK to 0.25.1 for Rust 1.58.1 ([#1918])
* Update kernel-5.4 and kernel-5.10 to include recent security fixes ([#1921])
* Migrate host-container to the latest version for vmware variants ([#1898])

## OS Changes
* Fix an issue which could impair nodes in Kubernetes 1.21 IPv6 clusters ([#1925])

[#1898]: https://github.com/bottlerocket-os/bottlerocket/pull/1898
[#1918]: https://github.com/bottlerocket-os/bottlerocket/pull/1918
[#1921]: https://github.com/bottlerocket-os/bottlerocket/pull/1921
[#1925]: https://github.com/bottlerocket-os/bottlerocket/pull/1925

# v1.5.2 (2022-01-05)

## Security Fixes
* Update containerd for CVE-2021-43816 ([8f085929588a][8f085929588a])

[8f085929588a]: https://github.com/bottlerocket-os/bottlerocket/commit/8f085929588a3f0cd575f865dd6f04f96a97e923

# v1.5.1 (2021-12-23)

## Security Fixes
* Update hotdog to the latest release. Hotdog now mimics the permissions of the target JVM process ([#1884])

## OS Changes
* Updated host containers to the latest version ([#1881], [#1882])

[#1881]: https://github.com/bottlerocket-os/bottlerocket/pull/1881
[#1882]: https://github.com/bottlerocket-os/bottlerocket/pull/1882
[#1884]: https://github.com/bottlerocket-os/bottlerocket/pull/1884

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
* Add guestinfo to available VMware user data retrieval methods ([#1393])
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
* Update GitHub Actions to ignore changes that only include .md files ([#1274])

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
