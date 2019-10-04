%global workspace_name api
%global workspace_dir %{_builddir}/workspaces/%{workspace_name}
%global migration_dir %{_cross_factorydir}%{_cross_sharedstatedir}/thar/datastore/migrations
%undefine _debugsource_packages

# List migrations to be installed here, eg:
#%%global migration_versions v1.0 v1.1 v1.2
%global migration_versions %{nil}

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar API packages
License: Apache-2.0 AND (Apache-2.0 OR BSL-1.0) AND (Apache-2.0 OR MIT) AND Apache-2.0/MIT AND BSD-2-Clause AND BSD-3-Clause AND CC0-1.0 AND ISC AND MIT AND (MIT OR Apache-2.0) AND MIT/Unlicense AND N/A AND (Unlicense OR MIT) AND Zlib
Source1: apiserver.service
Source2: moondog.service
Source3: sundog.service
Source4: storewolf.service
Source5: settings-committer.service
Source6: migration-tmpfiles.conf
Source7: settings-applier.service
Source8: data-store-version
Source9: migrator.service
Source10: api-sysusers.conf
Source11: host-containers@.service
Source12: host-containers-tmpfiles.conf
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}systemd-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%package -n %{_cross_os}apiserver
Summary: Thar API server
%description -n %{_cross_os}apiserver
%{summary}.

%package -n %{_cross_os}apiclient
Summary: Thar API client
%description -n %{_cross_os}apiclient
%{summary}.

%package -n %{_cross_os}moondog
Summary: Thar userdata configuration system
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}moondog
%{summary}.

%package -n %{_cross_os}netdog
Summary: Thar network configuration helper
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}netdog
%{summary}.

%package -n %{_cross_os}sundog
Summary: Updates settings dynamically based on user-specified generators
Requires: %{_cross_os}apiserver = %{version}-%{release}
Requires: %{_cross_os}pluto = %{version}-%{release}
Requires: %{_cross_os}bork = %{version}-%{release}
%description -n %{_cross_os}sundog
%{summary}.

%package -n %{_cross_os}bork
Summary: Dynamic setting generator for updog
%description -n %{_cross_os}bork
%{summary}.

%package -n %{_cross_os}pluto
Summary: Dynamic setting generator for kubernetes
%description -n %{_cross_os}pluto
%{summary}.

%package -n %{_cross_os}thar-be-settings
Summary: Applies changed settings to a Thar system
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}thar-be-settings
%{summary}.

%package -n %{_cross_os}servicedog
Summary: Manipulates systemd units based on setting changes
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}servicedog
%{summary}.

%package -n %{_cross_os}host-containers
Summary: Manages system- and user-defined host containers
Requires: %{_cross_os}apiserver = %{version}-%{release}
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

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --path %{workspace_dir}/apiserver --features sd_notify

for p in \
  apiclient \
  moondog netdog sundog pluto bork \
  thar-be-settings servicedog host-containers \
  storewolf settings-committer \
  migration/migrator ;
do
  %cargo_build --path %{workspace_dir}/${p}
done

for v in %migration_versions ; do
  for p in %{workspace_dir}/migration/migrations/${v}/* ; do
    name="${p##*/}"
    %cargo_build --path ${p}
    mv bin/${name} bin/migrate_${v}_${name}
  done
done

%install
install -d %{buildroot}%{_cross_bindir}
for p in \
  apiclient apiserver \
  moondog netdog sundog pluto bork \
  thar-be-settings servicedog host-containers \
  storewolf settings-committer \
  migrator ;
do
  install -p -m 0755 bin/${p} %{buildroot}%{_cross_bindir}
done

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 \
  %{S:1} %{S:2} %{S:3} %{S:4} %{S:5} %{S:7} %{S:9} %{S:11} \
  %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_datadir}/thar
install -p -m 0644 %{S:8} %{buildroot}%{_cross_datadir}/thar

install -d %{buildroot}%{migration_dir}
for m in bin/migrate_* ; do
  [ -f "${m}" ] || continue
  install -p -m0755 ${m} %{buildroot}%{migration_dir}
done

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:6} %{buildroot}%{_cross_tmpfilesdir}/migration.conf
install -p -m 0644 %{S:12} %{buildroot}%{_cross_tmpfilesdir}/host-containers.conf

install -d %{buildroot}%{_cross_sysusersdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_sysusersdir}/api.conf

%files -n %{_cross_os}apiserver
%{_cross_bindir}/apiserver
%{_cross_unitdir}/apiserver.service
%{_cross_unitdir}/migrator.service
%{_cross_datadir}/thar/data-store-version
%{_cross_sysusersdir}/api.conf

%files -n %{_cross_os}apiclient
%{_cross_bindir}/apiclient

%files -n %{_cross_os}moondog
%{_cross_bindir}/moondog
%{_cross_unitdir}/moondog.service

%files -n %{_cross_os}netdog
%{_cross_bindir}/netdog

%files -n %{_cross_os}sundog
%{_cross_bindir}/sundog
%{_cross_unitdir}/sundog.service

%files -n %{_cross_os}pluto
%{_cross_bindir}/pluto

%files -n %{_cross_os}bork
%{_cross_bindir}/bork

%files -n %{_cross_os}thar-be-settings
%{_cross_bindir}/thar-be-settings
%{_cross_unitdir}/settings-applier.service

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
%{migration_dir}
%{_cross_tmpfilesdir}/migration.conf

%files -n %{_cross_os}settings-committer
%{_cross_bindir}/settings-committer
%{_cross_unitdir}/settings-committer.service

%changelog
