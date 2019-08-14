%global workspace_name api
%global systemd_systemdir %{_cross_libdir}/systemd/system
%global migrationdir %{_cross_factorydir}%{_cross_sharedstatedir}/thar/datastore/migrations

# List migrations to be installed here, eg:
#%%global migration_versions v1.0 v1.1 v1.2
%global migration_versions %{nil}

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar API packages
License: Apache-2.0 AND (Apache-2.0 OR BSL-1.0) AND (Apache-2.0 OR MIT) AND Apache-2.0/MIT AND BSD-2-Clause AND BSD-3-Clause AND CC0-1.0 AND ISC AND MIT AND (MIT OR Apache-2.0) AND MIT/Unlicense AND N/A AND (Unlicense OR MIT) AND Zlib
Source0: %{workspace_name}.crate
Source1: apiserver.service
Source2: moondog.service
Source3: sundog.service
Source4: storewolf.service
Source5: migration-tmpfiles.conf
%cargo_bundle_crates -n %{workspace_name} -t 0
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
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

%package -n %{_cross_os}sundog
Summary: Updates settings dynamically based on user-specified generators
Requires: %{_cross_os}apiserver = %{version}-%{release}
Requires: %{_cross_os}pluto = %{version}-%{release}
%description -n %{_cross_os}sundog
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

%package -n %{_cross_os}storewolf
Summary: Data store creator
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}storewolf
%{summary}.

%package -n %{_cross_os}migration
Summary: Tools to migrate version formats
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}migration
%{summary}.

%prep
%setup -qn %{workspace_name}
%cargo_prep

%build
%cargo_build --all

%check
%cargo_test --all

%install
mkdir -p %{buildroot}/%{systemd_systemdir}
install -m 0644 -t %{buildroot}/%{systemd_systemdir} %{SOURCE1}
install -m 0644 -t %{buildroot}/%{systemd_systemdir} %{SOURCE2}
install -m 0644 -t %{buildroot}/%{systemd_systemdir} %{SOURCE3}
install -m 0644 -t %{buildroot}/%{systemd_systemdir} %{SOURCE4}

%cargo_install -p apiserver
%cargo_install -p apiclient
%cargo_install -p moondog
%cargo_install -p sundog
%cargo_install -p pluto
%cargo_install -p thar-be-settings
%cargo_install -p storewolf
%cargo_install -p migration/migrator

install -d %{buildroot}%{migrationdir}
echo %{_cross_bindir}/migrator > migration-binaries

for version in %migration_versions ; do
  for path in migration/migrations/${version}/* ; do
    [ -e "${path}" ] || continue
    name="${path##*/}"
    %cargo_install -p migration/migrations/${version}/${name} -d %{buildroot}%{migrationdir}
    mv %{buildroot}%{migrationdir}/bin/${name} %{buildroot}%{migrationdir}/migrate_${version}_${name}
    echo %{migrationdir}/migrate_${version}_${name} >> migration-binaries
  done
done
%{__rm} -rf %{buildroot}%{migrationdir}/bin

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:5} %{buildroot}%{_cross_tmpfilesdir}/migration.conf

%files -n %{_cross_os}apiserver
%{_cross_bindir}/apiserver
%{systemd_systemdir}/apiserver.service

%files -n %{_cross_os}apiclient
%{_cross_bindir}/apiclient

%files -n %{_cross_os}moondog
%{_cross_bindir}/moondog
%{systemd_systemdir}/moondog.service

%files -n %{_cross_os}sundog
%{_cross_bindir}/sundog
%{systemd_systemdir}/sundog.service

%files -n %{_cross_os}pluto
%{_cross_bindir}/pluto

%files -n %{_cross_os}thar-be-settings
%{_cross_bindir}/thar-be-settings

%files -n %{_cross_os}storewolf
%{_cross_bindir}/storewolf
%{systemd_systemdir}/storewolf.service

%files -n %{_cross_os}migration -f migration-binaries
%dir %{migrationdir}
%{_cross_tmpfilesdir}/migration.conf

%changelog
