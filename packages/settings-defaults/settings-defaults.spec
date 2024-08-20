%global _cross_first_party 1
%undefine _debugsource_packages

%global cargo_clean %{__cargo_cross_env} %{__cargo} clean

%global _cross_defaultsdir %{_cross_datadir}/storewolf

Name: %{_cross_os}settings-defaults
Version: 0.0
Release: 0%{?dist}
Summary: Settings defaults
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}settings-defaults(any)

%description
%{summary}.

%package aws-dev
Summary: Settings defaults for the aws-dev variant
Requires: %{_cross_os}variant(aws-dev)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-dev)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-dev
%{summary}.

%package aws-ecs-1
Summary: Settings defaults for the aws-ecs-1 variant
Requires: %{_cross_os}variant(aws-ecs-1)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-1)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-1
%{summary}.

%package aws-ecs-1-nvidia
Summary: Settings defaults for the aws-ecs-1-nvidia variant
Requires: %{_cross_os}variant(aws-ecs-1-nvidia)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-1-nvidia)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-1-nvidia
%{summary}.

%package aws-ecs-2
Summary: Settings defaults for the aws-ecs-2 variant
Requires: %{_cross_os}variant(aws-ecs-2)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-2)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-2
%{summary}.

%package aws-ecs-2-nvidia
Summary: Settings defaults for the aws-ecs-2-nvidia variant
Requires: %{_cross_os}variant(aws-ecs-2-nvidia)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-2-nvidia)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-2-nvidia
%{summary}.

%package aws-k8s-1.24
Summary: Settings defaults for the aws-k8s 1.23 and 1.24 variants
Requires: (%{_cross_os}variant(aws-k8s-1.23) or %{_cross_os}variant(aws-k8s-1.24))
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.23)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.24)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.24
%{summary}.

%package aws-k8s-1.24-nvidia
Summary: Settings defaults for the aws-k8s 1.23 and 1.24 nvidia variants
Requires: (%{_cross_os}variant(aws-k8s-1.23-nvidia) or %{_cross_os}variant(aws-k8s-1.24-nvidia))
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.23-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.24-nvidia)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.24-nvidia
%{summary}.

%package aws-k8s-1.25
Summary: Settings defaults for the aws-k8s-1.25 variant
Requires: %{_cross_os}variant(aws-k8s-1.25)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.25)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.25
%{summary}.

%package aws-k8s-1.25-nvidia
Summary: Settings defaults for the aws-k8s-1.25-nvidia variant
Requires: %{_cross_os}variant(aws-k8s-1.25-nvidia)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.25-nvidia)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.25-nvidia
%{summary}.

%package aws-k8s-1.26
Summary: Settings defaults for the aws-k8s-1.26 variant
Requires: %{_cross_os}variant(aws-k8s-1.26)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.26)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.26
%{summary}.

%package aws-k8s-1.26-nvidia
Summary: Settings defaults for the aws-k8s-1.26-nvidia variant
Requires: %{_cross_os}variant(aws-k8s-1.26-nvidia)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.26-nvidia)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.26-nvidia
%{summary}.

%package aws-k8s-1.31
Summary: Settings defaults for the aws-k8s 1.27 through 1.30 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.27) or
           %{_cross_os}variant(aws-k8s-1.28) or
           %{_cross_os}variant(aws-k8s-1.29) or
           %{_cross_os}variant(aws-k8s-1.30) or
           %{_cross_os}variant(aws-k8s-1.31)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.27)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.28)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.29)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.30)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.31)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.31
%{summary}.

%package aws-k8s-1.31-nvidia
Summary: Settings defaults for the aws-k8s 1.27 through 1.30 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.27-nvidia) or
           %{_cross_os}variant(aws-k8s-1.28-nvidia) or
           %{_cross_os}variant(aws-k8s-1.29-nvidia) or
           %{_cross_os}variant(aws-k8s-1.30-nvidia) or
           %{_cross_os}variant(aws-k8s-1.31-nvidia)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.27-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.28-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.29-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.30-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.31-nvidia)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.31-nvidia
%{summary}.

%package metal-dev
Summary: Settings defaults for the metal-dev variant
Requires: %{_cross_os}variant(metal-dev)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(metal-dev)
Conflicts: %{_cross_os}settings-defaults(any)

%description metal-dev
%{summary}.

%package metal-k8s-1.30
Summary: Settings defaults for the metal-k8s 1.27 through 1.30 variants
Requires: (%{shrink:
           %{_cross_os}variant(metal-k8s-1.27) or
           %{_cross_os}variant(metal-k8s-1.28) or
           %{_cross_os}variant(metal-k8s-1.29) or
           %{_cross_os}variant(metal-k8s-1.30)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(metal-k8s-1.27)
Provides: %{_cross_os}settings-defaults(metal-k8s-1.28)
Provides: %{_cross_os}settings-defaults(metal-k8s-1.29)
Provides: %{_cross_os}settings-defaults(metal-k8s-1.30)
Conflicts: %{_cross_os}settings-defaults(any)

%description metal-k8s-1.30
%{summary}.

%package vmware-dev
Summary: Settings defaults for the vmware-dev variant
Requires: %{_cross_os}variant(vmware-dev)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-dev)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-dev
%{summary}.

%package vmware-k8s-1.31
Summary: Settings defaults for the vmware-k8s 1.27 through 1.30 variants
Requires: (%{shrink:
           %{_cross_os}variant(vmware-k8s-1.27) or
           %{_cross_os}variant(vmware-k8s-1.28) or
           %{_cross_os}variant(vmware-k8s-1.29) or
           %{_cross_os}variant(vmware-k8s-1.30) or
           %{_cross_os}variant(vmware-k8s-1.31)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.27)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.28)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.29)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.30)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.31)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-k8s-1.31
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
declare -a projects
for defaults in \
  aws-dev \
  aws-ecs-1 \
  aws-ecs-1-nvidia \
  aws-ecs-2 \
  aws-ecs-2-nvidia \
  aws-k8s-1.24 \
  aws-k8s-1.24-nvidia \
  aws-k8s-1.25 \
  aws-k8s-1.25-nvidia \
  aws-k8s-1.26 \
  aws-k8s-1.26-nvidia \
  aws-k8s-1.31 \
  aws-k8s-1.31-nvidia \
  metal-dev \
  metal-k8s-1.30 \
  vmware-dev \
  vmware-k8s-1.31 \
  ;
do
  projects+=( "-p" "settings-defaults-$(echo "${defaults}" | sed -e 's,\.,_,g')" )
done

# Output is written to an unpredictable directory name, so clean it up first to
# avoid reusing any cached artifacts.
%cargo_clean --manifest-path %{_builddir}/sources/Cargo.toml \
  "${projects[@]}" \
  %{nil}

%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
  "${projects[@]}" \
  %{nil}

%install
install -d %{buildroot}%{_cross_defaultsdir}
install -d %{buildroot}%{_cross_tmpfilesdir}

for defaults in \
  aws-dev \
  aws-ecs-1 \
  aws-ecs-1-nvidia \
  aws-ecs-2 \
  aws-ecs-2-nvidia \
  aws-k8s-1.24 \
  aws-k8s-1.24-nvidia \
  aws-k8s-1.25 \
  aws-k8s-1.25-nvidia \
  aws-k8s-1.26 \
  aws-k8s-1.26-nvidia \
  aws-k8s-1.31 \
  aws-k8s-1.31-nvidia \
  metal-dev \
  metal-k8s-1.30 \
  vmware-dev \
  vmware-k8s-1.31 \
  ;
do
  crate="$(echo "${defaults}" | sed -e 's,\.,_,g')"
  for f in $(find "${HOME}/.cache" -name "settings-defaults-${crate}.toml") ; do
    install -p -m 0644 "${f}" "%{buildroot}%{_cross_defaultsdir}/${defaults}.toml"
  done
  echo \
    "L+ /etc/storewolf/defaults.toml - - - - %{_cross_defaultsdir}/${defaults}.toml" > \
    "%{buildroot}%{_cross_tmpfilesdir}/storewolf-defaults-${defaults}.conf"
done

%files
%dir %{_cross_defaultsdir}

%files aws-dev
%{_cross_defaultsdir}/aws-dev.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-dev.conf

%files aws-ecs-1
%{_cross_defaultsdir}/aws-ecs-1.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-1.conf

%files aws-ecs-1-nvidia
%{_cross_defaultsdir}/aws-ecs-1-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-1-nvidia.conf

%files aws-ecs-2
%{_cross_defaultsdir}/aws-ecs-2.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-2.conf

%files aws-ecs-2-nvidia
%{_cross_defaultsdir}/aws-ecs-2-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-2-nvidia.conf

%files aws-k8s-1.24
%{_cross_defaultsdir}/aws-k8s-1.24.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.24.conf

%files aws-k8s-1.24-nvidia
%{_cross_defaultsdir}/aws-k8s-1.24-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.24-nvidia.conf

%files aws-k8s-1.25
%{_cross_defaultsdir}/aws-k8s-1.25.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.25.conf

%files aws-k8s-1.25-nvidia
%{_cross_defaultsdir}/aws-k8s-1.25-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.25-nvidia.conf

%files aws-k8s-1.26
%{_cross_defaultsdir}/aws-k8s-1.26.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.26.conf

%files aws-k8s-1.26-nvidia
%{_cross_defaultsdir}/aws-k8s-1.26-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.26-nvidia.conf

%files aws-k8s-1.31
%{_cross_defaultsdir}/aws-k8s-1.31.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.31.conf

%files aws-k8s-1.31-nvidia
%{_cross_defaultsdir}/aws-k8s-1.31-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.31-nvidia.conf

%files metal-dev
%{_cross_defaultsdir}/metal-dev.toml
%{_cross_tmpfilesdir}/storewolf-defaults-metal-dev.conf

%files metal-k8s-1.30
%{_cross_defaultsdir}/metal-k8s-1.30.toml
%{_cross_tmpfilesdir}/storewolf-defaults-metal-k8s-1.30.conf

%files vmware-dev
%{_cross_defaultsdir}/vmware-dev.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-dev.conf

%files vmware-k8s-1.31
%{_cross_defaultsdir}/vmware-k8s-1.31.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-k8s-1.31.conf
