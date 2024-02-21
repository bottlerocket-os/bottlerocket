%global goproject github.com/NVIDIA
%global gorepo nvidia-container-toolkit
%global goimport %{goproject}/%{gorepo}

%global gover 1.13.5
%global rpmver %{gover}

Name: %{_cross_os}nvidia-container-toolkit
Version: %{rpmver}
Release: 1%{?dist}
Summary: Tool to build and run GPU accelerated containers
License: Apache-2.0
URL: https://%{goimport}

Source0: https://%{goimport}/archive/v%{gover}/nvidia-container-toolkit-%{gover}.tar.gz
Source1: nvidia-container-toolkit-config-k8s.toml
Source2: nvidia-container-toolkit-config-ecs.toml
Source3: nvidia-oci-hooks-json
Source4: nvidia-gpu-devices.rules
Source5: nvidia-container-toolkit-tmpfiles-ecs.conf
Source6: nvidia-container-toolkit-tmpfiles-k8s.conf

BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}libnvidia-container
Requires: %{_cross_os}shimpei

%description
%{summary}.

%package ecs
Summary: Files specific for the ECS variants
Requires: %{name}
Conflicts: %{name}-k8s

%description ecs
%{summary}.

%package k8s
Summary: Files specific for the Kubernetes variants
Requires: %{name}
Conflicts: %{name}-ecs

%description k8s
%{summary}.

%prep
%autosetup -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o nvidia-container-runtime-hook ./cmd/nvidia-container-runtime-hook
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o nvidia-ctk ./cmd/nvidia-ctk

%install
install -d %{buildroot}%{_cross_bindir}
install -d %{buildroot}%{_cross_tmpfilesdir}
install -d %{buildroot}%{_cross_templatedir}
install -d %{buildroot}%{_cross_udevrulesdir}
install -d %{buildroot}%{_cross_datadir}/nvidia-container-toolkit
install -d %{buildroot}%{_cross_factorydir}/nvidia-container-runtime
install -p -m 0755 nvidia-container-runtime-hook %{buildroot}%{_cross_bindir}/
install -p -m 0755 nvidia-ctk %{buildroot}%{_cross_bindir}/
install -m 0644 %{S:1} %{S:2} %{buildroot}%{_cross_factorydir}/nvidia-container-runtime/
install -m 0644 %{S:3} %{buildroot}%{_cross_templatedir}/nvidia-oci-hooks-json
install -p -m 0644 %{S:4} %{buildroot}%{_cross_udevrulesdir}/90-nvidia-gpu-devices.rules
install -m 0644 %{S:5} %{buildroot}%{_cross_tmpfilesdir}/nvidia-container-toolkit-ecs.conf
install -m 0644 %{S:6} %{buildroot}%{_cross_tmpfilesdir}/nvidia-container-toolkit-k8s.conf
ln -s shimpei %{buildroot}%{_cross_bindir}/nvidia-oci

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_bindir}/nvidia-container-runtime-hook
%{_cross_bindir}/nvidia-ctk
%{_cross_bindir}/nvidia-oci
%{_cross_templatedir}/nvidia-oci-hooks-json
%{_cross_udevrulesdir}/90-nvidia-gpu-devices.rules

%files ecs
%{_cross_factorydir}/nvidia-container-runtime/nvidia-container-toolkit-config-ecs.toml
%{_cross_tmpfilesdir}/nvidia-container-toolkit-ecs.conf

%files k8s
%{_cross_factorydir}/nvidia-container-runtime/nvidia-container-toolkit-config-k8s.toml
%{_cross_tmpfilesdir}/nvidia-container-toolkit-k8s.conf
