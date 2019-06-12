{ stdenv, docker-cli, lib }:
{ name, image, containerShell ? "/bin/bash",
  correctPermissions ? true,
  useHostNetworking ? true,
  extraArgs ? [],
  # Give hooks to manipulate the build before, in, in-after, and after
  # the container run.
  containerPhase,
  preContainerPhase ? ":",
  preBuildPhase ? ":",
  postBuildPhase ? ":",
  postContainerPhase ? ":", ... }:
let
# Environment variables to include in container environment.
passthroughEnv = ["out"];
passthroughEnvArgs = (map (env: ["--env" env]) passthroughEnv);

dockerArgs = [
  "--interactive"
  "--rm"
  "--entrypoint" containerShell
  "--volume" "/nix/store:/nix/store:ro"
  "--volume" "$(pwd)/container-out:/container-out"
  "--env" "uid=$(id -u)"
  "--env" "gid=$(id -g)"
  ] ++ passthroughEnvArgs
    ++ lib.optional useHostNetworking [ "--net" "host" ]
    ++ extraArgs;
    
dockerFlags = builtins.concatStringsSep " " (lib.flatten dockerArgs);
in
stdenv.mkDerivation {
  inherit name;

  phases = [ "preBuildPhase" "buildPhase" "postBuildPhase" "installPhase" ];
  inherit preBuildPhase postBuildPhase;

  # Interacting with Docker and trying to get outputs in the current
  # arrangement requires host and container paths to map through the
  # docker container.
  __noChroot = true;

  buildInputs = [ docker-cli ];

  buildPhase = ''
  mkdir container-out
  set -x
  env
  docker run ${dockerFlags} ${image} <<'END_CONTAINER_BUILD'
  # Inside the container
  out="/container-out$out"

  ${preContainerPhase}
  ${containerPhase}
  ${postContainerPhase}

  ${lib.optionalString correctPermissions "chown -R $uid:$gid /container-out"}

  END_CONTAINER_BUILD
  echo $?
  set +x
  '';

  installPhase = ''
  test -e ./container-out$out
  cp --archive ./container-out$out $out
  '';
}
