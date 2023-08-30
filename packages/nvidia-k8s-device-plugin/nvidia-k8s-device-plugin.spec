%global goproject github.com/NVIDIA
%global gorepo k8s-device-plugin
%global goimport %{goproject}/%{gorepo}

%global gover 0.14.1
%global rpmver %{gover}

Name: %{_cross_os}nvidia-k8s-device-plugin
Version: %{rpmver}
Release: 1%{?dist}
Summary: Kubernetes device plugin for NVIDIA GPUs
License: Apache-2.0
URL: https://github.com/NVIDIA/k8s-device-plugin
Source0: https://%{goimport}/archive/v%{gover}/v%{gover}.tar.gz#/k8s-device-plugin-%{gover}.tar.gz
Source1: nvidia-k8s-device-plugin.service

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
# We don't set `-Wl,-z,now`, because the binary uses lazy loading
# to load the NVIDIA libraries in the host
export CGO_LDFLAGS="-Wl,-z,relro"
go build -ldflags="${GOLDFLAGS}" -o nvidia-device-plugin ./cmd/nvidia-device-plugin/

%install
install -d %{buildroot}%{_cross_bindir}
install -d %{buildroot}%{_cross_unitdir}
install -p -m 0755 nvidia-device-plugin %{buildroot}%{_cross_bindir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_unitdir}/nvidia-k8s-device-plugin.service
%{_cross_bindir}/nvidia-device-plugin
