# Don't generate debug packages because we are compiling without CGO,
# and the `gc` compiler doesn't append the  the ".note.gnu.build-id" section
# https://fedoraproject.org/wiki/PackagingDrafts/Go#Build_ID
%global debug_package %{nil}

%global goproject github.com/bottlerocket
%global gorepo hotdog
%global goimport %{goproject}/%{gorepo}

%global gitrev 3f2ca9275fae8db87409c3a0999aa2c8a4bd44d1
%global shortrev %(c=%{gitrev}; echo ${c:0:7})

Name: %{_cross_os}hotdog
Version: 1.0.1
Release: 1%{?dist}
Summary: Tool with OCI hooks to run the Log4j Hot Patch in containers
License: Apache-2.0
URL: https://github.com/awslabs/oci-add-hooks

BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}log4j2-hotpatch

%description
%{summary}.

%prep
%setup -T -c
cp -r /home/builder/src/%{gorepo}-%{gitrev}/* .

%build
%set_cross_go_flags

# Set CGO_ENABLED=0 to statically link hotdog-hotpath, since it runs inside containers that
# may not have the glibc version used to compile it
CGO_ENABLED=0 go build ${GOFLAGS} -installsuffix cgo -a -ldflags "-s" -o hotdog-hotpatch ./cmd/hotdog-hotpatch

# The oci hooks commands can be compiled as we usually compile golang packages
for cmd in hotdog-cc-hook hotdog-poststart-hook; do
  go build ${GOFLAGS} -buildmode=pie -ldflags "${GOLDFLAGS}" -o $cmd ./cmd/$cmd
done

%install
install -d %{buildroot}%{_cross_libexecdir}/hotdog/
install -d %{buildroot}%{_cross_datadir}/hotdog/

install -p -m 0755 hotdog-hotpatch %{buildroot}%{_cross_datadir}/hotdog/

for cmd in hotdog-cc-hook hotdog-poststart-hook; do
  install -p -m 0755 $cmd %{buildroot}%{_cross_libexecdir}/hotdog
done

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%dir %{_cross_libexecdir}/hotdog
%dir %{_cross_datadir}/hotdog
%{_cross_libexecdir}/hotdog/hotdog-cc-hook
%{_cross_libexecdir}/hotdog/hotdog-poststart-hook
%{_cross_datadir}/hotdog/hotdog-hotpatch
