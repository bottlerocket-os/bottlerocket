%global _cross_first_party 1
%undefine _debugsource_packages

Name: %{_cross_os}migrations
Version: 0.0
Release: 0%{?dist}
Summary: Settings migrations
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

# Ideally this would be the package name, but for now the build system expects to find a package
# named "bottlerocket-migrations".
Provides: %{_cross_os}settings-migrations

%description
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
# First we find the migrations in the source tree.  We assume the directory name is the same as
# the crate name.
migrations=()
for migration in $(find %{_builddir}/sources/settings-migrations/v[0-9]* -mindepth 1 -maxdepth 1 -type d); do
    migrations+=("-p $(basename ${migration})")
done

# We need to build migrations statically, because they need to run after a system update where
# available libraries can change.
%cargo_build_static --manifest-path %{_builddir}/sources/Cargo.toml ${migrations[*]}

%install
install -d %{buildroot}%{_cross_datadir}/migrations
for version_path in %{_builddir}/sources/settings-migrations/v[0-9]*; do
  [ -e "${version_path}" ] || continue
  for migration_path in "${version_path}"/*; do
    [ -e "${migration_path}" ] || continue

    version="${version_path##*/}"
    crate_name="${migration_path##*/}"
    migration_binary_name="migrate_${version}_${crate_name#migrate-}"
    built_path="%{__cargo_outdir_static}/${crate_name}"
    target_path="%{buildroot}%{_cross_datadir}/migrations/${migration_binary_name}"

    install -m 0555 "${built_path}" "${target_path}"
  done
done

%files
%dir %{_cross_datadir}/migrations
%{_cross_datadir}/migrations
