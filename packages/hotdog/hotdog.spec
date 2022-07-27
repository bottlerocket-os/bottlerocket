# Don't generate debug packages because we are compiling without CGO,
# and the `gc` compiler doesn't append the  the ".note.gnu.build-id" section
# https://fedoraproject.org/wiki/PackagingDrafts/Go#Build_ID
%global debug_package %{nil}

%global goproject github.com/bottlerocket
%global gorepo hotdog
%global goimport %{goproject}/%{gorepo}

%global gitrev b85b75576adbbd7e133b54d71ebc11a28acf40db
%global shortrev %(c=%{gitrev}; echo ${c:0:7})

%global gosysrev 8c9f86f7a55f5af45a9080f6ed3ac915382d369c
%global gosysrevshort %(c=%{gosysrev}; echo ${c:0:7})

%global runtimespec 1.0.2

%global goselinux 1.10.1

%global libcap 1.2.63

Name: %{_cross_os}hotdog
Version: 1.0.1
Release: 1%{?dist}
Summary: Tool with OCI hooks to run the Log4j Hot Patch in containers
License: Apache-2.0
URL: https://github.com/awslabs/oci-add-hooks
Source0: https://%{goimport}/archive/%{gorev}/%{gorepo}-%{shortrev}.tar.gz
Source1: https://github.com/opencontainers/runtime-spec/archive/v%{runtimespec}/runtime-spec-%{runtimespec}.tar.gz
Source2: https://github.com/golang/sys/archive/%{gosysrev}/sys-%{gosysrevshort}.tar.gz
Source3: https://github.com/opencontainers/selinux/archive/refs/tags/v%{goselinux}.tar.gz#/go-selinux-v%{goselinux}.tar.gz
Source4: https://git.kernel.org/pub/scm/libs/libcap/libcap.git/snapshot/libcap-cap/v%{libcap}.tar.gz#/libcap-v%{libcap}.tar.gz

BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}log4j2-hotpatch

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gitrev} -p1
%cross_go_setup %{gorepo}-%{gitrev} %{goproject} %{goimport}

# We need to manage these third-party dependencies because the hotdog
# "release" that we use doesn't include the `vendor` directory, unlike our other
# go third party dependencies
mkdir -p GOPATH/src/github.com/opencontainers/runtime-spec
tar -C GOPATH/src/github.com/opencontainers/runtime-spec -xzf %{SOURCE1} --strip 1
cp GOPATH/src/github.com/opencontainers/runtime-spec/LICENSE LICENSE.runtime-spec

mkdir -p GOPATH/src/golang.org/x/sys
tar -C GOPATH/src/golang.org/x/sys -xzf %{SOURCE2} --strip 1
cp GOPATH/src/golang.org/x/sys/LICENSE LICENSE.golang-sys

mkdir -p GOPATH/src/github.com/opencontainers/selinux
tar -C GOPATH/src/github.com/opencontainers/selinux -xzf %{SOURCE3} --strip 1
cp GOPATH/src/github.com/opencontainers/selinux/LICENSE LICENSE.go-selinux

mkdir -p GOPATH/src/kernel.org/pub/linux/libs/security/libcap
tar -C GOPATH/src/kernel.org/pub/linux/libs/security/libcap -xzf %{SOURCE4} --strip 2
cp GOPATH/src/kernel.org/pub/linux/libs/security/libcap/License LICENSE.libcap

%build
%cross_go_configure %{goimport}

# Set CGO_ENABLED=0 to statically link hotdog-hotpath, since it runs inside containers that
# may not have the glibc version used to compile it
# Set `GO111MODULE=off` to force golang to look for the dependencies in the GOPATH
CGO_ENABLED=0 GO111MODULE=off go build -installsuffix cgo -a -ldflags "-s" -o hotdog-hotpatch ./cmd/hotdog-hotpatch

# The oci hooks commands can be compiled as we usually compile golang packages
for cmd in hotdog-cc-hook hotdog-poststart-hook; do
  GO111MODULE=off go build -buildmode=pie -ldflags "${GOLDFLAGS}" -o $cmd ./cmd/$cmd
done

%install
install -d %{buildroot}%{_cross_libexecdir}/hotdog/
install -d %{buildroot}%{_cross_datadir}/hotdog/

install -p -m 0755 hotdog-hotpatch %{buildroot}%{_cross_datadir}/hotdog/

for cmd in hotdog-cc-hook hotdog-poststart-hook; do
  install -p -m 0755 $cmd %{buildroot}%{_cross_libexecdir}/hotdog
done

%files
%license LICENSE LICENSE.runtime-spec LICENSE.golang-sys LICENSE.go-selinux LICENSE.libcap
%{_cross_attribution_file}
%dir %{_cross_libexecdir}/hotdog
%dir %{_cross_datadir}/hotdog
%{_cross_libexecdir}/hotdog/hotdog-cc-hook
%{_cross_libexecdir}/hotdog/hotdog-poststart-hook
%{_cross_datadir}/hotdog/hotdog-hotpatch
