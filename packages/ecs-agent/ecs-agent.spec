%global agent_goproject github.com/aws
%global agent_gorepo amazon-ecs-agent
%global agent_goimport %{agent_goproject}/%{agent_gorepo}

%global agent_gover 1.75.0

# git rev-parse --short=8
%global agent_gitrev 06008fa1

%global ecscni_goproject github.com/aws
%global ecscni_gorepo amazon-ecs-cni-plugins
%global ecscni_goimport %{ecscni_goproject}/%{ecscni_gorepo}
%global ecscni_gitrev 53a8481891251e66e35847554d52a13fc7c4fd03

%global vpccni_goproject github.com/aws
%global vpccni_gorepo amazon-vpc-cni-plugins
%global vpccni_goimport %{vpccni_goproject}/%{vpccni_gorepo}
%global vpccni_gitrev a83b66349768e020487a00e31767fc2e6fc88136
%global vpccni_gover 1.3

# Construct reproducible tar archives
# See https://reproducible-builds.org/docs/archives/
%global source_date_epoch 1234567890
%global tar_cf tar --sort=name --mtime="@%{source_date_epoch}" --owner=0 --group=0 --numeric-owner -cf

Name: %{_cross_os}ecs-agent
Version: %{agent_gover}
Release: 1%{?dist}
Summary: Amazon Elastic Container Service agent
License: Apache-2.0
URL: https://%{agent_goimport}
Source0: https://%{agent_goimport}/archive/v%{agent_gover}/%{agent_gorepo}-%{agent_gover}.tar.gz
Source1: https://%{ecscni_goimport}/archive/%{ecscni_gitrev}/%{ecscni_gorepo}.tar.gz
Source2: https://%{vpccni_goimport}/archive/%{vpccni_gitrev}/%{vpccni_gorepo}.tar.gz
Source101: ecs.service
Source102: ecs-tmpfiles.conf
Source103: ecs-sysctl.conf
Source104: ecs.config
Source105: pause-image-VERSION
Source106: pause-config.json
Source107: pause-manifest.json
Source108: pause-repositories
# Bottlerocket-specific - version data can be set with linker options
Source109: version.go

# Mount for writing ECS agent configuration
Source200: etc-ecs.mount

# Patches are numbered according to which source they apply to
# Patches 0000 - 0999 apply to Source0
# Patches 1000 - 1999 apply to Source1
# Patches 2000 - 2999 apply to Source2
# See the %prep section for the implementation of this logic

# Bottlerocket-specific - filesystem location of the pause image
Patch0001: 0001-bottlerocket-default-filesystem-locations.patch

# Bottlerocket-specific - remove unsupported capabilities
Patch0002: 0002-bottlerocket-remove-unsupported-capabilities.patch

# bind introspection to localhost
# https://github.com/aws/amazon-ecs-agent/pull/2588
Patch0003: 0003-bottlerocket-bind-introspection-to-localhost.patch

# Bottlerocket-specific - fix procfs path for non-containerized ECS agent
Patch0004: 0004-bottlerocket-fix-procfs-path-on-host.patch

# Bottlerocket-specific - fix ECS exec directories
Patch0005: 0005-bottlerocket-change-execcmd-directories-for-Bottlero.patch

# Bottlerocket-specific - filesystem location for ECS CNI plugins
Patch1001: 1001-bottlerocket-default-filesystem-locations.patch

BuildRequires: %{_cross_os}glibc-devel

Requires: %{_cross_os}docker-engine
Requires: %{_cross_os}iptables
Requires: %{_cross_os}amazon-ssm-agent

%description
%{summary}.

%prep
# After prep runs, the directory setup looks like this:
# %{_builddir} [root]
# └── %{name}-%{version} [created by setup]
#     ├── amazon-ecs-agent-%{agent_gover} [top level of Source0]
#     │   └── [unpacked sources]
#     ├── amazon-ecs-cni-plugins-%{ecscni_gitrev} [top level of Source1]
#     │   └── [unpacked sources]
#     ├── amazon-vpc-cni-plugins-%{vpccni_gitrev} [top level of Source2]
#     │   └── [unpacked sources]
#     └── GOPATH
#         └── src/github.com/aws
#             ├── amazon-ecs-agent [symlink]
#             ├── amazon-ecs-cni-plugins [symlink]
#             └── amazon-vpc-cni-plugins [symlink]

# Extract Source0, which has a top-level directory of
# %{agent_gorepo}-%{agent_gover}
# -c: Create directory (%{name}-%{version})
# -q: Unpack quietly
%setup -c -q
# Change to the directory that we unpacked
cd %{agent_gorepo}-%{agent_gover}
# Set up git so we can apply patches
# This is included in autosetup, but not autopatch
%global __scm git
%__scm_setup_git
# Apply patches up to 0999
%autopatch -M 0999
# Replace upstream's version.go to support build-time values from ldflags. This
# avoids maintenance of patches that use always changing version-control tokens
# in its replacement.
cp %{S:109} "agent/version/version.go"

# Extract Source1, which has a top-level directory of
# %{ecscni_gorepo}-%{ecscni_gitrev}
# -T: Do not perform default archive unpack (i.e., skip Source0)
# -D: Do not delete directory before unpacking sources (i.e., don't delete
#     unpacked Source0)
# -a: Unpack after changing into the directory
# -q: Unpack quietly
# See http://ftp.rpm.org/max-rpm/s1-rpm-inside-macros.html
%setup -T -D -a 1 -q
# Change to the directory that we unpacked
cd %{ecscni_gorepo}-%{ecscni_gitrev}
# Set up git so we can apply patches
# This is included in autosetup, but not autopatch
%__scm_setup_git
# Apply patches from 1000 to 1999
%autopatch -m 1000 -M 1999

# Extract Source2, which has a top-level directory of
# %{vpccni_gorepo}-%{vpccni_gitrev}
# -T: Do not perform default archive unpack (i.e., skip Source0)
# -D: Do not delete directory before unpacking sources (i.e., don't delete
#     unpacked Source0)
# -a: Unpack after changing into the directory
# -q: Unpack quietly
# See http://ftp.rpm.org/max-rpm/s1-rpm-inside-macros.html
%setup -T -D -a 2 -q
# Change to the directory that we unpacked
cd %{vpccni_gorepo}-%{vpccni_gitrev}
# Set up git so we can apply patches
# This is included in autosetup, but not autopatch
%__scm_setup_git
# Apply patches from 2000 to 2999
%autopatch -m 2000 -M 2999

cd ../
# Symlink amazon-ecs-agent-%{agent_gover} to the GOPATH location
%cross_go_setup %{name}-%{version}/%{agent_gorepo}-%{agent_gover} %{agent_goproject} %{agent_goimport}
# Symlink amazon-ecs-cni-plugins-%{ecscni_gitrev} to the GOPATH location
%cross_go_setup %{name}-%{version}/%{ecscni_gorepo}-%{ecscni_gitrev} %{ecscni_goproject} %{ecscni_goimport}
# Symlink amazon-vpc-cni-plugins-%{vpccni_gitrev} to the GOPATH location
%cross_go_setup %{name}-%{version}/%{vpccni_gorepo}-%{vpccni_gitrev} %{vpccni_goproject} %{vpccni_goimport}

%build
BUILD_TOP=$(pwd -P)
# Build the agent
# cross_go_configure cd's to the correct GOPATH location
%cross_go_configure %{agent_goimport}
PAUSE_CONTAINER_IMAGE_NAME="amazon/amazon-ecs-pause"
PAUSE_CONTAINER_IMAGE_TAG="bottlerocket"
LD_PAUSE_CONTAINER_NAME="-X github.com/aws/amazon-ecs-agent/agent/config.DefaultPauseContainerImageName=${PAUSE_CONTAINER_IMAGE_NAME}"
LD_PAUSE_CONTAINER_TAG="-X github.com/aws/amazon-ecs-agent/agent/config.DefaultPauseContainerTag=${PAUSE_CONTAINER_IMAGE_TAG}"
LD_VERSION="-X github.com/aws/amazon-ecs-agent/agent/version.Version=%{agent_gover}"
LD_GIT_REV="-X github.com/aws/amazon-ecs-agent/agent/version.GitShortHash=%{agent_gitrev}"
go build -a \
  -buildmode=pie \
  -ldflags "${GOLDFLAGS} ${LD_PAUSE_CONTAINER_NAME} ${LD_PAUSE_CONTAINER_TAG} ${LD_VERSION} ${LD_GIT_REV}" \
  -o amazon-ecs-agent \
  ./agent

# Build the pause container
(
  set -x
  cd misc/pause-container/

  # Build static pause executable for container image.
  mkdir -p rootfs/usr/bin
  %{_cross_triple}-musl-gcc ${_cross_cflags} -static pause.c -o rootfs/usr/bin/pause

  # Construct container image.
  mkdir -p image/rootfs
  %tar_cf image/rootfs/layer.tar -C rootfs .
  DIGEST=$(sha256sum image/rootfs/layer.tar | sed -e 's/ .*//')
  install -m 0644 %{S:105} image/rootfs/VERSION
  install -m 0644 %{S:106} image/config.json
  sed -i "s/~~digest~~/${DIGEST}/" image/config.json
  install -m 0644 %{S:107} image/manifest.json
  install -m 0644 %{S:108} image/repositories
  %tar_cf ../../amazon-ecs-pause.tar -C image .
)

cd "${BUILD_TOP}"

# Build the ECS CNI plugins
# cross_go_configure cd's to the correct GOPATH location
%cross_go_configure %{ecscni_goimport}
LD_ECS_CNI_VERSION="-X github.com/aws/amazon-ecs-cni-plugins/pkg/version.Version=$(cat VERSION)"
ECS_CNI_HASH="%{ecscni_gitrev}"
LD_ECS_CNI_SHORT_HASH="-X github.com/aws/amazon-ecs-cni-plugins/pkg/version.GitShortHash=${ECS_CNI_HASH::8}"
LD_ECS_CNI_PORCELAIN="-X github.com/aws/amazon-ecs-cni-plugins/pkg/version.GitPorcelain=0"
go build -a \
  -buildmode=pie \
  -ldflags "${GOLDFLAGS} ${LD_ECS_CNI_VERSION} ${LD_ECS_CNI_SHORT_HASH} ${LD_ECS_CNI_PORCELAIN}" \
  -o ecs-eni \
  ./plugins/eni
go build -a \
  -buildmode=pie \
  -ldflags "${GOLDFLAGS} ${LD_ECS_CNI_VERSION} ${LD_ECS_CNI_SHORT_HASH} ${LD_ECS_CNI_PORCELAIN}" \
  -o ecs-ipam \
  ./plugins/ipam
go build -a \
  -buildmode=pie \
  -ldflags "${GOLDFLAGS} ${LD_ECS_CNI_VERSION} ${LD_ECS_CNI_SHORT_HASH} ${LD_ECS_CNI_PORCELAIN}" \
  -o ecs-bridge \
  ./plugins/ecs-bridge

cd "${BUILD_TOP}"

# Build the VPC CNI plugins
# cross_go_configure cd's to the correct GOPATH location
%cross_go_configure %{vpccni_goimport}
LD_VPC_CNI_VERSION="-X github.com/aws/amazon-vpc-cni-plugins/version.Version=%{vpccni_gover}"
VPC_CNI_HASH="%{vpccni_gitrev}"
LD_VPC_CNI_SHORT_HASH="-X github.com/aws/amazon-vpc-cni-plugins/version.GitShortHash=${VPC_CNI_HASH::8}"
go build -a \
  -buildmode=pie \
  -ldflags "${GOLDFLAGS} ${LD_VPC_CNI_VERSION} ${LD_VPC_CNI_SHORT_HASH} ${LD_VPC_CNI_PORCELAIN}" \
  -mod=vendor \
  -o vpc-branch-eni \
  ./plugins/vpc-branch-eni
go build -a \
  -buildmode=pie \
  -ldflags "${GOLDFLAGS} ${LD_VPC_CNI_VERSION} ${LD_VPC_CNI_SHORT_HASH} ${LD_VPC_CNI_PORCELAIN}" \
  -mod=vendor \
  -o aws-appmesh \
  ./plugins/aws-appmesh

%install
install -D -p -m 0755 %{agent_gorepo}-%{agent_gover}/amazon-ecs-agent %{buildroot}%{_cross_bindir}/amazon-ecs-agent
install -D -p -m 0644 %{agent_gorepo}-%{agent_gover}/amazon-ecs-pause.tar %{buildroot}%{_cross_libdir}/amazon-ecs-agent/amazon-ecs-pause.tar
install -D -p -m 0755 %{ecscni_gorepo}-%{ecscni_gitrev}/ecs-bridge %{buildroot}%{_cross_libexecdir}/amazon-ecs-agent/ecs-bridge
install -D -p -m 0755 %{ecscni_gorepo}-%{ecscni_gitrev}/ecs-eni %{buildroot}%{_cross_libexecdir}/amazon-ecs-agent/ecs-eni
install -D -p -m 0755 %{ecscni_gorepo}-%{ecscni_gitrev}/ecs-ipam %{buildroot}%{_cross_libexecdir}/amazon-ecs-agent/ecs-ipam
install -D -p -m 0755 %{vpccni_gorepo}-%{vpccni_gitrev}/vpc-branch-eni %{buildroot}%{_cross_libexecdir}/amazon-ecs-agent/vpc-branch-eni
install -D -p -m 0755 %{vpccni_gorepo}-%{vpccni_gitrev}/aws-appmesh %{buildroot}%{_cross_libexecdir}/amazon-ecs-agent/aws-appmesh

install -d %{buildroot}%{_cross_unitdir}
install -D -p -m 0644 %{S:101} %{S:200} %{buildroot}%{_cross_unitdir}

install -D -p -m 0644 %{S:102} %{buildroot}%{_cross_tmpfilesdir}/ecs.conf
install -D -p -m 0644 %{S:103} %{buildroot}%{_cross_sysctldir}/90-ecs.conf
install -D -p -m 0644 %{S:104} %{buildroot}%{_cross_templatedir}/ecs.config

# Directory for agents used by the ECS agent, e.g. SSM, Service Connect
%global managed_agents %{_cross_libexecdir}/amazon-ecs-agent/managed-agents
install -d %{buildroot}%{managed_agents}

# Directory for ECS exec artifacts
%global ecs_exec_dir %{managed_agents}/execute-command
install -d %{buildroot}%{ecs_exec_dir}

# The ECS agent looks for real versioned directories under bin, symlinks will be
# ignored. Thus, link the bin directory in the ssm-agent directory which contains
# the versioned binaries.
ln -rs %{buildroot}%{_cross_libexecdir}/amazon-ssm-agent/bin %{buildroot}/%{ecs_exec_dir}/bin

# The ECS agent generates and stores configurations for ECS exec sessions inside
# "config", thus reference it with a symlink to a directory under /var
ln -rs %{buildroot}%{_cross_localstatedir}/ecs/managed-agents/execute-command/config %{buildroot}%{ecs_exec_dir}/config

# Use the host's certificates bundle for ECS exec sessions
install -d %{buildroot}%{ecs_exec_dir}/certs
ln -rs %{buildroot}%{_cross_sysconfdir}/pki/tls/certs/ca-bundle.crt %{buildroot}%{ecs_exec_dir}/certs/tls-ca-bundle.pem

# Prepare license and vendor information so it can be co-installable
mv %{ecscni_gorepo}-%{ecscni_gitrev}/LICENSE %{ecscni_gorepo}-%{ecscni_gitrev}/LICENSE.%{ecscni_gorepo}
mv %{vpccni_gorepo}-%{vpccni_gitrev}/LICENSE %{vpccni_gorepo}-%{vpccni_gitrev}/LICENSE.%{vpccni_gorepo}
# Move vendor folder into a single directory so cross_scan_attribution can run once
mkdir go-vendor
mv %{agent_gorepo}-%{agent_gover}/agent/vendor go-vendor/%{agent_gorepo}
mv %{ecscni_gorepo}-%{ecscni_gitrev}/vendor go-vendor/%{ecscni_gorepo}
mv %{vpccni_gorepo}-%{vpccni_gitrev}/vendor go-vendor/%{vpccni_gorepo}
%cross_scan_attribution go-vendor go-vendor

%files
# License and attribution files are installed into /usr/share/licenses with a
# directory structure as follows:
# /usr/share/licenses/ecs-agent/
# ├── attribution.txt
# ├── LICENSE
# ├── LICENSE.amazon-ecs-cni-plugins
# ├── LICENSE.amazon-vpc-cni-plugins
# ├── NOTICE
# ├── THIRD_PARTY.md
# └── vendor
#     ├── amazon-ecs-agent
#     │ └── ...
#     ├── amazon-ecs-cni-plugins
#     │ └── ...
#     └── amazon-vpc-cni-plugins
#       └── ...

%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%license %{agent_gorepo}-%{agent_gover}/LICENSE
%license %{agent_gorepo}-%{agent_gover}/NOTICE
%license %{agent_gorepo}-%{agent_gover}/ecs-agent/THIRD_PARTY.md
%license %{ecscni_gorepo}-%{ecscni_gitrev}/LICENSE.%{ecscni_gorepo}
%license %{vpccni_gorepo}-%{vpccni_gitrev}/LICENSE.%{vpccni_gorepo}

%{_cross_bindir}/amazon-ecs-agent
%{_cross_libexecdir}/amazon-ecs-agent/ecs-bridge
%{_cross_libexecdir}/amazon-ecs-agent/ecs-eni
%{_cross_libexecdir}/amazon-ecs-agent/ecs-ipam
%{_cross_libexecdir}/amazon-ecs-agent/vpc-branch-eni
%{_cross_libexecdir}/amazon-ecs-agent/aws-appmesh
%{_cross_libexecdir}/amazon-ecs-agent/managed-agents
%{_cross_unitdir}/ecs.service
%{_cross_unitdir}/etc-ecs.mount
%{_cross_tmpfilesdir}/ecs.conf
%{_cross_sysctldir}/90-ecs.conf
%{_cross_templatedir}/ecs.config
%{_cross_libdir}/amazon-ecs-agent/amazon-ecs-pause.tar

%changelog
