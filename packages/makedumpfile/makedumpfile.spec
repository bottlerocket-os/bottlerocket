Name: %{_cross_os}makedumpfile
Version: 1.7.3
Release: 1%{?dist}
Summary: Tool to create dumps from kernel memory images
License: GPL-2.0-or-later AND GPL-2.0-only
URL: https://github.com/makedumpfile/makedumpfile
Source0: https://github.com/makedumpfile/makedumpfile/archive/%{version}/makedumpfile-%{version}.tar.gz

# First party patches from 0 to 1000
Patch0000: 0000-fix-strip-invocation-for-TARGET-env-variable.patch

BuildRequires: %{_cross_os}libbzip2-devel
BuildRequires: %{_cross_os}libz-devel
BuildRequires: %{_cross_os}libelf-devel
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}libbzip2
Requires: %{_cross_os}libelf
Requires: %{_cross_os}libz

%description
%{summary}.

%prep
%autosetup -n makedumpfile-%{version}

%global set_env \
%set_cross_build_flags \\\
export CC=%{_cross_target}-gcc \\\
export DESTDIR=%{buildroot}%{_cross_rootdir} \\\
export TARGET=%{_cross_arch} \\\
export LINKTYPE="dynamic" \\\
export USELZO="off" \\\
export USESNAPPY="off" \\\
%{nil}

%build
%set_env
%make_build

%install
%set_env
make install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_sbindir}/makedumpfile
%exclude %{_cross_mandir}
%exclude %{_cross_sbindir}/makedumpfile-R.pl
%exclude %{_cross_prefix}/share/makedumpfile
