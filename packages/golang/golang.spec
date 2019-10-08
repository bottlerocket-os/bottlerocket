# Disable debug symbol extraction and packaging.
%global debug_package %{nil}
%global __objdump /bin/true
%global __strip /bin/true
%global _build_id_links none

Name: %{_cross_os}golang
Version: 1.13.1
Release: 1%{?dist}
Summary: The Go Progamming Language
License: BSD and Public Domain
URL: https://golang.org

%if %{_build_cpu} == aarch64
Source0: https://dl.google.com/go/go%{version}.linux-arm64.tar.gz
%else
Source0: https://dl.google.com/go/go%{version}.linux-amd64.tar.gz
%endif

%description
🙊⚙️

%prep

%build

%install
mkdir -p %{buildroot}%{_prefix}/lib
tar xf %{SOURCE0} -C %{buildroot}%{_prefix}/lib

mkdir -p %{buildroot}%{_bindir}
for g in go godoc gofmt ; do
  ln -s ../lib/go/bin/${g} %{buildroot}%{_bindir}/${g}
done

%files
%{_bindir}/go
%{_bindir}/godoc
%{_bindir}/gofmt
%dir %{_prefix}/lib/go
%{_prefix}/lib/go/*

%changelog
