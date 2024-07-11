%global goproject github.com/kubernetes
%global gorepo kubernetes
%global goimport %{goproject}/%{gorepo}

%global gover 1.23.17
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

# Additional patches on top of last 1.23 point release

Patch0001: 0001-EKS-PATCH-Pass-region-to-sts-client.patch
Patch0002: 0002-EKS-PATCH-admission-webhook-exclusion-from-file.patch
Patch0003: 0003-EKS-PATCH-Use-GNU-date.patch
Patch0004: 0004-EKS-PATCH-aws_credentials-update-ecr-url-validation-.patch
Patch0005: 0005-EKS-PATCH-extend-sa-token-if-audience-is-apiserver-1.patch
Patch0006: 0006-EKS-PATCH-Parse-ipv6-address-before-comparison-10773.patch
Patch0007: 0007-EKS-PATCH-AWS-Include-IPv6-addresses-in-NodeAddresse.patch
Patch0009: 0009-EKS-PATCH-Make-kubelet-set-alpha.kubernetes.io-provi.patch
Patch0010: 0010-EKS-PATCH-Skip-mount-point-checks-when-possible-duri.patch
Patch0017: 0017-EKS-PATCH-Update-aws-sdk-go-to-include-new-regions.patch
Patch0018: 0018-EKS-PATCH-Prevent-watch-requests-on-resources-with-a.patch
Patch0019: 0019-EKS-PATCH-add-Authentication-tracking-request-error-.patch
Patch0020: 0020-EKS-PATCH-Added-serialization-from-etcd-error-metric.patch
Patch0021: 0021-EKS-PATCH-Bump-1.23-runc-version-to-1.1.6.patch
Patch0022: 0022-EKS-PATCH-Handle-eventually-consistent-EC2-PrivateDn.patch
Patch0023: 0023-EKS-PATCH-Remove-check-for-CSI-driver-running-on-nod.patch
Patch0024: 0024-EKS-PATCH-Update-managedFields-time-when-value-is-mo.patch
Patch0025: 0025-EKS-PATCH-Return-error-for-localhost-seccomp-type-wi.patch
Patch0026: 0026-EKS-PATCH-Cherry-pick-119832-Fix-the-problem-Pod-ter.patch
Patch0027: 0027-EKS-PATCH-Prevent-rapid-reset-http2-DOS-on-API-serve.patch
Patch0028: 0028-EKS-PATCH-bump-golang.org-x-net-to-v0.17.patch
Patch0029: 0029-EKS-PATCH-Add-ephemeralcontainer-to-imagepolicy-secu.patch
Patch0030: 0030-EKS-PATCH-go-Bump-images-dependencies-and-versions-t.patch
Patch0031: 0031-EKS-PATCH-Fix-CVE-2023-5528.patch
Patch0032: 0032-EKS-PATCH-bump-google.golang.org-grpc-to-v1.56.3.patch
Patch0033: 0033-EKS-PATCH-Fix-multiple-CVEs-in-windows-Kubelet.patch
Patch0034: 0034-EKS-PATCH-Support-tracking-executing-requests.patch
Patch0035: 0035-EKS-PATCH-Fix-CVE-for-kube-proxy-v1.23.17.patch
Patch0036: 0036-EKS-PATCH-kubelet-expose-OOM-metrics.patch
Patch0037: 0037-EKS-PATCH-GO-UPDATE-Explicitly-call-rand.Seed-method.patch
Patch0038: 0038-EKS-PATCH-GO-UPDATE-Fixes-the-issue-114145.patch
Patch0039: 0039-EKS-PATCH-GO-UPDATE-bump-honnef.co-go-tools-to-suppo.patch
Patch0040: 0040-EKS-PATCH-GO-UPDATE-Avoid-typechecking-stdlib.patch
Patch0041: 0041-EKS-PATCH-GO-UPDATE-hack-tools-Bump-golangci-lint-ve.patch
Patch0042: 0042-EKS-PATCH-GO-UPDATE-Fix-linter-warnings.patch
Patch0043: 0043-EKS-PATCH-GO-UPDATE-Bump-version-of-vmware-govmomi.patch
Patch0044: 0044-EKS-PATCH-GO-UPDATE-vsphere-Adapt-to-govmomi-version.patch
Patch0045: 0045-EKS-PATCH-GO-UPDATE-vclib-Modify-x509.UnknownAuthori.patch
Patch0046: 0046-EKS-PATCH-GO-UPDATE-Switch-to-assert.ErrorEquals-fro.patch
Patch0047: 0047-EKS-PATCH-GO-UPDATE-.-bump-govmomi-to-v0.30.6.patch
Patch0048: 0048-EKS-PATCH-GO-UPDATE-vsphere-adapt-to-govmomi-bump.patch
Patch0049: 0049-EKS-PATCH-GO-UPDATE-release-1.23-releng-go-Update-im.patch
Patch0050: 0050-EKS-PATCH-GO-UPDATE-Defer-builds-to-test-cmd-and-tes.patch
Patch0051: 0051-EKS-PATCH-GO-UPDATE-Add-gimme.patch
Patch0052: 0052-EKS-PATCH-GO-UPDATE-Invoke-gimme-from-kube-golang-ve.patch
Patch0053: 0053-EKS-PATCH-CVE-2024-24786-Bump-github.com-golang-prot.patch
Patch0054: 0054-EKS-PATCH-GO-UPDATE-kubeadm-remove-function-pointer-.patch
Patch0055: 0055-EKS-PATCH-GO-UPDATE-update-webhook-test-to-go-1.21.patch
Patch0056: 0056-EKS-PATCH-GO-UPDATE-Fix-the-git-repo-test-error-caus.patch
Patch0057: 0057-EKS-PATCH-GO-UPDATE-prep-for-go1.21-use-e-in-go-list.patch
Patch0058: 0058-EKS-PATCH-GO-UPDATE-Merge-pull-request-122077-from-B.patch
Patch0059: 0059-EKS-PATCH-GO-UPDATE-update-to-golangci-lint-v1.54.1-.patch
Patch0060: 0060-EKS-PATCH-GO-UPDATE-feat-use-clock-instead.patch
Patch0061: 0061-EKS-PATCH-GO-UPDATE-go-Bump-images-dependencies-and-.patch
Patch0062: 0062-EKS-PATCH-CVE-2023-45288-Bump-dependecy-for-1.23.patch
Patch0063: 0063-EKS-PATCH-Update-aws-sdk-go-to-include-NCL-region.patch

BuildRequires: git
BuildRequires: rsync
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package -n %{_cross_os}kubelet-1.23
Summary: Container cluster node agent
Requires: %{_cross_os}conntrack-tools
Requires: %{_cross_os}containerd
Requires: %{_cross_os}findutils
Requires: %{_cross_os}ecr-credential-provider
Requires: %{_cross_os}aws-signing-helper
Requires: %{_cross_os}static-pods
Requires: %{_cross_os}kubelet-1.23(binaries)

%description -n %{_cross_os}kubelet-1.23
%{summary}.

%package -n %{_cross_os}kubelet-1.23-bin
Summary: Container cluster node agent binaries
Provides: %{_cross_os}kubelet-1.23(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{_cross_os}kubelet-1.23)
Conflicts: (%{_cross_os}image-feature(fips) or %{_cross_os}kubelet-1.23-fips-bin)

%description -n %{_cross_os}kubelet-1.23-bin
%{summary}.

%package -n %{_cross_os}kubelet-1.23-fips-bin
Summary: Container cluster node agent binaries, FIPS edition
Provides: %{_cross_os}kubelet-1.23(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{_cross_os}kubelet-1.23)
Conflicts: (%{_cross_os}image-feature(no-fips) or %{_cross_os}kubelet-1.23-bin)

%description -n %{_cross_os}kubelet-1.23-fips-bin
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

%files -n %{_cross_os}kubelet-1.23
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

%files -n %{_cross_os}kubelet-1.23-bin
%{_cross_bindir}/kubelet

%files -n %{_cross_os}kubelet-1.23-fips-bin
%{_cross_fips_bindir}/kubelet

%changelog
