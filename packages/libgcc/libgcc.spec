# This is a wrapper package that vends a pre-built shared library from
# the SDK, allowing it to be loaded at runtime. It also lets us extract
# debuginfo in the usual way.
%undefine _debugsource_packages

Name: %{_cross_os}libgcc
Version: 0.0
Release: 1%{?dist}
Summary: GCC runtime library
License: GPL-3.0-or-later WITH GCC-exception-3.1
URL: https://gcc.gnu.org/

%description
%{summary}.

%prep
%setup -T -c
cp %{_cross_licensedir}/gcc/COPYING{3,.RUNTIME} .

%build
install -p -m0755 %{_cross_libdir}/libgcc_s.so.1 .

%install
mkdir -p %{buildroot}%{_cross_libdir}
install -p -m0755 libgcc_s.so.1 %{buildroot}%{_cross_libdir}

%files
%license COPYING3 COPYING.RUNTIME
%{_cross_attribution_file}
%{_cross_libdir}/libgcc_s.so.1

%changelog
