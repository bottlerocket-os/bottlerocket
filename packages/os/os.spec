%global _cross_first_party 1
%global _is_k8s_variant %(if echo %{_cross_variant} | grep -Fqw "k8s"; then echo 1; else echo 0; fi)
%global _is_ecs_variant %(if echo %{_cross_variant} | grep -Fqw "ecs"; then echo 1; else echo 0; fi)
%global _is_aws_variant %(if echo %{_cross_variant} | grep -Fqw "aws"; then echo 1; else echo 0; fi)
%global _is_vendor_variant %(if echo %{_cross_variant} | grep -Fqw "nvidia"; then echo 1; else echo 0; fi)
%undefine _debugsource_packages

Name: %{_cross_os}os
Version: 0.0
Release: 0%{?dist}
Summary: Bottlerocket's first-party code
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

# sources < 100: misc
Source2: api-sysusers.conf
# Taken from https://github.com/awslabs/amazon-eks-ami/blob/master/files/eni-max-pods.txt
Source3: eni-max-pods

# Note: root.json is copied into place by Dockerfile and its path is made
# available with the _cross_repo_root_json macro, so we don't list it as a
# source here.  The alternative is copying/mapping it into the SOURCES
# directory, but then root.json would be present in all package builds,
# potentially causing a conflict.
#SourceX: root.json

Source5: updog-toml
Source6: metricdog-toml
Source7: host-ctr-toml
Source8: oci-default-hooks-json
Source9: cfsignal-toml

# 1xx sources: systemd units
Source100: apiserver.service
Source101: early-boot-config.service
Source102: sundog.service
Source103: storewolf.service
Source105: settings-applier.service
Source106: migrator.service
Source107: host-containers@.service
Source110: mark-successful-boot.service
Source111: metricdog.service
Source112: metricdog.timer
Source113: send-boot-success.service
Source114: bootstrap-containers@.service
Source115: link-kernel-modules.service
Source116: load-kernel-modules.service
Source117: cfsignal.service
Source118: generate-network-config.service
Source119: prepare-primary-interface.service

# 2xx sources: tmpfilesd configs
Source200: migration-tmpfiles.conf
Source201: host-containers-tmpfiles.conf
Source202: thar-be-updates-tmpfiles.conf
Source203: bootstrap-containers-tmpfiles.conf
Source204: netdog-tmpfiles.conf

# 3xx sources: udev rules
Source300: ephemeral-storage.rules

BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}apiclient
Requires: %{_cross_os}apiserver
Requires: %{_cross_os}bootstrap-containers
Requires: %{_cross_os}bork
Requires: %{_cross_os}corndog
Requires: %{_cross_os}certdog
Requires: %{_cross_os}early-boot-config
Requires: %{_cross_os}ghostdog
Requires: %{_cross_os}growpart
Requires: %{_cross_os}host-containers
Requires: %{_cross_os}logdog
Requires: %{_cross_os}metricdog
Requires: %{_cross_os}migration
Requires: %{_cross_os}netdog
Requires: %{_cross_os}prairiedog
Requires: %{_cross_os}schnauzer
Requires: %{_cross_os}settings-committer
Requires: %{_cross_os}signpost
Requires: %{_cross_os}storewolf
Requires: %{_cross_os}sundog
Requires: %{_cross_os}thar-be-settings
Requires: %{_cross_os}thar-be-updates
Requires: %{_cross_os}updog
Requires: %{_cross_os}shimpei

%if %{_is_k8s_variant}
%if %{_is_aws_variant}
Requires: %{_cross_os}pluto
%endif
Requires: %{_cross_os}static-pods
%endif

%if %{_is_aws_variant}
Requires: %{_cross_os}shibaken
Requires: %{_cross_os}cfsignal
%endif

%if %{_is_ecs_variant}
Requires: %{_cross_os}ecs-settings-applier
%endif

%if %{_is_vendor_variant}
Requires: %{_cross_os}driverdog
%endif

%description
%{summary}.

%package -n %{_cross_os}apiserver
Summary: Bottlerocket API server
%description -n %{_cross_os}apiserver
%{summary}.

%package -n %{_cross_os}apiclient
Summary: Bottlerocket API client
%description -n %{_cross_os}apiclient
%{summary}.

%package -n %{_cross_os}early-boot-config
Summary: Bottlerocket userdata configuration system
%description -n %{_cross_os}early-boot-config
%{summary}.

%package -n %{_cross_os}netdog
Summary: Bottlerocket network configuration helper
%description -n %{_cross_os}netdog
%{summary}.

%package -n %{_cross_os}sundog
Summary: Updates settings dynamically based on user-specified generators
%description -n %{_cross_os}sundog
%{summary}.

%package -n %{_cross_os}bork
Summary: Dynamic setting generator for updog
%description -n %{_cross_os}bork
%{summary}.

%package -n %{_cross_os}corndog
Summary: Bottlerocket sysctl helper
%description -n %{_cross_os}corndog
%{summary}.

%package -n %{_cross_os}schnauzer
Summary: Setting generator for templated settings values.
%description -n %{_cross_os}schnauzer
%{summary}.

%package -n %{_cross_os}thar-be-settings
Summary: Applies changed settings to a Bottlerocket system
%description -n %{_cross_os}thar-be-settings
%{summary}.

%package -n %{_cross_os}thar-be-updates
Summary: Dispatches Bottlerocket update commands
%description -n %{_cross_os}thar-be-updates
%{summary}.

%package -n %{_cross_os}servicedog
Summary: Manipulates systemd units based on setting changes
%description -n %{_cross_os}servicedog
%{summary}.

%package -n %{_cross_os}host-containers
Summary: Manages system- and user-defined host containers
Requires: %{_cross_os}host-ctr
%description -n %{_cross_os}host-containers
%{summary}.

%package -n %{_cross_os}storewolf
Summary: Data store creator
%description -n %{_cross_os}storewolf
%{summary}.

%package -n %{_cross_os}migration
Summary: Tools to migrate version formats
%description -n %{_cross_os}migration

%package -n %{_cross_os}settings-committer
Summary: Commits settings from user data, defaults, and generators at boot
%description -n %{_cross_os}settings-committer
%{summary}.

%package -n %{_cross_os}ghostdog
Summary: Tool to manage ephemeral disks
%description -n %{_cross_os}ghostdog
%{summary}.

%package -n %{_cross_os}growpart
Summary: Tool to grow partitions
%description -n %{_cross_os}growpart
%{summary}.

%package -n %{_cross_os}signpost
Summary: Bottlerocket GPT priority querier/switcher
%description -n %{_cross_os}signpost
%{summary}.

%package -n %{_cross_os}updog
Summary: Bottlerocket updater CLI
%description -n %{_cross_os}updog
not much what's up with you

%package -n %{_cross_os}metricdog
Summary: Bottlerocket health metrics sender
%description -n %{_cross_os}metricdog
%{summary}.

%package -n %{_cross_os}logdog
Summary: Bottlerocket log extractor
%description -n %{_cross_os}logdog
use logdog to extract logs from the Bottlerocket host

%package -n %{_cross_os}migrations
Summary: Thar data store migrations
%description -n %{_cross_os}migrations
%{summary}.

%package -n %{_cross_os}prairiedog
Summary: Tools for kdump support
Requires: %{_cross_os}kexec-tools
Requires: %{_cross_os}makedumpfile
%description -n %{_cross_os}prairiedog
%{summary}.

%package -n %{_cross_os}certdog
Summary: Bottlerocket certificates handler
%description -n %{_cross_os}certdog
%{summary}.

%if %{_is_ecs_variant}
%package -n %{_cross_os}ecs-settings-applier
Summary: Settings generator for ECS
%description -n %{_cross_os}ecs-settings-applier
%{summary}.
%endif

%if %{_is_k8s_variant}
%if %{_is_aws_variant}
%package -n %{_cross_os}pluto
Summary: Dynamic setting generator for kubernetes
%description -n %{_cross_os}pluto
%{summary}.
%endif

%package -n %{_cross_os}static-pods
Summary: Manages user-defined K8S static pods
%description -n %{_cross_os}static-pods
%{summary}.
%endif

%if %{_is_aws_variant}
%package -n %{_cross_os}shibaken
Summary: Setting generator for populating admin container user-data from IMDS.
%description -n %{_cross_os}shibaken
%{summary}.

%package -n %{_cross_os}cfsignal
Summary: Bottlerocket CloudFormation Stack signaler
%description -n %{_cross_os}cfsignal
%{summary}.
%endif

%package -n %{_cross_os}shimpei
Summary: OCI-compatible shim around oci-add-hooks
Requires: %{_cross_os}oci-add-hooks
Requires: %{_cross_os}hotdog
%description -n %{_cross_os}shimpei
%{summary}.

%if %{_is_vendor_variant}
%package -n %{_cross_os}driverdog
Summary: Tool to load additional drivers
Requires: %{_cross_os}binutils
%description -n %{_cross_os}driverdog
%{summary}.
%endif

%package -n %{_cross_os}bootstrap-containers
Summary: Manages bootstrap-containers
%description -n %{_cross_os}bootstrap-containers
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
mkdir bin

# We want to build some components statically:
# * apiclient, because it needs to run from containers that don't have the same libraries available.
# * migrations, because they need to run after a system update where available libraries can change.
#
# Most of our components don't need to be static, though.  This means we run cargo once for static
# and once for non-static.  There's a long tail of crate builds for each of these that can be
# mitigated by running them in parallel, saving a fair amount of time.  To do this, we kick off the
# static build in the background, run the non-static (main) build in the foreground, and then wait
# for the static build and print its output afterward.  A failure of either will stop the build.

# For static builds, first we find the migrations in the source tree.  We assume the directory name
# is the same as the crate name.
migrations=()
for migration in $(find %{_builddir}/sources/api/migration/migrations/v[0-9]* -mindepth 1 -maxdepth 1 -type d); do
    migrations+=("-p $(basename ${migration})")
done
# Store the output so we can print it after waiting for the backgrounded job.
static_output="$(mktemp)"
# Build static binaries in the background.
%cargo_build_static --manifest-path %{_builddir}/sources/Cargo.toml \
    -p apiclient \
    ${migrations[*]} \
    >> ${static_output} 2>&1 &
# Save the PID so we can wait for it later.
static_pid="$!"

# Run non-static builds in the foreground.
echo "** Output from non-static builds:"
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p apiserver \
    -p early-boot-config \
    -p netdog \
    -p sundog \
    -p schnauzer \
    -p bork \
    -p thar-be-settings \
    -p thar-be-updates \
    -p servicedog \
    -p host-containers \
    -p storewolf \
    -p settings-committer \
    -p migrator \
    -p signpost \
    -p updog \
    -p logdog \
    -p metricdog \
    -p ghostdog \
    -p growpart \
    -p corndog \
    -p bootstrap-containers \
    -p prairiedog \
    -p certdog \
    -p shimpei \
%if %{_is_ecs_variant}
    -p ecs-settings-applier \
%endif
%if %{_is_aws_variant}
    -p shibaken \
    -p cfsignal \
%endif
%if %{_is_k8s_variant}
%if %{_is_aws_variant}
    -p pluto \
%endif
    -p static-pods \
%endif
%if %{_is_vendor_variant}
    -p driverdog \
%endif
    %{nil}

# Wait for static builds from the background, if they're not already done.
set +e; wait "${static_pid}"; static_rc="${?}"; set -e
echo -e "\n** Output from static builds:"
cat "${static_output}"
if [ "${static_rc}" -ne 0 ]; then
   exit "${static_rc}"
fi

%install
install -d %{buildroot}%{_cross_bindir}
for p in \
  apiserver \
  early-boot-config netdog sundog schnauzer bork corndog \
  thar-be-settings thar-be-updates servicedog host-containers \
  storewolf settings-committer \
  migrator prairiedog certdog \
  signpost updog metricdog logdog \
  ghostdog bootstrap-containers \
  shimpei \
%if %{_is_ecs_variant}
  ecs-settings-applier \
%endif
%if %{_is_aws_variant}
  shibaken \
  cfsignal \
%endif
%if %{_is_k8s_variant}
%if %{_is_aws_variant}
  pluto \
%endif
  static-pods \
%endif
%if %{_is_vendor_variant}
  driverdog \
%endif
; do
  install -p -m 0755 ${HOME}/.cache/%{__cargo_target}/release/${p} %{buildroot}%{_cross_bindir}
done

for p in apiclient ; do
  install -p -m 0755 ${HOME}/.cache/.static/%{__cargo_target_static}/release/${p} %{buildroot}%{_cross_bindir}
done

install -d %{buildroot}%{_cross_sbindir}
for p in growpart ; do
  install -p -m 0755 ${HOME}/.cache/%{__cargo_target}/release/${p} %{buildroot}%{_cross_sbindir}
done

install -d %{buildroot}%{_cross_datadir}/migrations
for version_path in %{_builddir}/sources/api/migration/migrations/v[0-9]*; do
  [ -e "${version_path}" ] || continue
  for migration_path in "${version_path}"/*; do
    [ -e "${migration_path}" ] || continue

    version="${version_path##*/}"
    crate_name="${migration_path##*/}"
    migration_binary_name="migrate_${version}_${crate_name#migrate-}"
    built_path="${HOME}/.cache/.static/%{__cargo_target_static}/release/${crate_name}"
    target_path="%{buildroot}%{_cross_datadir}/migrations/${migration_binary_name}"

    install -m 0555 "${built_path}" "${target_path}"
  done
done

install -d %{buildroot}%{_cross_datadir}/bottlerocket

install -d %{buildroot}%{_cross_sysusersdir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_sysusersdir}/api.conf

%if %{_is_k8s_variant}
%if %{_is_aws_variant}
install -d %{buildroot}%{_cross_datadir}/eks
install -p -m 0644 %{S:3} %{buildroot}%{_cross_datadir}/eks
%endif
%endif

install -d %{buildroot}%{_cross_datadir}/updog
install -p -m 0644 %{_cross_repo_root_json} %{buildroot}%{_cross_datadir}/updog

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:5} %{S:6} %{S:7} %{S:8} %{buildroot}%{_cross_templatedir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 \
  %{S:100} %{S:101} %{S:102} %{S:103} %{S:105} \
  %{S:106} %{S:107} %{S:110} %{S:111} %{S:112} \
  %{S:113} %{S:114} %{S:118} %{S:119} \
%if %{_is_vendor_variant}
  %{S:115} %{S:116} \
%endif
  %{buildroot}%{_cross_unitdir}

%if %{_is_aws_variant}
install -p -m 0644 %{S:9} %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:117} %{buildroot}%{_cross_unitdir}
%endif

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:200} %{buildroot}%{_cross_tmpfilesdir}/migration.conf
install -p -m 0644 %{S:201} %{buildroot}%{_cross_tmpfilesdir}/host-containers.conf
install -p -m 0644 %{S:202} %{buildroot}%{_cross_tmpfilesdir}/thar-be-updates.conf
install -p -m 0644 %{S:203} %{buildroot}%{_cross_tmpfilesdir}/bootstrap-containers.conf
install -p -m 0644 %{S:204} %{buildroot}%{_cross_tmpfilesdir}/netdog.conf

install -d %{buildroot}%{_cross_udevrulesdir}
install -p -m 0644 %{S:300} %{buildroot}%{_cross_udevrulesdir}/80-ephemeral-storage.rules

%cross_scan_attribution --clarify %{_builddir}/sources/clarify.toml \
    cargo --offline --locked %{_builddir}/sources/Cargo.toml

%files
%{_cross_attribution_vendor_dir}

%files -n %{_cross_os}apiserver
%{_cross_bindir}/apiserver
%{_cross_unitdir}/apiserver.service
%{_cross_unitdir}/migrator.service
%{_cross_sysusersdir}/api.conf

%files -n %{_cross_os}apiclient
%{_cross_bindir}/apiclient

%files -n %{_cross_os}early-boot-config
%{_cross_bindir}/early-boot-config
%{_cross_unitdir}/early-boot-config.service

%files -n %{_cross_os}netdog
%{_cross_bindir}/netdog
%{_cross_tmpfilesdir}/netdog.conf
%{_cross_unitdir}/generate-network-config.service
%{_cross_unitdir}/prepare-primary-interface.service

%files -n %{_cross_os}corndog
%{_cross_bindir}/corndog

%files -n %{_cross_os}sundog
%{_cross_bindir}/sundog
%{_cross_unitdir}/sundog.service

%files -n %{_cross_os}schnauzer
%{_cross_bindir}/schnauzer

%files -n %{_cross_os}bork
%{_cross_bindir}/bork

%files -n %{_cross_os}thar-be-settings
%{_cross_bindir}/thar-be-settings
%{_cross_unitdir}/settings-applier.service

%files -n %{_cross_os}thar-be-updates
%{_cross_bindir}/thar-be-updates
%{_cross_tmpfilesdir}/thar-be-updates.conf

%files -n %{_cross_os}servicedog
%{_cross_bindir}/servicedog

%files -n %{_cross_os}host-containers
%{_cross_bindir}/host-containers
%{_cross_unitdir}/host-containers@.service
%{_cross_tmpfilesdir}/host-containers.conf
%dir %{_cross_templatedir}
%{_cross_templatedir}/host-ctr-toml

%files -n %{_cross_os}storewolf
%{_cross_bindir}/storewolf
%{_cross_unitdir}/storewolf.service

%files -n %{_cross_os}migration
%{_cross_bindir}/migrator
%{_cross_tmpfilesdir}/migration.conf

%files -n %{_cross_os}migrations
%dir %{_cross_datadir}/migrations
%{_cross_datadir}/migrations

%files -n %{_cross_os}settings-committer
%{_cross_bindir}/settings-committer

%files -n %{_cross_os}ghostdog
%{_cross_bindir}/ghostdog
%{_cross_udevrulesdir}/80-ephemeral-storage.rules

%files -n %{_cross_os}growpart
%{_cross_sbindir}/growpart

%files -n %{_cross_os}signpost
%{_cross_bindir}/signpost
%{_cross_unitdir}/mark-successful-boot.service

%files -n %{_cross_os}updog
%{_cross_bindir}/updog
%{_cross_datadir}/updog
%dir %{_cross_templatedir}
%{_cross_templatedir}/updog-toml

%files -n %{_cross_os}metricdog
%{_cross_bindir}/metricdog
%dir %{_cross_templatedir}
%{_cross_templatedir}/metricdog-toml
%{_cross_unitdir}/metricdog.service
%{_cross_unitdir}/metricdog.timer
%{_cross_unitdir}/send-boot-success.service

%files -n %{_cross_os}logdog
%{_cross_bindir}/logdog

%if %{_is_ecs_variant}
%files -n %{_cross_os}ecs-settings-applier
%{_cross_bindir}/ecs-settings-applier
%endif

%if %{_is_aws_variant}
%files -n %{_cross_os}shibaken
%{_cross_bindir}/shibaken

%files -n %{_cross_os}cfsignal
%{_cross_bindir}/cfsignal
%dir %{_cross_templatedir}
%{_cross_templatedir}/cfsignal-toml
%{_cross_unitdir}/cfsignal.service
%endif

%if %{_is_vendor_variant}
%files -n %{_cross_os}driverdog
%{_cross_bindir}/driverdog
%{_cross_unitdir}/link-kernel-modules.service
%{_cross_unitdir}/load-kernel-modules.service
%endif

%if %{_is_k8s_variant}
%if %{_is_aws_variant}
%files -n %{_cross_os}pluto
%{_cross_bindir}/pluto
%dir %{_cross_datadir}/eks
%{_cross_datadir}/eks/eni-max-pods
%endif

%files -n %{_cross_os}static-pods
%{_cross_bindir}/static-pods
%endif

%files -n %{_cross_os}shimpei
%{_cross_bindir}/shimpei
%{_cross_templatedir}/oci-default-hooks-json

%files -n %{_cross_os}prairiedog
%{_cross_bindir}/prairiedog

%files -n %{_cross_os}certdog
%{_cross_bindir}/certdog

%files -n %{_cross_os}bootstrap-containers
%{_cross_bindir}/bootstrap-containers
%{_cross_unitdir}/bootstrap-containers@.service
%{_cross_tmpfilesdir}/bootstrap-containers.conf

%changelog
