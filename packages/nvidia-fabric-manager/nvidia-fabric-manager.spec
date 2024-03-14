%global _enable_debug_package 0
%global debug_package %{nil}
%global __os_install_post /usr/lib/rpm/brp-compress %{nil}
%global rpmver 550.54.14
%global branch 550

%if "%{_cross_arch}" == "aarch64"
%global arch sbsa
%endif

Name:           nvidia-fabric-manager
Version:        %{rpmver}
Release:        1
Summary:        Fabric Manager for NVSwitch based systems

License:        NVIDIA Proprietary
URL:            http://www.nvidia.com
Source0:        fabricmanager-linux-%{?arch:%{arch}}%{!?arch:%{_cross_arch}}-%{rpmver}-archive.tar.xz

Provides:       nvidia-fabricmanager = %{rpmver}
Provides:       nvidia-fabricmanager-%{branch} = %{rpmver}
Obsoletes:      nvidia-fabricmanager-branch < %{rpmver}
Obsoletes:      nvidia-fabricmanager < %{rpmver}

%description
Fabric Manager for NVIDIA NVSwitch based systems.

%package -n nvidia-fabric-manager-devel
Summary:        Fabric Manager API headers and associated library
# Normally we would have a dev package depend on its runtime package. However
# FM isn't a normal package. All the libs are in the dev package, and the
# runtime package is actually a service package.
Provides:       nvidia-fabricmanager-devel-%{branch} = %{rpmver}
Obsoletes:      nvidia-fabricmanager-devel-branch < %{rpmver}
Obsoletes:      nvidia-fabricmanager-devel < %{rpmver}

%description -n nvidia-fabric-manager-devel
Fabric Manager API headers and associated library

%package -n cuda-drivers-fabricmanager-%{branch}
Summary:        Meta-package for FM and Driver
Requires:       nvidia-fabric-manager = %{rpmver}
Requires:       cuda-drivers-%{branch} = %{rpmver}

Obsoletes:      cuda-drivers-fabricmanager-branch < %{rpmver}
Conflicts:      cuda-drivers-fabricmanager-%{branch} < %{rpmver}
Conflicts:      cuda-drivers-fabricmanager-branch

%description -n cuda-drivers-fabricmanager-%{branch}
Convience meta-package for installing fabricmanager and the cuda-drivers
meta-package simultaneously while keeping version equivalence. This meta-
package is branch-specific.

%package -n cuda-drivers-fabricmanager
Summary:        Meta-package for FM and Driver
Requires:       cuda-drivers-fabricmanager-%{branch} = %{rpmver}

%description -n cuda-drivers-fabricmanager
Convience meta-package for installing fabricmanager and the cuda-drivers
meta-package simultaneously while keeping version equivalence. This meta-
package is across all driver branches.

%prep
%setup -q -n fabricmanager-linux-%{?arch:%{arch}}%{!?arch:%{_cross_arch}}-%{rpmver}-archive

%build

%install
export DONT_STRIP=1

rm -rf %{buildroot}

mkdir -p %{buildroot}%{_bindir}/
cp -a bin/nv-fabricmanager %{buildroot}%{_bindir}/
cp -a bin/nvswitch-audit %{buildroot}%{_bindir}/

mkdir -p %{buildroot}/usr/lib/systemd/system
cp -a systemd/nvidia-fabricmanager.service  %{buildroot}/usr/lib/systemd/system

mkdir -p %{buildroot}/usr/share/nvidia/nvswitch
cp -a share/nvidia/nvswitch/*_topology %{buildroot}/usr/share/nvidia/nvswitch
cp -a etc/fabricmanager.cfg %{buildroot}/usr/share/nvidia/nvswitch

mkdir -p %{buildroot}%{_libdir}/
cp lib/libnvfm.so.1 %{buildroot}%{_libdir}/
ln -s lib/libnvfm.so.1 %{buildroot}%{_libdir}/libnvfm.so

mkdir -p %{buildroot}%{_includedir}/
cp include/nv_fm_agent.h %{buildroot}%{_includedir}/
cp include/nv_fm_types.h %{buildroot}%{_includedir}/

%post -n nvidia-fabric-manager-devel -p /sbin/ldconfig

%postun -n nvidia-fabric-manager-devel -p /sbin/ldconfig

%files
%{_bindir}/*
%license LICENSE
%{_cross_attribution_file}
/usr/lib/systemd/system/*
/usr/share/nvidia/nvswitch/*
%exclude /usr/share/nvidia/nvswitch/fabricmanager.cfg
%config(noreplace) /usr/share/nvidia/nvswitch/fabricmanager.cfg

%files -n nvidia-fabric-manager-devel
%{_libdir}/*
%{_includedir}/*

%files -n cuda-drivers-fabricmanager-%{branch}

%files -n cuda-drivers-fabricmanager
