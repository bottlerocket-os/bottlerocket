%global _cross_first_party 1
%undefine _debugsource_packages

# Do not prefer shared linking, since the libstd we use at build time
# may not match the one installed on the final image.
%global __global_rustflags_shared %__global_rustflags -C link-arg=-Wl,-soname=libsettings.so

%global _cross_pluginsdir %{_cross_libdir}/settings-plugins

Name: %{_cross_os}settings-plugins
Version: 0.0
Release: 1%{?dist}
Summary: Settings plugins
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}settings-plugin(any)

%description
%{summary}.

%package aws-dev
Summary: Settings plugin for the aws-dev variant
Requires: %{_cross_os}variant(aws-dev)
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(aws-dev)
Conflicts: %{_cross_os}settings-plugin(any)

%description aws-dev
%{summary}.

%package aws-ecs-2
Summary: Settings plugin for the aws-ecs-2 variant
Requires: (%{shrink:
           %{_cross_os}variant(aws-ecs-2) or
           %{_cross_os}variant(aws-ecs-2-fips) or
           %{_cross_os}variant(aws-ecs-2-nvidia) or
           %{_cross_os}variant(aws-ecs-2-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(aws-ecs-2)
Provides: %{_cross_os}settings-plugin(aws-ecs-2-nvidia)
Provides: %{_cross_os}settings-plugin(aws-ecs-2-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-ecs-2-fips)
Conflicts: %{_cross_os}settings-plugin(any)

%description aws-ecs-2
%{summary}.

%package aws-ecs-3
Summary: Settings plugin for the aws-ecs-3 variant
Requires: (%{shrink:
           %{_cross_os}variant(aws-ecs-3) or
           %{_cross_os}variant(aws-ecs-3-fips) or
           %{_cross_os}variant(aws-ecs-3-nvidia) or
           %{_cross_os}variant(aws-ecs-3-nvidia-fips)
           %{nil}})
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(aws-ecs-3)
Provides: %{_cross_os}settings-plugin(aws-ecs-3-nvidia)
Provides: %{_cross_os}settings-plugin(aws-ecs-3-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-ecs-3-fips)
Conflicts: %{_cross_os}settings-plugin(any)

%description aws-ecs-3
%{summary}.

%package aws-k8s
Summary: Settings plugin for the aws-k8s variants
Requires: %{_cross_os}variant-family(aws-k8s)
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.30)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.30-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.31)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.31-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.32)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.32-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.33)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.33-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.34)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.34-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.35)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.35-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.36)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.36-fips)
Conflicts: %{_cross_os}settings-plugin(any)
Conflicts: %{_cross_os}variant-flavor(nvidia)


%description aws-k8s
%{summary}.

%package aws-k8s-nvidia
Summary: Settings plugin for the aws-k8s-nvidia variants
Requires: (%{_cross_os}variant-family(aws-k8s) and %{_cross_os}variant-flavor(nvidia))
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.30-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.30-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.31-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.31-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.32-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.32-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.33-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.33-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.34-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.34-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.35-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.35-nvidia-fips)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.36-nvidia)
Provides: %{_cross_os}settings-plugin(aws-k8s-1.36-nvidia-fips)
Conflicts: %{_cross_os}settings-plugin(any)

%description aws-k8s-nvidia
%{summary}.

%package metal-dev
Summary: Settings plugin for the metal-dev variant
Requires: %{_cross_os}variant(metal-dev)
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(metal-dev)
Conflicts: %{_cross_os}settings-plugin(any)

%description metal-dev
%{summary}.

%package vmware-dev
Summary: Settings plugin for the vmware-dev variant
Requires: %{_cross_os}variant(vmware-dev)
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(vmware-dev)
Conflicts: %{_cross_os}settings-plugin(any)

%description vmware-dev
%{summary}.

%package vmware-k8s
Summary: Settings plugin for the vmware-k8s variants
Requires: %{_cross_os}variant-family(vmware-k8s)
Provides: %{_cross_os}settings-plugin(any)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.30)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.30-fips)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.31)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.31-fips)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.32)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.32-fips)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.33)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.33-fips)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.34)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.34-fips)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.35)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.35-fips)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.36)
Provides: %{_cross_os}settings-plugin(vmware-k8s-1.36-fips)
Conflicts: %{_cross_os}settings-plugin(any)

%description vmware-k8s
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
  -p settings-plugin-aws-dev \
  -p settings-plugin-aws-ecs-2 \
  -p settings-plugin-aws-ecs-3 \
  -p settings-plugin-aws-k8s \
  -p settings-plugin-aws-k8s-nvidia \
  -p settings-plugin-metal-dev \
  -p settings-plugin-vmware-dev \
  -p settings-plugin-vmware-k8s \
  %{nil}

%install
install -d %{buildroot}%{_cross_pluginsdir}
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d
install -d %{buildroot}%{_cross_tmpfilesdir}

for plugin in \
  aws-dev \
  aws-ecs-2 \
  aws-ecs-3 \
  aws-k8s-nvidia \
  aws-k8s \
  metal-dev \
  vmware-dev \
  vmware-k8s \
  ;
do
  install -d "%{buildroot}%{_cross_pluginsdir}/${plugin}"
  plugin_so="libsettings_$(echo "${plugin}" | sed -e 's,-,_,g' -e 's,\.,_,g').so"
  install -p -m 0755 \
    "${HOME}/.cache/%{__cargo_target}/release/${plugin_so}" \
    "%{buildroot}%{_cross_pluginsdir}/${plugin}/libsettings.so"
  echo \
    "%{_cross_pluginsdir}/${plugin}" > \
    "%{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/${plugin}.conf"
  echo \
    "C /etc/ld.so.conf.d/${plugin}.conf" > \
    "%{buildroot}%{_cross_tmpfilesdir}/settings-plugin-${plugin}.conf"
done

%files
%dir %{_cross_pluginsdir}

%files aws-dev
%{_cross_pluginsdir}/aws-dev/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/aws-dev.conf
%{_cross_tmpfilesdir}/settings-plugin-aws-dev.conf

%files aws-ecs-2
%{_cross_pluginsdir}/aws-ecs-2/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/aws-ecs-2.conf
%{_cross_tmpfilesdir}/settings-plugin-aws-ecs-2.conf

%files aws-ecs-3
%{_cross_pluginsdir}/aws-ecs-3/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/aws-ecs-3.conf
%{_cross_tmpfilesdir}/settings-plugin-aws-ecs-3.conf

%files aws-k8s
%{_cross_pluginsdir}/aws-k8s/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/aws-k8s.conf
%{_cross_tmpfilesdir}/settings-plugin-aws-k8s.conf

%files aws-k8s-nvidia
%{_cross_pluginsdir}/aws-k8s-nvidia/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/aws-k8s-nvidia.conf
%{_cross_tmpfilesdir}/settings-plugin-aws-k8s-nvidia.conf

%files metal-dev
%{_cross_pluginsdir}/metal-dev/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/metal-dev.conf
%{_cross_tmpfilesdir}/settings-plugin-metal-dev.conf

%files vmware-dev
%{_cross_pluginsdir}/vmware-dev/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/vmware-dev.conf
%{_cross_tmpfilesdir}/settings-plugin-vmware-dev.conf

%files vmware-k8s
%{_cross_pluginsdir}/vmware-k8s/libsettings.so
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/vmware-k8s.conf
%{_cross_tmpfilesdir}/settings-plugin-vmware-k8s.conf
