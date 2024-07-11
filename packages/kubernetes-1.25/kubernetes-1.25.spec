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

%global gover 1.25.16
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
Source0: https://github.com/kubernetes/kubernetes/archive/v%{gover}/kubernetes-%{gover}.tar.gz
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
Source15: logdog.kubelet.conf

# ExecStartPre drop-ins
Source20: prestart-pull-pause-ctr.conf
Source21: dockershim-symlink.conf
Source22: make-kubelet-dirs.conf

Source1000: clarify.toml

Patch0001: 0001-EKS-PATCH-Pass-region-to-sts-client.patch
Patch0002: 0002-EKS-PATCH-admission-webhook-exclusion-from-file.patch
Patch0003: 0003-EKS-PATCH-Use-GNU-date.patch
Patch0004: 0004-EKS-PATCH-aws_credentials-update-ecr-url-validation-.patch
Patch0005: 0005-EKS-PATCH-AWS-Include-IPv6-addresses-in-NodeAddresse.patch
Patch0007: 0007-EKS-PATCH-kubelet-Keep-trying-fast-status-update-at-.patch
Patch0008: 0008-EKS-PATCH-add-Authentication-tracking-request-error-.patch
Patch0009: 0009-EKS-PATCH-Added-serialization-from-etcd-error-metric.patch
Patch0010: 0010-EKS-PATCH-Handle-eventually-consistent-EC2-PrivateDn.patch
Patch0011: 0011-EKS-PATCH-Fix-controllers-scheduling-and-logging-bug.patch
Patch0012: 0012-EKS-PATCH-Fix-CVE-for-kube-proxy-v1.25.11.patch
Patch0013: 0013-EKS-PATCH-Support-tracking-executing-requests.patch
Patch0014: 0014-EKS-PATCH-Fix-CVE-for-kube-proxy-v1.25.14.patch
Patch0015: 0015-EKS-PATCH-bump-google.golang.org-grpc-to-v1.56.3.patch
Patch0016: 0016-EKS-PATCH-kms-fix-go-routine-leak-in-gRPC-connection.patch
Patch0017: 0017-EKS-PATCH-Update-log-verbosity-for-node-health-and-t.patch
Patch0018: 0018-EKS-PATCH-CVE-2024-24786-Bump-github.com-golang-prot.patch
Patch0019: 0019-EKS-PATCH-GO-UPDATE-handle-GOTOOLCHAIN-in-kube-golan.patch
Patch0020: 0020-EKS-PATCH-GO-UPDATE-Bump-images-dependencies-and-ver.patch
Patch0021: 0021-EKS-PATCH-Bumps-dependencies-for-fixing-CVE-2023-452.patch
Patch0022: 0022-EKS-PATCH-Update-aws-sdk-go-to-include-new-regions.patch

BuildRequires: git
BuildRequires: rsync
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package -n %{_cross_os}kubelet-1.25
Summary: Container cluster node agent
Requires: %{_cross_os}conntrack-tools
Requires: %{_cross_os}containerd
Requires: %{_cross_os}findutils
Requires: %{_cross_os}ecr-credential-provider
Requires: %{_cross_os}aws-signing-helper
Requires: %{_cross_os}static-pods
Requires: %{_cross_os}kubelet-1.25(binaries)

%description -n %{_cross_os}kubelet-1.25
%{summary}.

%package -n %{_cross_os}kubelet-1.25-bin
Summary: Container cluster node agent binaries
Provides: %{_cross_os}kubelet-1.25(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{_cross_os}kubelet-1.25)
Conflicts: (%{_cross_os}image-feature(fips) or %{_cross_os}kubelet-1.25-fips-bin)

%description -n %{_cross_os}kubelet-1.25-bin
%{summary}.

%package -n %{_cross_os}kubelet-1.25-fips-bin
Summary: Container cluster node agent binaries, FIPS edition
Provides: %{_cross_os}kubelet-1.25(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{_cross_os}kubelet-1.25)
Conflicts: (%{_cross_os}image-feature(no-fips) or %{_cross_os}kubelet-1.25-bin)

%description -n %{_cross_os}kubelet-1.25-fips-bin
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1

# third_party licenses
# multiarch/qemu-user-static ignored, we're not using it
cp third_party/forked/gonum/graph/LICENSE LICENSE.gonum.graph
cp third_party/forked/shell2junit/LICENSE LICENSE.shell2junit
cp third_party/forked/golang/LICENSE LICENSE.golang
cp third_party/forked/golang/PATENTS PATENTS.golang

%build
export FORCE_HOST_GO=1
# Build codegen programs with the host toolchain.
make generated_files

# Build kubelet with the target toolchain.
%set_cross_go_flags
unset CC
export KUBE_BUILD_PLATFORMS="linux/%{_cross_go_arch}"
export %{kube_cc}
export GOFLAGS="${GOFLAGS} -tags=dockerless"
export GOLDFLAGS="${GOLDFLAGS}"
make WHAT="cmd/kubelet"

export KUBE_OUTPUT_SUBPATH="_fips_output/local"
export GOEXPERIMENT="boringcrypto"
make WHAT="cmd/kubelet"

%install
output="./_output/local/bin/linux/%{_cross_go_arch}"
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${output}/kubelet %{buildroot}%{_cross_bindir}

fips_output="./_fips_output/local/bin/linux/%{_cross_go_arch}"
install -d %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 ${fips_output}/kubelet %{buildroot}%{_cross_fips_bindir}

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

install -d %{buildroot}%{_cross_datadir}/logdog.d
install -p -m 0644 %{S:15} %{buildroot}%{_cross_datadir}/logdog.d

%files -n %{_cross_os}kubelet-1.25
%license LICENSE LICENSE.gonum.graph LICENSE.shell2junit LICENSE.golang PATENTS.golang
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
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
%{_cross_datadir}/logdog.d/logdog.kubelet.conf

%files -n %{_cross_os}kubelet-1.25-bin
%{_cross_bindir}/kubelet

%files -n %{_cross_os}kubelet-1.25-fips-bin
%{_cross_fips_bindir}/kubelet

%changelog
