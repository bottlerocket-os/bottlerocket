# This is a wrapper package that vends a pre-built shared library from
# the SDK, allowing it to be loaded at runtime. It also lets us extract
# debuginfo in the usual way.
%undefine _debugsource_packages

Name: %{_cross_os}libstd-rust
Version: 0.0
Release: 1%{?dist}
Summary: Rust standard library
License: Apache-2.0 OR MIT
URL: https://www.rust-lang.org/

%description
%{summary}.

%prep
%setup -T -c
cp /usr/share/licenses/rust/* .

%build
install -p -m0755 %{_libexecdir}/rust/lib/rustlib/%{__cargo_target}/lib/libstd-*.so .

%install
mkdir -p %{buildroot}%{_cross_libdir}
install -p -m0755 libstd-*.so %{buildroot}%{_cross_libdir}

%files
%license COPYRIGHT LICENSE-APACHE LICENSE-MIT
%{_cross_attribution_file}
%{_cross_libdir}/libstd-*.so

%changelog
