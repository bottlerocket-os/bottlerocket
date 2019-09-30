%global goproject github.com/kubernetes
%global gorepo kubernetes
%global goimport %{goproject}/%{gorepo}

%global gover 1.14.6
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Container cluster management
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: kubelet.service
Source2: kubelet-env
Source3: kubelet-config
Source4: kubelet-kubeconfig
Source5: kubernetes-ca-crt
Patch1: 0001-always-set-relevant-variables-for-cross-compiling.patch
Patch2: 0002-do-not-omit-debug-info.patch

BuildRequires: git
BuildRequires: rsync
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang

%description
%{summary}.

%package -n %{_cross_os}kubelet
Summary: Container cluster node agent
License: ASL 2.0
Requires: %{_cross_os}conntrack-tools
Requires: %{_cross_os}containerd
Requires: %{_cross_os}glibc
Requires: %{_cross_os}findutils

%description -n %{_cross_os}kubelet
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export KUBE_BUILD_PLATFORMS="linux/%{_cross_go_arch}"
make WHAT="cmd/kubelet"

%install
output="./_output/local/bin/linux/%{_cross_go_arch}"
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${output}/kubelet %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/kubelet.service

mkdir -p %{buildroot}%{_cross_templatedir}
install -m 0644 %{S:2} %{buildroot}%{_cross_templatedir}/kubelet-env
install -m 0644 %{S:3} %{buildroot}%{_cross_templatedir}/kubelet-config
install -m 0644 %{S:4} %{buildroot}%{_cross_templatedir}/kubelet-kubeconfig
install -m 0644 %{S:5} %{buildroot}%{_cross_templatedir}/kubernetes-ca-crt

%files -n %{_cross_os}kubelet
%{_cross_bindir}/kubelet
%{_cross_unitdir}/kubelet.service
%dir %{_cross_templatedir}
%{_cross_templatedir}/kubelet-env
%{_cross_templatedir}/kubelet-config
%{_cross_templatedir}/kubelet-kubeconfig
%{_cross_templatedir}/kubernetes-ca-crt

%changelog
