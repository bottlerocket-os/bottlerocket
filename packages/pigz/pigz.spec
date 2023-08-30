Name: %{_cross_os}pigz
Version: 2.8
Release: 1%{?dist}
Summary: pigz is a parallel implementation of gzip which utilizes multiple cores
License: Zlib AND Apache-2.0
URL: http://www.zlib.net/pigz
Source0: https://zlib.net/pigz/pigz-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libz-devel

%description
%{summary}.

%prep
%autosetup -n pigz-%{version} -p1

%global set_env \
%set_cross_build_flags \\\
export CC=%{_cross_target}-gcc \\\
%{nil}

%build
%set_env
%make_build CC="${CC}" CFLAGS="${CFLAGS}" LDFLAGS="${LDFLAGS}"

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 unpigz %{buildroot}%{_cross_bindir}

%files
%license README zopfli/COPYING
%{_cross_bindir}/unpigz
%{_cross_attribution_file}

%changelog
