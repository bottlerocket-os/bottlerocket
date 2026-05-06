%global _cross_first_party 1
%undefine _debugsource_packages

%global cargo_clean %{__cargo_cross_env} %{__cargo} clean

%global _cross_defaultsdir %{_cross_datadir}/storewolf

Name: %{_cross_os}settings-defaults
Version: 0.0
Release: 1%{?dist}
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

%package aws-ecs-2
Summary: Settings defaults for the aws-ecs-2 FIPS and non-FIPS variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-ecs-2) or
           %{_cross_os}variant(aws-ecs-2-fips)
          %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-2)
Provides: %{_cross_os}settings-defaults(aws-ecs-2-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-2
%{summary}.

%package aws-ecs-2-nvidia
Summary: Settings defaults for the aws-ecs-2-nvidia variant
Requires: (%{shrink:
           %{_cross_os}variant(aws-ecs-2-nvidia) or
           %{_cross_os}variant(aws-ecs-2-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-2-nvidia)
Provides: %{_cross_os}settings-defaults(aws-ecs-2-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-2-nvidia
%{summary}.

%package aws-ecs-3
Summary: Settings defaults for the aws-ecs-3 FIPS and non-FIPS variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-ecs-3) or
           %{_cross_os}variant(aws-ecs-3-fips)
          %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-3)
Provides: %{_cross_os}settings-defaults(aws-ecs-3-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-3
%{summary}.

%package aws-ecs-3-nvidia
Summary: Settings defaults for the aws-ecs-3-nvidia variant
Requires: (%{shrink:
           %{_cross_os}variant(aws-ecs-3-nvidia) or
           %{_cross_os}variant(aws-ecs-3-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-ecs-3-nvidia)
Provides: %{_cross_os}settings-defaults(aws-ecs-3-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-ecs-3-nvidia
%{summary}.

%package aws-k8s-1.31
Summary: Settings defaults for the aws-k8s 1.30 through 1.31 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.30)      or
           %{_cross_os}variant(aws-k8s-1.30-fips) or
           %{_cross_os}variant(aws-k8s-1.31)      or
           %{_cross_os}variant(aws-k8s-1.31-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.30)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.30-fips)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.31)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.31-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.31
%{summary}.

%package aws-k8s-1.31-nvidia
Summary: Settings defaults for the aws-k8s 1.30 through 1.31 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.30-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.30-nvidia-fips) or
           %{_cross_os}variant(aws-k8s-1.31-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.31-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.30-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.30-nvidia-fips)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.31-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.31-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.31-nvidia
%{summary}.

%package aws-k8s-1.32
Summary: Settings defaults for the aws-k8s 1.32 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.32)      or
           %{_cross_os}variant(aws-k8s-1.32-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.32)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.32-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.32
%{summary}.

%package aws-k8s-1.32-nvidia
Summary: Settings defaults for the aws-k8s 1.32 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.32-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.32-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.32-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.32-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.32-nvidia
%{summary}.

%package aws-k8s-1.33
Summary: Settings defaults for the aws-k8s 1.33 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.33)      or
           %{_cross_os}variant(aws-k8s-1.33-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.33)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.33-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.33
%{summary}.

%package aws-k8s-1.33-nvidia
Summary: Settings defaults for the aws-k8s 1.33 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.33-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.33-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.33-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.33-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.33-nvidia
%{summary}.

%package aws-k8s-1.34
Summary: Settings defaults for the aws-k8s 1.34 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.34)      or
           %{_cross_os}variant(aws-k8s-1.34-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.34)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.34-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.34
%{summary}.

%package aws-k8s-1.34-nvidia
Summary: Settings defaults for the aws-k8s 1.34 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.34-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.34-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.34-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.34-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.34-nvidia
%{summary}.

%package aws-k8s-1.35
Summary: Settings defaults for the aws-k8s 1.35 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.35)      or
           %{_cross_os}variant(aws-k8s-1.35-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.35)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.35-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.35
%{summary}.

%package aws-k8s-1.35-nvidia
Summary: Settings defaults for the aws-k8s 1.35 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.35-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.35-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.35-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.35-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.35-nvidia
%{summary}.

%package aws-k8s-1.36
Summary: Settings defaults for the aws-k8s 1.36 variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.36)      or
           %{_cross_os}variant(aws-k8s-1.36-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.36)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.36-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.36
%{summary}.

%package aws-k8s-1.36-nvidia
Summary: Settings defaults for the aws-k8s 1.36 nvidia variants
Requires: (%{shrink:
           %{_cross_os}variant(aws-k8s-1.36-nvidia)      or
           %{_cross_os}variant(aws-k8s-1.36-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.36-nvidia)
Provides: %{_cross_os}settings-defaults(aws-k8s-1.36-nvidia-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description aws-k8s-1.36-nvidia
%{summary}.

%package metal-dev
Summary: Settings defaults for the metal-dev variant
Requires: %{_cross_os}variant(metal-dev)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(metal-dev)
Conflicts: %{_cross_os}settings-defaults(any)

%description metal-dev
%{summary}.

%package vmware-dev
Summary: Settings defaults for the vmware-dev variant
Requires: %{_cross_os}variant(vmware-dev)
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-dev)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-dev
%{summary}.

%package vmware-k8s-1.32
Summary: Settings defaults for the vmware-k8s 1.30 through 1.32 variants
Requires: (%{shrink:
           %{_cross_os}variant(vmware-k8s-1.30)      or
           %{_cross_os}variant(vmware-k8s-1.30-fips) or
           %{_cross_os}variant(vmware-k8s-1.31)      or
           %{_cross_os}variant(vmware-k8s-1.31-fips) or
           %{_cross_os}variant(vmware-k8s-1.32)      or
           %{_cross_os}variant(vmware-k8s-1.32-fips)
          %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.30)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.30-fips)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.31)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.31-fips)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.32)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.32-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-k8s-1.32
%{summary}.

%package vmware-k8s-1.33
Summary: Settings defaults for the vmware-k8s 1.33 variants
Requires: (%{shrink:
           %{_cross_os}variant(vmware-k8s-1.33)      or
           %{_cross_os}variant(vmware-k8s-1.33-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.33)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.33-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-k8s-1.33
%{summary}.

%package vmware-k8s-1.34
Summary: Settings defaults for the vmware-k8s 1.34 variants
Requires: (%{shrink:
           %{_cross_os}variant(vmware-k8s-1.34)      or
           %{_cross_os}variant(vmware-k8s-1.34-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.34)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.34-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-k8s-1.34
%{summary}.

%package vmware-k8s-1.35
Summary: Settings defaults for the vmware-k8s 1.35 variants
Requires: (%{shrink:
           %{_cross_os}variant(vmware-k8s-1.35)      or
           %{_cross_os}variant(vmware-k8s-1.35-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.35)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.35-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-k8s-1.35
%{summary}.

%package vmware-k8s-1.36
Summary: Settings defaults for the vmware-k8s 1.35 variants
Requires: (%{shrink:
           %{_cross_os}variant(vmware-k8s-1.36)      or
           %{_cross_os}variant(vmware-k8s-1.36-fips)
           %{nil}})
Provides: %{_cross_os}settings-defaults(any)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.36)
Provides: %{_cross_os}settings-defaults(vmware-k8s-1.36-fips)
Conflicts: %{_cross_os}settings-defaults(any)

%description vmware-k8s-1.36
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
declare -a projects
for defaults in \
  aws-dev \
  aws-ecs-2 \
  aws-ecs-2-nvidia \
  aws-ecs-3 \
  aws-ecs-3-nvidia \
  aws-k8s-1.31 \
  aws-k8s-1.31-nvidia \
  aws-k8s-1.32 \
  aws-k8s-1.32-nvidia \
  aws-k8s-1.33 \
  aws-k8s-1.33-nvidia \
  aws-k8s-1.34 \
  aws-k8s-1.34-nvidia \
  aws-k8s-1.35 \
  aws-k8s-1.35-nvidia \
  aws-k8s-1.36 \
  aws-k8s-1.36-nvidia \
  metal-dev \
  vmware-dev \
  vmware-k8s-1.32 \
  vmware-k8s-1.33 \
  vmware-k8s-1.34 \
  vmware-k8s-1.35 \
  vmware-k8s-1.36 \
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
  aws-ecs-2 \
  aws-ecs-2-nvidia \
  aws-ecs-3 \
  aws-ecs-3-nvidia \
  aws-k8s-1.31 \
  aws-k8s-1.31-nvidia \
  aws-k8s-1.32 \
  aws-k8s-1.32-nvidia \
  aws-k8s-1.33 \
  aws-k8s-1.33-nvidia \
  aws-k8s-1.34 \
  aws-k8s-1.34-nvidia \
  aws-k8s-1.35 \
  aws-k8s-1.35-nvidia \
  aws-k8s-1.36 \
  aws-k8s-1.36-nvidia \
  metal-dev \
  vmware-dev \
  vmware-k8s-1.32 \
  vmware-k8s-1.33 \
  vmware-k8s-1.34 \
  vmware-k8s-1.35 \
  vmware-k8s-1.36 \
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

%files aws-ecs-2
%{_cross_defaultsdir}/aws-ecs-2.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-2.conf

%files aws-ecs-2-nvidia
%{_cross_defaultsdir}/aws-ecs-2-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-2-nvidia.conf

%files aws-ecs-3
%{_cross_defaultsdir}/aws-ecs-3.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-3.conf

%files aws-ecs-3-nvidia
%{_cross_defaultsdir}/aws-ecs-3-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-ecs-3-nvidia.conf

%files aws-k8s-1.31
%{_cross_defaultsdir}/aws-k8s-1.31.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.31.conf

%files aws-k8s-1.31-nvidia
%{_cross_defaultsdir}/aws-k8s-1.31-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.31-nvidia.conf

%files aws-k8s-1.32
%{_cross_defaultsdir}/aws-k8s-1.32.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.32.conf

%files aws-k8s-1.32-nvidia
%{_cross_defaultsdir}/aws-k8s-1.32-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.32-nvidia.conf

%files aws-k8s-1.33
%{_cross_defaultsdir}/aws-k8s-1.33.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.33.conf

%files aws-k8s-1.33-nvidia
%{_cross_defaultsdir}/aws-k8s-1.33-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.33-nvidia.conf

%files aws-k8s-1.34
%{_cross_defaultsdir}/aws-k8s-1.34.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.34.conf

%files aws-k8s-1.34-nvidia
%{_cross_defaultsdir}/aws-k8s-1.34-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.34-nvidia.conf

%files aws-k8s-1.35
%{_cross_defaultsdir}/aws-k8s-1.35.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.35.conf

%files aws-k8s-1.35-nvidia
%{_cross_defaultsdir}/aws-k8s-1.35-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.35-nvidia.conf

%files aws-k8s-1.36
%{_cross_defaultsdir}/aws-k8s-1.36.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.36.conf

%files aws-k8s-1.36-nvidia
%{_cross_defaultsdir}/aws-k8s-1.36-nvidia.toml
%{_cross_tmpfilesdir}/storewolf-defaults-aws-k8s-1.36-nvidia.conf

%files metal-dev
%{_cross_defaultsdir}/metal-dev.toml
%{_cross_tmpfilesdir}/storewolf-defaults-metal-dev.conf

%files vmware-dev
%{_cross_defaultsdir}/vmware-dev.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-dev.conf

%files vmware-k8s-1.32
%{_cross_defaultsdir}/vmware-k8s-1.32.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-k8s-1.32.conf

%files vmware-k8s-1.33
%{_cross_defaultsdir}/vmware-k8s-1.33.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-k8s-1.33.conf

%files vmware-k8s-1.34
%{_cross_defaultsdir}/vmware-k8s-1.34.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-k8s-1.34.conf

%files vmware-k8s-1.35
%{_cross_defaultsdir}/vmware-k8s-1.35.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-k8s-1.35.conf

%files vmware-k8s-1.36
%{_cross_defaultsdir}/vmware-k8s-1.36.toml
%{_cross_tmpfilesdir}/storewolf-defaults-vmware-k8s-1.36.conf
