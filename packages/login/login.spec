Name: %{_cross_os}login
Version: 0.0.1
Release: 1%{?dist}
Summary: A login that doesn't actually allow logins
License: FIXME
Source0: login.c
BuildRequires: %{_cross_os}glibc-devel

# This package should only be installed if there is no shell.
Conflicts: %{_cross_os}bash

%description
%{summary}.

%prep

%build
%set_cross_build_flags
%{_cross_target}-gcc ${CFLAGS} ${LDFLAGS} -o login %{S:0}

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 login %{buildroot}%{_cross_bindir}/login

%files
%{_cross_bindir}/login

%changelog
