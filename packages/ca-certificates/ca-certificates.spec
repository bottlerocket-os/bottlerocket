Name: %{_cross_os}ca-certificates
Version: 2021.05.25
Release: 1%{?dist}
Summary: CA certificates extracted from Mozilla
License: MPL-2.0
# Note: You can see changes here:
# https://hg.mozilla.org/projects/nss/log/tip/lib/ckfw/builtins/certdata.txt
URL: https://curl.haxx.se/docs/caextract.html
Source0: https://curl.haxx.se/ca/cacert-2021-05-25.pem
Source1: ca-certificates-tmpfiles.conf

%description
%{summary}.

%prep

%build

%install
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/pki/tls/certs
install -p -m 0644 %{S:0} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/pki/tls/certs/ca-bundle.crt

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/ca-certificates.conf

%files
%{_cross_attribution_file}
%dir %{_cross_factorydir}%{_cross_sysconfdir}/pki
%dir %{_cross_factorydir}%{_cross_sysconfdir}/pki/tls
%dir %{_cross_factorydir}%{_cross_sysconfdir}/pki/tls/certs
%{_cross_factorydir}%{_cross_sysconfdir}/pki/tls/certs/ca-bundle.crt
%{_cross_tmpfilesdir}/ca-certificates.conf

%changelog
