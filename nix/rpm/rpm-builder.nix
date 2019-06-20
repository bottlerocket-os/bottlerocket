{ stdenvNoCC, system, lib, writeScript, linkFarm, docker-cli, docker-container,
  docker-load, rpm-container, rpm-macros, fetchRpmSources }:
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

      spec = builtins.head (lib.sourceByRegex src ["${name}.spec"]);
      sources = builtins.head (lib.sourceByRegex src ["sources"]);

      rpmbuildFarm = let
        linkInDir = dir: elems:
          map (s: { name = "${dir}/${s.name}"; path = "${s}"; }) elems;
        linkInRoot = elems:
          map (s: { name = "."; path = "${s}"; }) elems;

        rpmSources = fetchRpmSources { inherit spec sources; };

        rpms = linkInDir "RPMS" rpmInputs;
        sources = linkInDir "SOURCES" rpmSources;
        packageSrc = linkInDir "SOURCES" ([src] ++ srcs);
        
        links = rpms ++ sources ++ packageSrc;
      in
        linkFarm "rpmbuildFarm" links;

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
      
      # Setup the builder user
      cd /home/builder
      export HOME=/home/builder
      su --preserve-environment builder rpmdev-setuptree
      # And build.
      su --preserve-environment builder ${rpmTreeScript}
      su --preserve-environment builder ${rpmBuildScript}

      ${postBuildPhase}
      '';

      # RPM tree setup for build
      rpmTreeScript = writeScript "rpmbuild-setup" ''
      # Setup required macros
      find ${rpm-macros} ${rpm-macros.arches} -type f > ~/.rpmmacros
      
      rpmdev-setuptree
      '';

      rpmBuildScript = writeScript "rpmbuild-build" ''
      pushd rpmbuild

      rpmbuild -ba --clean 
               --define '_sourcedir ${rpmbuildFarm}/SOURCES' \
               --define '_specdir ${rpmbuildFarm}/SOURCES' \
                /${name}.spec
      popd
      '';
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
