%global goproject github.com/kubernetes
%global gorepo kubernetes
%global goimport %{goproject}/%{gorepo}

%global gover 1.15.12
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Container cluster management
# base Apache-2.0, third_party Apache-2.0 AND BSD-3-Clause
License: Apache-2.0 AND BSD-3-Clause
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: kubelet.service
Source2: kubelet-env
Source3: kubelet-config
Source4: kubelet-kubeconfig
Source5: kubernetes-ca-crt
Source6: kubernetes-tmpfiles.conf
Source1000: clarify.toml
Patch1: 0001-always-set-relevant-variables-for-cross-compiling.patch

# Fix builds in $GOPATH when using Go 1.13 - drop when we catch up in v1.17.0
# https://github.com/kubernetes/kubernetes/commit/8618c09
Patch2: 0002-opt-out-of-module-mode-for-builds.patch

Patch3: 0003-kubelet-block-non-forwarded-packets.patch
Patch4: 0004-include-etc-hosts-in-eviction-calc.patch

BuildRequires: git
BuildRequires: rsync
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package -n %{_cross_os}kubelet-1.15
Summary: Container cluster node agent
Requires: %{_cross_os}conntrack-tools
Requires: %{_cross_os}containerd
Requires: %{_cross_os}findutils

%description -n %{_cross_os}kubelet-1.15
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

# third_party licenses
# multiarch/qemu-user-static ignored, we're not using it
cp third_party/forked/gonum/graph/LICENSE LICENSE.gonum.graph
cp third_party/forked/shell2junit/LICENSE LICENSE.shell2junit
cp third_party/forked/golang/LICENSE LICENSE.golang
cp third_party/forked/golang/PATENTS PATENTS.golang
cp third_party/go-srcimporter/LICENSE LICENSE.go-srcimporter
cp third_party/intemp/LICENSE LICENSE.intemp

%build
%cross_go_configure %{goimport}
export KUBE_BUILD_PLATFORMS="linux/%{_cross_go_arch}"
export GOLDFLAGS="-buildmode=pie -linkmode=external"
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

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:6} %{buildroot}%{_cross_tmpfilesdir}/kubernetes.conf

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files -n %{_cross_os}kubelet-1.15
%license LICENSE LICENSE.gonum.graph LICENSE.shell2junit LICENSE.golang PATENTS.golang LICENSE.go-srcimporter LICENSE.intemp
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/kubelet
%{_cross_unitdir}/kubelet.service
%dir %{_cross_templatedir}
%{_cross_templatedir}/kubelet-env
%{_cross_templatedir}/kubelet-config
%{_cross_templatedir}/kubelet-kubeconfig
%{_cross_templatedir}/kubernetes-ca-crt
%{_cross_tmpfilesdir}/kubernetes.conf

%changelog
