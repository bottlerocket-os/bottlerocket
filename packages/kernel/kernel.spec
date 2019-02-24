%global debug_package %{nil}

Name: %{_cross_os}kernel
Version: 4.14.102
Release: 1%{?dist}
Summary: The Linux kernel
License: GPLv2 and Redistributable, no modification permitted
URL: https://www.kernel.org/
Source0: https://www.kernel.org/pub/linux/kernel/v4.x/linux-%{version}.tar.xz
Source100: config-%{_cross_arch}
BuildRequires: gcc-%{_cross_target}

%description
%{summary}.

%package headers
Summary: Header files for the Linux kernel for use by glibc

%description headers
%{summary}.

%prep
%setup -q -n linux-%{version}
cp %{SOURCE100} "arch/%{_cross_karch}/configs/%{_cross_vendor}_defconfig"

%build
make -s \
  ARCH="%{_cross_karch}" \
  CROSS_COMPILE="%{_cross_target}-" \
  %{_cross_vendor}_defconfig

%install
make -s \
  ARCH="%{_cross_karch}" \
  CROSS_COMPILE="%{_cross_target}-" \
  INSTALL_HDR_PATH="%{buildroot}%{_cross_prefix}" \
  headers_install

find %{buildroot}%{_cross_prefix} \
   \( -name .install -o -name .check -o \
      -name ..install.cmd -o -name ..check.cmd \) -delete

%files

%files headers
%dir %{_cross_includedir}/asm
%dir %{_cross_includedir}/asm-generic
%dir %{_cross_includedir}/drm
%dir %{_cross_includedir}/linux
%dir %{_cross_includedir}/misc
%dir %{_cross_includedir}/mtd
%dir %{_cross_includedir}/rdma
%dir %{_cross_includedir}/scsi
%dir %{_cross_includedir}/sound
%dir %{_cross_includedir}/video
%dir %{_cross_includedir}/xen
%{_cross_includedir}/asm/*
%{_cross_includedir}/asm-generic/*
%{_cross_includedir}/drm/*
%{_cross_includedir}/linux/*
%{_cross_includedir}/misc/*
%{_cross_includedir}/mtd/*
%{_cross_includedir}/rdma/*
%{_cross_includedir}/scsi/*
%{_cross_includedir}/sound/*
%{_cross_includedir}/video/*
%{_cross_includedir}/xen/*

%changelog
