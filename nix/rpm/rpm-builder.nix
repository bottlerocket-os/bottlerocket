{ stdenvNoCC, system, lib, writeScript, docker-cli, docker-container, docker-load, rpm-container, rpm-macros }:
let
  mkDockerDerivation =
    { name,
      entrypoint ? "/bin/sh", image ? rpm-container,
      src ? null, srcs ? null, rpmInputs ? [],
      preBuildPhase ? "", postBuildPhase ? "",
      useHostNetwork ? false, ... }@args:
    let
      # Load the rpm builder container and use its ref for running.
      imageRef = lib.fileContents (docker-load { inherit image; });

      su = "su --preserve-environment builder";
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
      
      # Setup the builder user
      cd /home/builder
      export HOME=/home/builder
      su --preserve-environment builder rpmdev-setuptree
      # And build.
      su --preserve-environment builder ${rpmBuildScript}

      ${postBuildPhase}
      '';

      rpmBuildScript = writeScript "container-rpm-build-script" ''
      # Link required macros in.
      find ${rpm-macros} ${rpm-macros.arch} -type f > ~/.rpmmacros
      
      rpmdev-setuptree

      set -x

      # Symlink input rpms that are used in the build and make them available for use.
      ${lib.concatMapStringsSep "\n"
        (p: "find ${p} -name '*.rpm' -exec ln -sv {} ./rpmbuild/RPMS/") rpmInputs}
      # Symlink sources (that are files)
      find -L "''${srcs[@]}" -type f -maxdepth 0 \
            -exec ln -vs {} ./rpmbuild/SOURCES/ \;
      # Symlink sources' children (from those that are directories)
      find -L "''${srcs[@]}" -type d -maxdepth 0 -print0  | xargs -0 -L1 -I DIR -- find DIR -mindepth 1 -maxdepth 1 \
           -exec ln -sv {} ./rpmbuild/SOURCES/ \;
      find -L "''${srcs[@]}" -maxdepth 1 -type f -name '*.spec' -exec ln -sv {} ./rpmbuild/SPECS \;
      set +x
      
      rpmbuild -ba --clean ./rpmbuild/SPECS/${name}.spec
      '';

      netMode = if useHostNetwork then "host" else "none";
    in
      stdenvNoCC.mkDerivation ({
        inherit name;
        
        phases = [ "setupPhase" "buildPhase" ];
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
                                         --env "out=$containerOut/out" \
                                         $containerSetupArgs \
                                         -e src -e srcs -e outputs \
                                         ${imageRef} "${containerBuildScript}"
        mv containerOut/out $out
        '';
      } // args);
in
{
  inherit mkDockerDerivation;
  mkDerivation = mkDockerDerivation;
}
