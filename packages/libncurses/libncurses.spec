Name: %{_cross_os}libncurses
Version: 6.4
Release: 1%{?dist}
Summary: Ncurses libraries
License: X11
URL: https://invisible-island.net/ncurses/ncurses.html
Source0: https://invisible-mirror.net/archives/ncurses/ncurses-%{version}.tar.gz
Patch1: ncurses-config.patch
Patch2: ncurses-libs.patch
Patch3: ncurses-urxvt.patch
Patch4: ncurses-kbs.patch
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the ncurses libraries.
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n ncurses-%{version} -p1

%build
%cross_configure \
  --disable-big-core \
  --disable-rpath \
  --disable-rpath-hack \
  --disable-stripping \
  --disable-wattr-macros \
  --enable-colorfgbg \
  --enable-const \
  --enable-echo \
  --enable-ext-colors \
  --enable-hard-tabs \
  --enable-overwrite \
  --enable-pc-files \
  --enable-widec \
  --enable-xmc-glitch \
  --with-abi-version=6 \
  --with-ospeed=unsigned \
  --with-pkg-config-libdir=%{_cross_pkgconfigdir} \
  --with-shared \
  --with-terminfo-dirs=%{_cross_sysconfdir}/terminfo:%{_cross_datadir}/terminfo \
  --with-termlib=tinfo \
  --with-ticlib=tic \
  --with-xterm-kbs=DEL \
  --without-ada \
  --without-cxx \
  --without-cxx-binding \
  --without-gpm \
  --without-manpages \
  --without-normal \
  --without-profile \
  --without-progs \
  --without-tests

make %{?_smp_mflags} libs

%install
make DESTDIR=%{buildroot} install.{libs,data,includes}
chmod 755 %{buildroot}%{_cross_libdir}/lib*.so.*.*
chmod 644 %{buildroot}%{_cross_libdir}/lib*.a
mkdir -p %{buildroot}%{_cross_sysconfdir}/terminfo
rm -rf "%{buildroot}%{_cross_libdir}/terminfo"
rm -rf "%{buildroot}%{_cross_datadir}/tabset"

mv "%{buildroot}%{_cross_datadir}/terminfo"{,.bak}
for t in \
  a/ansi \
  d/dumb \
  l/linux \
  s/screen \
  s/screen-256color \
  v/vt100 \
  v/vt102 \
  v/vt200 \
  v/vt220 \
  x/xterm \
  x/xterm+256color \
  x/xterm-256color \
  x/xterm-color \
  x/xterm-xfree86 \
;
do
  install -D -m 0644 \
    %{buildroot}%{_cross_datadir}/terminfo.bak/${t} \
    %{buildroot}%{_cross_datadir}/terminfo/${t}
done
rm -rf "%{buildroot}%{_cross_datadir}/terminfo.bak"

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/lib*.so.6*
%dir %{_cross_datadir}/terminfo
%{_cross_datadir}/terminfo/*

%files devel
%exclude %{_cross_bindir}/ncurses*-config
%{_cross_libdir}/lib*.so
%{_cross_libdir}/lib*.a
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
