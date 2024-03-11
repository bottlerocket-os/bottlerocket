%global goproject github.com/containernetworking
%global gorepo plugins
%global goimport %{goproject}/%{gorepo}

%global gover 1.4.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}cni-%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Plugins for container networking
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: cni-plugins-tmpfiles.conf
BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}iptables
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: Plugins for container networking binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Plugins for container networking binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
for d in $(find plugins -mindepth 2 -maxdepth 2 -type d ! -name windows) ; do
  go build -ldflags="${GOLDFLAGS}" -o "bin/${d##*/}" %{goimport}/${d}
  gofips build -ldflags="${GOLDFLAGS}" -o "fips/bin/${d##*/}" %{goimport}/${d}
done

%install
install -d %{buildroot}%{_cross_libexecdir}/cni/bin
install -p -m 0755 bin/* %{buildroot}%{_cross_libexecdir}/cni/bin

install -d %{buildroot}%{_cross_fips_libexecdir}/cni/bin
install -p -m 0755 fips/bin/* %{buildroot}%{_cross_fips_libexecdir}/cni/bin

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/cni-plugins.conf

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_tmpfilesdir}/cni-plugins.conf

%files bin
%{_cross_libexecdir}/cni/bin/loopback
%{_cross_libexecdir}/cni/bin/bandwidth
%{_cross_libexecdir}/cni/bin/bridge
%{_cross_libexecdir}/cni/bin/dhcp
%{_cross_libexecdir}/cni/bin/dummy
%{_cross_libexecdir}/cni/bin/firewall
%{_cross_libexecdir}/cni/bin/host-device
%{_cross_libexecdir}/cni/bin/host-local
%{_cross_libexecdir}/cni/bin/ipvlan
%{_cross_libexecdir}/cni/bin/macvlan
%{_cross_libexecdir}/cni/bin/portmap
%{_cross_libexecdir}/cni/bin/ptp
%{_cross_libexecdir}/cni/bin/sbr
%{_cross_libexecdir}/cni/bin/static
%{_cross_libexecdir}/cni/bin/tap
%{_cross_libexecdir}/cni/bin/tuning
%{_cross_libexecdir}/cni/bin/vlan
%{_cross_libexecdir}/cni/bin/vrf

%files fips-bin
%{_cross_fips_libexecdir}/cni/bin/loopback
%{_cross_fips_libexecdir}/cni/bin/bandwidth
%{_cross_fips_libexecdir}/cni/bin/bridge
%{_cross_fips_libexecdir}/cni/bin/dhcp
%{_cross_fips_libexecdir}/cni/bin/dummy
%{_cross_fips_libexecdir}/cni/bin/firewall
%{_cross_fips_libexecdir}/cni/bin/host-device
%{_cross_fips_libexecdir}/cni/bin/host-local
%{_cross_fips_libexecdir}/cni/bin/ipvlan
%{_cross_fips_libexecdir}/cni/bin/macvlan
%{_cross_fips_libexecdir}/cni/bin/portmap
%{_cross_fips_libexecdir}/cni/bin/ptp
%{_cross_fips_libexecdir}/cni/bin/sbr
%{_cross_fips_libexecdir}/cni/bin/static
%{_cross_fips_libexecdir}/cni/bin/tap
%{_cross_fips_libexecdir}/cni/bin/tuning
%{_cross_fips_libexecdir}/cni/bin/vlan
%{_cross_fips_libexecdir}/cni/bin/vrf

%changelog
