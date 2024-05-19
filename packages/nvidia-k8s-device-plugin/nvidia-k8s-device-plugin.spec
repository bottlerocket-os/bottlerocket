%global goproject github.com/NVIDIA
%global gorepo k8s-device-plugin
%global goimport %{goproject}/%{gorepo}

%global gover 0.14.4
%global rpmver %{gover}

Name: %{_cross_os}nvidia-k8s-device-plugin
Version: %{rpmver}
Release: 1%{?dist}
Summary: Kubernetes device plugin for NVIDIA GPUs
License: Apache-2.0
URL: https://github.com/NVIDIA/k8s-device-plugin
Source0: https://%{goimport}/archive/v%{gover}/v%{gover}.tar.gz#/k8s-device-plugin-%{gover}.tar.gz
Source1: nvidia-k8s-device-plugin.service
Source2: nvidia-k8s-device-plugin-conf
Source3: nvidia-k8s-device-plugin-tmpfiles.conf

BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: Kubernetes device plugin for NVIDIA GPUs binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Kubernetes device plugin for NVIDIA GPUs binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%autosetup -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
# We don't set `-Wl,-z,now`, because the binary uses lazy loading
# to load the NVIDIA libraries in the host
export CGO_LDFLAGS="-Wl,-z,relro -Wl,--export-dynamic"
export GOLDFLAGS="-compressdwarf=false -linkmode=external -extldflags '${CGO_LDFLAGS}'"

go build -ldflags="${GOLDFLAGS}" -o nvidia-device-plugin ./cmd/nvidia-device-plugin/
gofips build -ldflags="${GOLDFLAGS}" -o fips/nvidia-device-plugin ./cmd/nvidia-device-plugin/

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 nvidia-device-plugin %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 fips/nvidia-device-plugin %{buildroot}%{_cross_fips_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}
install -d %{buildroot}%{_cross_tmpfilesdir}
install -D -p -m 0644 %{S:2} %{buildroot}%{_cross_templatedir}/nvidia-k8s-device-plugin-conf
install -m 0644 %{S:3} %{buildroot}%{_cross_tmpfilesdir}/nvidia-k8s-device-plugin-tmpfiles.conf
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/nvidia-k8s-device-plugin

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_unitdir}/nvidia-k8s-device-plugin.service
%dir %{_cross_factorydir}%{_cross_sysconfdir}/nvidia-k8s-device-plugin
%{_cross_templatedir}/nvidia-k8s-device-plugin-conf
%{_cross_tmpfilesdir}/nvidia-k8s-device-plugin-tmpfiles.conf

%files bin
%{_cross_bindir}/nvidia-device-plugin

%files fips-bin
%{_cross_fips_bindir}/nvidia-device-plugin
