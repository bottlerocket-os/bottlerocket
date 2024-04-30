%global _cross_first_party 1
%global workspace_name host-ctr

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Bottlerocket host container runner
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}containerd
Requires: %{name}(binaries)

Source10: host-containerd.service
Source11: host-containerd-tmpfiles.conf
Source12: host-containerd-config.toml

# Mount for writing host-ctr configuration
Source100: etc-host-containers.mount.in

%description
%{summary}.

%package bin
Summary: Bottlerocket host container runner binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Bottlerocket host container runner binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%setup -T -c
cp -r %{_builddir}/sources/%{workspace_name}/* .

%build
%set_cross_go_flags
go build -ldflags="${GOLDFLAGS}" -o host-ctr ./cmd/host-ctr
gofips build -ldflags="${GOLDFLAGS}" -o fips/host-ctr ./cmd/host-ctr

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 host-ctr %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 fips/host-ctr %{buildroot}%{_cross_fips_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_unitdir}
ETC_HOST_CONTAINERS=$(systemd-escape --path /etc/host-containers)
install -p -m 0644 %{S:100} %{buildroot}%{_cross_unitdir}/${ETC_HOST_CONTAINERS}.mount

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:11} %{buildroot}%{_cross_tmpfilesdir}/host-containerd.conf

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd
install -p -m 0644 %{S:12} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

%cross_scan_attribution go-vendor vendor

%files
%{_cross_attribution_vendor_dir}
%{_cross_unitdir}/host-containerd.service
%{_cross_unitdir}/*.mount
%{_cross_tmpfilesdir}/host-containerd.conf
%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

%files bin
%{_cross_bindir}/host-ctr

%files fips-bin
%{_cross_fips_bindir}/host-ctr

%changelog
