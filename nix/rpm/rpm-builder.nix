{ stdenvNoCC, system, lib, writeScript,
  docker-cli, docker-image, docker-container, docker-load, rpm-container, rpm-metadata,
  rpm-macros, fetchRpmSources }:
let
  mkDockerDerivation =
    { # name of the package
      name,
      # entrypoint of the container
      entrypoint ? "/bin/sh",
      # docker image to use for the build
      image ? rpm-container,
      # src of the package
      src ? null,
      # srcs (multiple) of the package if needed from multiple sources
      # - should not be the sources from the file named "sources".
      srcs ? [],
      # rpmInputs are the rpms that are provided as input to this
      # build as a repository.
      rpmInputs ? [], rpmInputsFn ? (metadata: metadata.dependentPackages),
      # rpmHostInputs are the BuildRequires needed on the "host"
      # building the package.
      rpmHostInputs ? [],
      # rpmSources provided directly to control the used sources
      # instead of automatically parsing and loading them.
      rpmSources ? null, doFetchRpmSources ? true,
      # rpmbuildExtraFlags are provided to the rpmbuild command used.
      rpmbuildExtraFlags ? "",
      # builddepExtraFlags are provided to the command used to install
      # dependencies prior to build in addition to the existing
      # set. This may be used, for example, to set additional options
      # for dnf or to restrict operations to a specific repository.
      builddepExtraFlags ? "",
      # pre and postRpmbuildCommands are executed just before and
      # after the rpmbuild command is run allowing for additional
      # steps to be taken prior to starting or concluding the build.
      preRpmbuildCommands ? "", postRpmbuildCommands ? "",
      # pre and postBuildPhase are executed just before and after the
      # container builder is executed allowing for additional fork on
      # the build.
      preBuildPhase ? "", postBuildPhase ? "",
      # allowBuilddepDownload enables the container to fetch new
      # dependencies as needed. This option introduces many unknowns
      # but is useful for developmental purposes where a base-level
      # change (here) or in the base container would result in a wider
      # rebuild.
      allowBuilddepDownload ? false,
      # useHostNetwork enables the container's network stack to reach
      # the internet rather than be isolated on its own.
      useHostNetwork ? false,
      # Reflexivity (varargs like); allows for unhandled arguments to
      # be provided at call sites.
      ... }@args:

    # Downloading builddeps requires networking - and we're only
    # allowing via host networking.
    assert with lib; assertMsg (allowBuilddepDownload -> useHostNetwork)
      "useHostNetwork is required to download dependencies.";
    assert with lib; assertMsg (all (x: hasAttr "rpms" x) rpmInputs)
      "rpmInputs provided must have an 'rpms' output";
    assert with lib; assertMsg (src != null)
      "src must be provided and contain the packaging source";

    let
      # Networking mode for the building container.
      netMode = if useHostNetwork then "host" else "none";

      spec = "${src}/${name}.spec";
      sources = "${src}/sources";

      rpmMetadata = rpm-metadata { inherit name spec sources; };
      rpmHostInputs' = rpmHostInputs ++ rpmMetadata.hostBuildRequires;
      rpmInputs' = rpmInputs ++ (rpmInputsFn rpmMetadata);

      # Upstream sources referenced in spec.
      rpmSources' = if rpmSources == null
                    then (fetchRpmSources { inherit name spec sources; })
                    else lib.optionals (rpmSources != []) rpmSources;

      srcs' = if src == null
              then srcs
              else [ src ] ++ srcs;

      passthru = { inherit rpmMetadata; rpmHostInputs = rpmHostInputs'; };

      # Snippet printing combined macros used by rpmbuild and dnf.
      macrosContent = "find -L ${rpm-macros} ${rpm-macros.arches}/x86_64 -type f -exec cat {} \\;";

      # Build script executed in the container managing the full run
      # of the build.
      #
      # 1. Setup the build user to match executing user
      # 2. Setup a rpmbuild tree in that user's home
      # 3. Build!
      # 4. Export results for nix
      #
      containerBuildScript = writeScript "container-build-script" ''
      set -e
      # Catch early exit and run teardown to allow host user to
      # manipulate $out if used.
      trap "originalExit=$?; ${docker-container.teardown}; exit $originalExit;" EXIT

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

      echo "Setup thar macros for dnf builddep to parse"
      mkdir -p /etc/rpm
      ${macrosContent} > /etc/rpm/macros

      # Install the build dependencies allowing ONLY the inputs as installable.
      dnf builddep --assumeyes ${lib.optionalString  (!allowBuilddepDownload) "--disablerepo '*'"} \
                   --enablerepo build-inputs \
                   --repofrompath build-inputs,/home/builder/rpmbuild/rpmInputs \
                   --setopt build-inputs.gpgcheck=False \
                   ${builddepExtraFlags} \
                   ${spec}
      su --preserve-environment builder ${rpmBuildScript}

      ${postBuildPhase}
      '';

      # RPM tree setup for build
      rpmTreeScript = writeScript "rpmbuild-setup" ''
      # Setup thar macros for build
      ${macrosContent} > ~/.rpmmacros
      mkdir -p /build/rpmbuild
      ln -sv /build/rpmbuild rpmbuild

      rpmdev-setuptree

      mkdir ./rpmbuild/rpmInputs
      ${lib.concatMapStringsSep "\n" (s: "ln -sv ${s.rpms}/*.rpm ./rpmbuild/rpmInputs/") rpmInputs'}
      createrepo_c ./rpmbuild/rpmInputs

      ${lib.concatMapStringsSep "\n" (s: "ln -s ${s} ./rpmbuild/SOURCES/${s.name}") rpmSources'}
      ${lib.concatMapStringsSep "\n" (s: "ln -sv ${s}/* ./rpmbuild/SOURCES/") srcs'}
      ln -s ${spec} ./rpmbuild/SPECS/
      '';

      rpmBuildScript = writeScript "rpmbuild-build" ''
      set -e
      pushd rpmbuild
      ${preRpmbuildCommands}
      time rpmbuild ${rpmbuildExtraFlags} -ba SPECS/${name}.spec
      ${postRpmbuildCommands}

      mkdir -p $out/srpms $out/rpms

      echo "Copying SRPMS and RPMS from successful build"
      find SRPMS -type f -exec cp -v {} $out/srpms \;
      find RPMS -type f -exec cp -v {} $out/rpms  \;

      popd
      '';
    in
      stdenvNoCC.mkDerivation ({
        inherit name passthru;

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

        # Ensure the docker image is loaded
        ${rpm-container.docker.loader}
        '';

        buildPhase = ''
        docker run --rm --entrypoint "/bin/sh" --userns=host --net=${netMode} \
                                         --volume "$NIX_STORE:$NIX_STORE:ro" \
                                         --volume "$containerOut:$containerOut" \
                                         --env "out=$containerOut" \
                                         --tmpfs /build:rw,size=8G,mode=1777,exec \
                                         $containerSetupArgs \
                                         -e src -e srcs -e outputs \
                                         ${rpm-container.docker.ref} "${containerBuildScript}"
        mv containerOut/srpms $srpms
        mv containerOut/rpms $rpms
        '';
      } // args);
in
{
  inherit mkDockerDerivation;
  mkDerivation = mkDockerDerivation;
}
