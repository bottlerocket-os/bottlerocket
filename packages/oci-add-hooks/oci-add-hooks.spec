%global goproject github.com/awslabs
%global gorepo oci-add-hooks
%global goimport %{goproject}/%{gorepo}

%global gitrev ef29fe312d2e1858d5eb28ab0abe0cbee298a165
%global shortrev %(c=%{gitrev}; echo ${c:0:7})
%global gosimplejson 0.5.0
%global jsonlosslessrev e0cd1ca6349bf167e33d44f28c14c728a277205f
%global jsonlosslessshort %(c=%{jsonlosslessrev}; echo ${c:0:7})

Name: %{_cross_os}oci-add-hooks
Version: 1.0.0
Release: 1%{?dist}
Summary: OCI runtime wrapper that injects OCI hooks
License: Apache-2.0 and MIT
URL: https://github.com/awslabs/oci-add-hooks
Source0: https://%{goimport}/archive/%{gorev}/%{gorepo}-%{shortrev}.tar.gz
Source1: https://github.com/bitly/go-simplejson/archive/v%{gosimplejson}/go-simplejson-%{gosimplejson}.tar.gz
Source2: https://github.com/joeshaw/json-lossless/archive/%{jsonlosslessrev}/json-lossless-%{jsonlosslessshort}.tar.gz

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n %{gorepo}-%{gitrev}
%cross_go_setup %{gorepo}-%{gitrev} %{goproject} %{goimport}

# We need to manage these third-party dependencies because the oci-add-hooks
# "release" that we use doesn't include the `vendor` directory, unlike our other
# go third party dependencies
mkdir -p GOPATH/src/github.com/bitly/go-simplejson GOPATH/src/github.com/joeshaw/json-lossless
tar -C GOPATH/src/github.com/bitly/go-simplejson -xzf %{SOURCE1} --strip 1
cp GOPATH/src/github.com/bitly/go-simplejson/LICENSE LICENSE.go-simplejson
tar -C GOPATH/src/github.com/joeshaw/json-lossless -xzf %{SOURCE2} --strip 1
cp GOPATH/src/github.com/joeshaw/json-lossless/LICENSE LICENSE.json-lossless

%build
%cross_go_configure %{goimport}
# We use `GO111MODULE=off` to force golang to look for the dependencies in the GOPATH
GO111MODULE=off go build -buildmode=pie -ldflags "-linkmode=external" -o oci-add-hooks

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 oci-add-hooks %{buildroot}%{_cross_bindir}

%files
%license LICENSE NOTICE LICENSE.go-simplejson LICENSE.json-lossless
%{_cross_attribution_file}
%{_cross_bindir}/oci-add-hooks
