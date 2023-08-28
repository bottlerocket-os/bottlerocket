# After this upstream change, the linker flags `-s -w` are always added unless
# DBG=1 is set in the environment, which would set compiler flags to disable
# optimizations and inlining:
#  https://github.com/kubernetes/kubernetes/pull/108371
#
# For now, work around this by indicating that no debug package is expected.
%global debug_package %{nil}

%global goproject github.com/kubernetes
%global gorepo kubernetes
%global goimport %{goproject}/%{gorepo}

%global gover 1.26.7
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

# The kubernetes build process expects the cross-compiler to be specified via `KUBE_*_CC`
# Here we generate that variable to use bottlerocket-specific compile aliases
# Examples of the generated variable:
# KUBE_LINUX_AMD64_CC=x86_64-bottlerocket-linux-gnu-gcc
# KUBE_LINUX_ARM64_CC=aarch64-bottlerocket-linux-gnu-gcc
%global kube_cc %{shrink: \
  %{lua: print(string.upper( \
     rpm.expand("KUBE_%{_cross_go_os}_%{_cross_go_arch}_CC=")) .. \
     rpm.expand("%{_cross_target}-gcc")) }}

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Container cluster management
# base Apache-2.0, third_party Apache-2.0 AND BSD-3-Clause
License: Apache-2.0 AND BSD-3-Clause
URL: https://%{goimport}
Source0: https://distro.eks.amazonaws.com/kubernetes-1-26/releases/15/artifacts/kubernetes/v%{gover}/kubernetes-src.tar.gz
Source1: kubelet.service
Source2: kubelet-env
Source3: kubelet-config
Source4: kubelet-kubeconfig
Source5: kubernetes-ca-crt
Source6: kubelet-exec-start-conf
Source7: kubelet-bootstrap-kubeconfig
Source8: kubernetes-tmpfiles.conf
Source9: kubelet-sysctl.conf
Source10: prepare-var-lib-kubelet.service
Source11: kubelet-server-crt
Source12: kubelet-server-key
Source13: etc-kubernetes-pki-private.mount
Source14: credential-provider-config-yaml

# ExecStartPre drop-ins
Source20: prestart-pull-pause-ctr.conf
Source21: dockershim-symlink.conf
Source22: make-kubelet-dirs.conf

Source1000: clarify.toml

BuildRequires: git
BuildRequires: rsync
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package -n %{_cross_os}kubelet-1.26
Summary: Container cluster node agent
Requires: %{_cross_os}conntrack-tools
Requires: %{_cross_os}containerd
Requires: %{_cross_os}findutils
Requires: %{_cross_os}ecr-credential-provider
Requires: %{_cross_os}aws-signing-helper

%description -n %{_cross_os}kubelet-1.26
%{summary}.

%prep
%autosetup -Sgit -c -n %{gorepo}-%{gover} -p1

# third_party licenses
# multiarch/qemu-user-static ignored, we're not using it
cp third_party/forked/gonum/graph/LICENSE LICENSE.gonum.graph
cp third_party/forked/shell2junit/LICENSE LICENSE.shell2junit
cp third_party/forked/golang/LICENSE LICENSE.golang
cp third_party/forked/golang/PATENTS PATENTS.golang

%build
# Build codegen programs with the host toolchain.
make hack/update-codegen.sh

# Build kubelet with the target toolchain.
export KUBE_BUILD_PLATFORMS="linux/%{_cross_go_arch}"
export %{kube_cc}
export GOFLAGS='-tags=dockerless'
export GOLDFLAGS="-buildmode=pie -linkmode=external -compressdwarf=false"
make WHAT="cmd/kubelet"

%install
output="./_output/local/bin/linux/%{_cross_go_arch}"
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${output}/kubelet %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{S:10} %{S:13} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_unitdir}/kubelet.service.d
install -p -m 0644 %{S:20} %{S:21} %{S:22} %{buildroot}%{_cross_unitdir}/kubelet.service.d

mkdir -p %{buildroot}%{_cross_templatedir}
install -m 0644 %{S:2} %{buildroot}%{_cross_templatedir}/kubelet-env
install -m 0644 %{S:3} %{buildroot}%{_cross_templatedir}/kubelet-config
install -m 0644 %{S:4} %{buildroot}%{_cross_templatedir}/kubelet-kubeconfig
install -m 0644 %{S:5} %{buildroot}%{_cross_templatedir}/kubernetes-ca-crt
install -m 0644 %{S:6} %{buildroot}%{_cross_templatedir}/kubelet-exec-start-conf
install -m 0644 %{S:7} %{buildroot}%{_cross_templatedir}/kubelet-bootstrap-kubeconfig
install -m 0644 %{S:11} %{buildroot}%{_cross_templatedir}/kubelet-server-crt
install -m 0644 %{S:12} %{buildroot}%{_cross_templatedir}/kubelet-server-key
install -m 0644 %{S:14} %{buildroot}%{_cross_templatedir}/credential-provider-config-yaml

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:8} %{buildroot}%{_cross_tmpfilesdir}/kubernetes.conf

install -d %{buildroot}%{_cross_sysctldir}
install -p -m 0644 %{S:9} %{buildroot}%{_cross_sysctldir}/90-kubelet.conf

install -d %{buildroot}%{_cross_libexecdir}/kubernetes
ln -rs \
  %{buildroot}%{_sharedstatedir}/kubelet/plugins \
  %{buildroot}%{_cross_libexecdir}/kubernetes/kubelet-plugins

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files -n %{_cross_os}kubelet-1.26
%license LICENSE LICENSE.gonum.graph LICENSE.shell2junit LICENSE.golang PATENTS.golang
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/kubelet
%{_cross_unitdir}/kubelet.service
%{_cross_unitdir}/prepare-var-lib-kubelet.service
%{_cross_unitdir}/etc-kubernetes-pki-private.mount
%dir %{_cross_unitdir}/kubelet.service.d
%{_cross_unitdir}/kubelet.service.d/prestart-pull-pause-ctr.conf
%{_cross_unitdir}/kubelet.service.d/make-kubelet-dirs.conf
%{_cross_unitdir}/kubelet.service.d/dockershim-symlink.conf
%dir %{_cross_templatedir}
%{_cross_templatedir}/kubelet-env
%{_cross_templatedir}/kubelet-config
%{_cross_templatedir}/kubelet-kubeconfig
%{_cross_templatedir}/kubelet-bootstrap-kubeconfig
%{_cross_templatedir}/kubelet-exec-start-conf
%{_cross_templatedir}/kubernetes-ca-crt
%{_cross_templatedir}/kubelet-server-crt
%{_cross_templatedir}/kubelet-server-key
%{_cross_templatedir}/credential-provider-config-yaml
%{_cross_tmpfilesdir}/kubernetes.conf
%{_cross_sysctldir}/90-kubelet.conf
%dir %{_cross_libexecdir}/kubernetes
%{_cross_libexecdir}/kubernetes/kubelet-plugins

%changelog
