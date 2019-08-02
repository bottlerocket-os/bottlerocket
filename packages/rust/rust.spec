# Disable debug symbol extraction and packaging.
%global debug_package %{nil}
%global __strip /bin/true
%global _build_id_links none

# Skip check-rpaths since rust needs them to find its libs, and this package
# doesn't end up on final systems.
%global __arch_install_post /usr/lib/rpm/check-buildroot

Name: %{_cross_os}rust
Version: 1.36.0
%global cargo_version 0.37.0
Release: 1%{?dist}
Summary: The Rust Progamming Language
License: ASL 2.0 or MIT
URL: https://www.rust-lang.org

Source0: https://static.rust-lang.org/dist/rustc-%{version}-x86_64-unknown-linux-gnu.tar.xz
Source1: https://static.rust-lang.org/dist/cargo-%{cargo_version}-x86_64-unknown-linux-gnu.tar.xz
Source2: https://static.rust-lang.org/dist/rust-std-%{version}-%{_cross_arch}-unknown-linux-%{_cross_libc}.tar.xz

%description
🦀⚙️

# Packages containing binaries meant to execute on the host system
# are kept as architecture-specific, since we will install and run
# them on systems of that type. Packages containing libraries for the
# target system are marked as "noarch", since although they can be
# installed, they are not native, and the resulting binaries must be
# executed elsewhere.

%prep
%autosetup -c -T
xz -dc %{SOURCE0} | tar -xof -
xz -dc %{SOURCE1} | tar -xof -
xz -dc %{SOURCE2} | tar -xof -

%build
# whole lot of nothin'

%install
for dir in \
    rustc-%{version}-x86_64-unknown-linux-gnu \
    cargo-%{cargo_version}-x86_64-unknown-linux-gnu \
    rust-std-%{version}-%{_cross_arch}-unknown-linux-%{_cross_libc} \
; do
    pushd $dir
    ./install.sh --destdir=%{buildroot} --disable-ldconfig \
        --prefix=%{_prefix}
    popd
done
# remove installer cruft (this can't just be %excluded because RPM complains about builddir references)
rm %{buildroot}%{_prefix}/lib/rustlib/install.log
rm %{buildroot}%{_prefix}/lib/rustlib/uninstall.sh
rm %{buildroot}%{_prefix}/lib/rustlib/rust-installer-version
rm %{buildroot}%{_prefix}/lib/rustlib/components
rm %{buildroot}%{_prefix}/lib/rustlib/manifest-*

%files
%{_bindir}/cargo
%{_bindir}/rustc
%{_bindir}/rustdoc
%exclude %{_bindir}/rust-gdb
%exclude %{_bindir}/rust-gdbgui
%exclude %{_bindir}/rust-lldb
%{_prefix}/lib/*.so
%{_prefix}/lib/rustlib/%{_cross_arch}-unknown-linux-%{_cross_libc}
%exclude %{_prefix}/lib/rustlib/etc
%exclude %{_docdir}
%exclude %{_mandir}
%exclude %{_datarootdir}/zsh
%exclude %{_prefix}%{_sysconfdir}/bash_completion.d

%changelog
