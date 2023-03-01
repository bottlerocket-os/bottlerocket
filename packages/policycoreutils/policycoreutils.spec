Name: %{_cross_os}policycoreutils
Version: 3.5
Release: 1%{?dist}
Summary: A set of SELinux policy tools
License: GPL-2.0-only
URL: https://github.com/SELinuxProject/
Source0: https://github.com/SELinuxProject/selinux/releases/download/%{version}/policycoreutils-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libsemanage-devel
BuildRequires: %{_cross_os}libsepol-devel
Requires: %{_cross_os}libselinux
Requires: %{_cross_os}libselinux-utils
Requires: %{_cross_os}libsemanage
Requires: %{_cross_os}libsepol

%description
%{summary}.

%prep
%autosetup -n policycoreutils-%{version} -p1

%global set_env \
%set_cross_build_flags \\\
export CC="%{_cross_target}-gcc" \\\
export DESTDIR='%{buildroot}' \\\
export PREFIX='%{_cross_prefix}' \\\
export SBINDIR='%{_cross_sbindir}' \\\
export LOCALEDIR='%{_cross_localedir}' \\\
%{nil}

%build
%set_env
for dir in load_policy semodule sestatus setfiles ; do
  %make_build -C ${dir}
done

%install
%set_env
for dir in load_policy semodule sestatus setfiles ; do
  %make_install -C ${dir}
done
# remove unneeded compatibility symlink
rm %{buildroot}%{_cross_sbindir}/sestatus

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_sbindir}/load_policy
%{_cross_sbindir}/semodule
%{_cross_bindir}/sestatus
%{_cross_sbindir}/setfiles
%exclude %{_cross_sbindir}/genhomedircon
%exclude %{_cross_sbindir}/restorecon
%exclude %{_cross_sbindir}/restorecon_xattr
%exclude %{_cross_bashdir}
%exclude %{_cross_mandir}
%exclude %{_cross_sysconfdir}

%changelog
