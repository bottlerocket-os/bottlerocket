%global goproject github.com/awslabs
%global gorepo oci-add-hooks
%global goimport %{goproject}/%{gorepo}

%global gitrev ef29fe312d2e1858d5eb28ab0abe0cbee298a165
%global shortrev %(c=%{gitrev}; echo ${c:0:7})

Name: %{_cross_os}oci-add-hooks
Version: 1.0.0
Release: 1%{?dist}
Summary: OCI runtime wrapper that injects OCI hooks
License: Apache-2.0 AND MIT
URL: https://github.com/awslabs/oci-add-hooks
Source0: %{gorepo}-%{shortrev}.tar.gz
Source1: bundled-%{gorepo}-%{shortrev}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{gitrev}
%setup -T -D -n %{gorepo}-%{gitrev} -b 1

%build
%set_cross_go_flags
export LD_VERSION="-X main.commit=oci-add-hooks-%{gitrev}"
go build ${GOFLAGS} -v -x -buildmode=pie -ldflags="${GOLDFLAGS} ${LD_VERSION}" -o oci-add-hooks

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 oci-add-hooks %{buildroot}%{_cross_bindir}

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE NOTICE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/oci-add-hooks
