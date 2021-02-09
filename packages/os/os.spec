%global _cross_first_party 1
%global _is_k8s_variant %(if echo %{_cross_variant} | grep -q "k8s"; then echo 1; else echo 0; fi)
%undefine _debugsource_packages

Name: %{_cross_os}os
Version: 0.0
Release: 0%{?dist}
Summary: Bottlerocket's first-party code
License: Apache-2.0 OR MIT

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

# 2xx sources: tmpfilesd configs
Source200: migration-tmpfiles.conf
Source201: host-containers-tmpfiles.conf
Source202: thar-be-updates-tmpfiles.conf

# 3xx sources: udev rules
Source300: ephemeral-storage.rules

BuildRequires: %{_cross_os}glibc-devel

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
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}early-boot-config
%{summary}.

%package -n %{_cross_os}netdog
Summary: Bottlerocket network configuration helper
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}netdog
%{summary}.

%package -n %{_cross_os}sundog
Summary: Updates settings dynamically based on user-specified generators
Requires: %{_cross_os}apiserver = %{version}-%{release}
Requires: %{_cross_os}schnauzer = %{version}-%{release}
Requires: %{_cross_os}pluto = %{version}-%{release}
Requires: %{_cross_os}bork = %{version}-%{release}
%description -n %{_cross_os}sundog
%{summary}.

%package -n %{_cross_os}bork
Summary: Dynamic setting generator for updog
%description -n %{_cross_os}bork
%{summary}.

%package -n %{_cross_os}corndog
Summary: Bottlerocket sysctl helper
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}corndog
%{summary}.

%package -n %{_cross_os}schnauzer
Summary: Setting generator for templated settings values.
%description -n %{_cross_os}schnauzer
%{summary}.

%package -n %{_cross_os}pluto
Summary: Dynamic setting generator for kubernetes
%description -n %{_cross_os}pluto
%{summary}.

%package -n %{_cross_os}thar-be-settings
Summary: Applies changed settings to a Bottlerocket system
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}thar-be-settings
%{summary}.

%package -n %{_cross_os}thar-be-updates
Summary: Dispatches Bottlerocket update commands
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}thar-be-updates
%{summary}.

%package -n %{_cross_os}servicedog
Summary: Manipulates systemd units based on setting changes
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}servicedog
%{summary}.

%package -n %{_cross_os}host-containers
Summary: Manages system- and user-defined host containers
Requires: %{_cross_os}apiserver = %{version}-%{release}
Requires: %{_cross_os}host-ctr
%description -n %{_cross_os}host-containers
%{summary}.

%package -n %{_cross_os}storewolf
Summary: Data store creator
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}storewolf
%{summary}.

%package -n %{_cross_os}migration
Summary: Tools to migrate version formats
Requires: %{_cross_os}apiserver = %{version}-%{release}
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

%if "%{_cross_variant}" == "aws-ecs-1"
%package -n %{_cross_os}ecs-settings-applier
Summary: Settings generator for ECS
%description -n %{_cross_os}ecs-settings-applier
%{summary}.
%endif

%if %{_is_k8s_variant}
%package -n %{_cross_os}static-pods
Summary: Manages user-defined K8S static pods
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}static-pods
%{summary}.
%endif

%prep
%setup -T -c
%cargo_prep

%build
mkdir bin
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p apiserver \
    -p early-boot-config \
    -p netdog \
    -p sundog \
    -p schnauzer \
    -p pluto \
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
%if "%{_cross_variant}" == "aws-ecs-1"
    -p ecs-settings-applier \
%endif
%if %{_is_k8s_variant}
    -p static-pods \
%endif
    %{nil}

# Next, build components that should be static.
# * apiclient, because it needs to run from containers that don't have the same libraries available.
# * migrations, because they need to run after a system update where available libraries can change.

# First we find the migrations in the source tree.  We assume the directory name is the same as the crate name.
migrations=()
for migration in $(find %{_builddir}/sources/api/migration/migrations/* -mindepth 1 -maxdepth 1 -type d); do
    migrations+=("-p $(basename ${migration})")
done
# Build static binaries.
%cargo_build_static --manifest-path %{_builddir}/sources/Cargo.toml \
    -p apiclient \
    ${migrations[*]} \
    %{nil}

%install
install -d %{buildroot}%{_cross_bindir}
for p in \
  apiserver \
  early-boot-config netdog sundog schnauzer pluto bork corndog \
  thar-be-settings thar-be-updates servicedog host-containers \
  storewolf settings-committer \
  migrator \
  signpost updog metricdog logdog \
  ghostdog \
%if "%{_cross_variant}" == "aws-ecs-1"
  ecs-settings-applier \
%endif
%if %{_is_k8s_variant}
  static-pods \
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
for version_path in %{_builddir}/sources/api/migration/migrations/*; do
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

install -d %{buildroot}%{_cross_datadir}/eks
install -p -m 0644 %{S:3} %{buildroot}%{_cross_datadir}/eks

install -d %{buildroot}%{_cross_datadir}/updog
install -p -m 0644 %{_cross_repo_root_json} %{buildroot}%{_cross_datadir}/updog

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:5} %{S:6} %{buildroot}%{_cross_templatedir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 \
  %{S:100} %{S:101} %{S:102} %{S:103} %{S:105} \
  %{S:106} %{S:107} %{S:110} %{S:111} %{S:112} \
  %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:200} %{buildroot}%{_cross_tmpfilesdir}/migration.conf
install -p -m 0644 %{S:201} %{buildroot}%{_cross_tmpfilesdir}/host-containers.conf
install -p -m 0644 %{S:202} %{buildroot}%{_cross_tmpfilesdir}/thar-be-updates.conf

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

%files -n %{_cross_os}corndog
%{_cross_bindir}/corndog

%files -n %{_cross_os}sundog
%{_cross_bindir}/sundog
%{_cross_unitdir}/sundog.service

%files -n %{_cross_os}schnauzer
%{_cross_bindir}/schnauzer

%files -n %{_cross_os}pluto
%{_cross_bindir}/pluto
%dir %{_cross_datadir}/eks
%{_cross_datadir}/eks/eni-max-pods

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

%files -n %{_cross_os}logdog
%{_cross_bindir}/logdog

%if "%{_cross_variant}" == "aws-ecs-1"
%files -n %{_cross_os}ecs-settings-applier
%{_cross_bindir}/ecs-settings-applier
%endif

%if %{_is_k8s_variant}
%files -n %{_cross_os}static-pods
%{_cross_bindir}/static-pods
%endif

%changelog
