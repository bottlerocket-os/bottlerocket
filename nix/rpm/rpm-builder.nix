{ stdenvNoCC, system, lib, writeScript,
  docker-cli, docker-container, docker-load, rpm-container,
  rpm-macros, fetchRpmSources }:
let
  mkDockerDerivation =
    { name,
      entrypoint ? "/bin/sh", image ? rpm-container,
      src ? null, srcs ? [],
      rpmInputs ? [], rpmSources ? [],
      preBuildPhase ? "", postBuildPhase ? "",
      useHostNetwork ? false, ... }@args:
    let
      # Load the rpm builder container and use its ref for running.
      imageRef = lib.fileContents (docker-load { inherit image; });
      # Networking mode for the building container.
      netMode = if useHostNetwork then "host" else "none";
      spec = "${src}/${name}.spec";
      sources = "${src}/sources";
      rpmSources' = if rpmSources == []
                    then (fetchRpmSources { inherit name spec sources; })
                    else rpmSources;

      macrosContent = "find -L ${rpm-macros} ${rpm-macros.arches}/x86_64 -type f -exec cat {} \\;";

      # Build script executed in the container.
      containerBuildScript = writeScript "container-build-script" ''
      set -e
      # Catch early exit and run teardown to allow host user to
      # manipulate $out if used.
      trap ${docker-container.teardown} EXIT

      ${preBuildPhase}

      # Create user for the build, it will match the build user's
      # uid/gid on the building host.
      groupadd builder -g $egid
      useradd builder --uid $euid --gid $egid --create-home --no-user-group

      # Setup the builder user and build
      cd /home/builder
      export HOME=/home/builder
      # Prepare rpmbuild dir and provide repository for dependencies.
      su --preserve-environment builder ${rpmTreeScript}

      echo "Installing RPM macros for dnf"
      mkdir -p /etc/rpm
      ${macrosContent} | tee /etc/rpm/macros

      dnf builddep --assumeyes --cacheonly \
                   --repofrompath build-inputs,/home/builder/rpmbuild/rpmInputs \
                   ${spec}
      su --preserve-environment builder ${rpmBuildScript}

      ${postBuildPhase}
      '';

      # RPM tree setup for build
      rpmTreeScript = writeScript "rpmbuild-setup" ''
      # Setup required macros
      ${macrosContent} | tee ~/.rpmmacros
      mkdir -p /build/rpmbuild
      ln -sv /build/rpmbuild rpmbuild

      rpmdev-setuptree

      mkdir ./rpmbuild/rpmInputs
      ${lib.concatMapStringsSep "\n" (s: "ln -s ${s}/*.rpm ./rpmbuild/rpmInputs/") rpmInputs}
      createrepo_c ./rpmbuild/rpmInputs

      ${lib.concatMapStringsSep "\n" (s: "ln -s ${s} ./rpmbuild/SOURCES/${s.name}") rpmSources'}
      ln -sv ${src}/* ./rpmbuild/SOURCES/
      ln -s ${spec} ./rpmbuild/SPECS/
      '';

      rpmBuildScript = writeScript "rpmbuild-build" ''
      pushd rpmbuild

      time rpmbuild -ba --clean SPECS/${name}.spec

      mkdir -p $out/srpms $out/rpms

      echo "Copying SRPMS and RPMS from successful build"
      find SRPMS -type f -exec cp -v {} $out/srpms \;
      find RPMS -type f -exec cp -v {} $out/rpms  \;

      popd
      '';
    in
      stdenvNoCC.mkDerivation ({
        inherit name;

        phases = [ "setupPhase" "buildPhase" ];
        outputs = [ "rpms" "srpms" ];
        setOutputFlags = false;
        out = "rpms";
        buildInputs = [ docker-cli ];

        setupPhase = ''
        # Get the sandbox details for docker's perspective.
        source "${docker-container.setup}"

        # Setup a space for the container to write out to us with the appropriate permissions.
        containerOut="$sandboxBuild/containerOut"
        mkdir -p containerOut
        '';

        buildPhase = ''
        docker run --rm --entrypoint "/bin/sh" --userns=host --net=${netMode} \
                                         --volume "$NIX_STORE:$NIX_STORE:ro" \
                                         --volume "$containerOut:$containerOut" \
                                         --env "out=$containerOut" \
                                         --tmpfs /build:rw,size=8G,mode=1777,exec \
                                         $containerSetupArgs \
                                         -e src -e srcs -e outputs \
                                         ${imageRef} "${containerBuildScript}"
        mv containerOut/srpms $srpms
        mv containerOut/rpms $rpms
        '';
      } // args);
in
{
  inherit mkDockerDerivation;
  mkDerivation = mkDockerDerivation;
}
